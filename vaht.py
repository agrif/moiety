#!/usr/bin/python

import sys
import os.path

import ctypes
import ctypes.util
from ctypes import c_int, c_uint8, c_int16, c_uint16, c_uint32, c_int32
from ctypes import c_void_p, c_char_p, POINTER

# py3k compatibility
if sys.version_info[0] == 3:
    long = int

# custom library finder
def find_library(name):
    path = ctypes.util.find_library(name)
    if path:
        return path
    
    # OSX will not find the dylib if it's installed in /usr/local
    if sys.platform.startswith('darwin'):
        if os.path.isfile('/usr/local/lib/lib%s.dylib' % (name,)):
            return '/usr/local/lib/lib%s.dylib' % (name,)
        
        # fallback
        return 'lib%s.dylib' % (name,)
    elif sys.platform.startswith('linux'):
        return "lib%s.so" % (name,)
    
    return None

# create our library pointers
# FIXME cross-platform loading
try:
    vaht = ctypes.CDLL(find_library('vaht'))
    vaht.vaht_archive_open
except AttributeError:
    raise RuntimeError("could not find libvaht")
try:
    stdc = ctypes.CDLL(find_library('c'))
    stdc.free
except AttributeError:
    raise RuntimeError("could not find libc")

# standard c free (used occasionally)
free = stdc.free
free.argtypes = [c_void_p]
free.restype = None

##
## Metaclass and Base Classes
##

class VahtMetaclass(type):
    """This metaclass searches for method signatures in
    Class.Methods.methname, and looks up the corresponding vaht_class_*
    method, sets the signature, then puts it into the main class. For
    example: 
    
    class Example:
        __metaclass__ = VahtMetaclass
        class Methods:
            method_name = (None, [c_void_p, c_char_p])
        class Properties:
            prop = (c_void_p, NBT, lambda ptr: NBT(ptr))
    
    will create a class where Example._method_name maps directly to
    vaht_example_method_name, set up to take a void* and a char* and
    return void. Also, it will create a property prop, with get_prop
    and set_prop using c-type c_void_p, python type NBT, and
    converting from C to python using NBT(ptr).
    """
    
    def __new__(cls, name, bases, dct):
        if not 'Methods' in dct:
            return super(VahtMetaclass, cls).__new__(cls, name, bases, dct)
        
        m = dct['Methods']
        p = dct.get('Properties', object())
        prefix = 'vaht_' + getattr(m, '_prefix_', name.lower()) + '_'
        del dct['Methods']
        if 'Properties' in dct:
            del dct['Properties']
        obj = super(VahtMetaclass, cls).__new__(cls, name, bases, dct)
        
        for methname in dir(m):
            if methname.startswith('__'):
                continue
            res, args = getattr(m, methname)
            methname = methname.rstrip('_')
            meth = getattr(vaht, prefix + methname)
            
            meth.restype = res
            meth.argtypes = args
            
            setattr(obj, '_' + methname, meth)
        for propname in dir(p):
            if propname.startswith('__'):
                continue
            try:
                ctype, pytype, convert, check = getattr(p, propname)
            except ValueError:
                ctype, pytype, convert = getattr(p, propname)
                check = None
            propname = propname.rstrip('_')
            getmeth = getattr(vaht, prefix + 'get_' + propname)
            setmeth = getattr(vaht, prefix + 'set_' + propname)
            
            if not convert:
                convert = lambda s: s
            if not check:
                check = lambda o: True
            
            getmeth.restype = ctype
            getmeth.argtypes = [c_void_p]
            setmeth.restype = None
            setmeth.argtypes = [c_void_p, ctype]
            
            def gen_functions(propname, getmeth, setmeth, ctype, pytype, convert, check):
                def get_prop(self):
                    if not check(self):
                        raise TypeError("could not get %s, %s is incorrect type" % (propname, obj.__name__))
                    return convert(getmeth(self))
                def set_prop(self, val):
                    if not check(self):
                        raise TypeError("could not set %s, %s is incorrect type" % (propname, obj.__name__))
                    if not isinstance(val, pytype):
                        raise TypeError("could not set %s, value not a %s" % (propname, pytype.__name__))
                    setmeth(self, val)
                return get_prop, set_prop
            
            get_prop, set_prop = gen_functions(propname, getmeth, setmeth, ctype, pytype, convert, check)
            
            setattr(obj, 'get_' + propname, get_prop)
            setattr(obj, 'set_' + propname, set_prop)
            setattr(obj, propname, property(get_prop, set_prop))
        return obj

# python3 handles metaclasses differently, so this hack is needed
VahtMetaclassObject = VahtMetaclass('VahtMetaclassObject', (object,), {})

class VahtObject(VahtMetaclassObject):
    """This is a superclass for all simple libvaht objects. It
    stores the pointer provided so that you may pass instances of this
    class directly to ctypes-bound functions. It automatically calls
    the function named in _destructor_ when the object is freed;
    this defaults to self._close."""
    _destructor_ = '_close'
    def __init__(self, ptr, owner=None):
        self._as_parameter_ = ptr
        self._owner = owner
    def __del__(self):
        if self._owner is None:
            destructor = getattr(self, self._destructor_, None)
            if destructor: destructor(self)
    def __repr__(self):
        return "<%s.%s : 0x%x>" % (__name__, self.__class__.__name__, self._as_parameter_)
    def __eq__(self, other):
        if isinstance(other, self.__class__):
            return self._as_parameter_ == other._as_parameter_
        else:
            return False
    def __ne__(self, other):
        return not self.__eq__(other)
    def __hash__(self):
        if hasattr(self._as_parameter_, 'value'):
            return hash(self._as_parameter_.value)
        else:
            return hash(self._as_parameter_)
    def __nonzero__(self):
        return bool(self._as_parameter_)

class VahtCountedObject(VahtMetaclassObject):
    """This is a superclass for all reference-counted libvaht
    objects. It behaves like VahtObject, except it will ref an
    object on creation if an extra init parameter is passed as
    True. The functions named in _managers_ will be called for ref
    and unref; this defaults to _managers_ = ('_grab', '_close'),
    calling self._grab and self._close."""
    _managers_ = ('_grab', '_close')
    def __init__(self, ptr, unowned=False):
        self._as_parameter_ = ptr
        if unowned:
            ref = getattr(self, self._managers_[0], None)
            if ref: ref(self)
    def __del__(self):
        unref = getattr(self, self._managers_[1], None)
        if unref: unref(self)
    def __repr__(self):
        return "<%s.%s : 0x%x>" % (__name__, self.__class__.__name__, self._as_parameter_)
    def __eq__(self, other):
        if isinstance(other, self.__class__):
            return self._as_parameter_ == other._as_parameter_
        else:
            return False
    def __ne__(self, other):
        return not self.__eq__(other)
    def __hash__(self):
        if hasattr(self._as_parameter_, 'value'):
            return hash(self._as_parameter_.value)
        else:
            return hash(self._as_parameter_)
    def __nonzero__(self):
        return bool(self._as_parameter_)

##
## vaht_archive.h
##

class Archive(VahtCountedObject):
    class Methods:
        open = (c_void_p, [c_char_p])
        close = (c_uint16, [c_void_p])
        grab = (c_uint16, [c_void_p])
        get_resource_types = (c_uint16, [c_void_p])
        get_resource_type = (c_char_p, [c_void_p, c_uint16])
    
    @classmethod
    def open(cls, filename):
        obj = cls._open(filename)
        if not obj:
            raise IOError("could not open archive")
        return cls(obj)
    
    @property
    def resource_types(self):
        ret = []
        for i in range(self._get_resource_types(self)):
            ret.append(self._get_resource_type(self, i))
        return ret
    
    def open_resource_raw(self, type, id):
        return Resource.open(self, type, id)
    
    def open_resource(self, type, id):
        r = self.open_resource_raw(type, id)
        
        if r.type == 'tBMP':
            return BMP.open(r)
        elif r.type == 'tMOV':
            return MOV.open(r)
        elif r.type == 'tWAV':
            return WAV.open(r)
        elif r.type == 'NAME':
            return NAME.open(r)
        elif r.type == 'CARD':
            return CARD.open(r)
        elif r.type == 'PLST':
            return PLST.open(r)
        elif r.type == 'BLST':
            return BLST.open(r)
        elif r.type == 'HSPT':
            return HSPT.open(r)
        elif r.type == 'RMAP':
            return RMAP.open(r)
        elif r.type == 'SLST':
            return SLST.open(r)
        
        return r

##
## vaht_resource.h
##

class Resource(VahtCountedObject):
    class Methods:
        open = (c_void_p, [c_void_p, c_char_p, c_uint16])
        close = (c_uint16, [c_void_p])
        grab = (c_uint16, [c_void_p])
        name = (c_char_p, [c_void_p])
        type = (c_char_p, [c_void_p])
        id = (c_uint16, [c_void_p])
        size = (c_uint32, [c_void_p])
        read = (c_uint32, [c_void_p, c_uint32, c_void_p])
        seek = (None, [c_void_p, c_uint32])
        tell = (c_uint32, [c_void_p])
    
    @classmethod
    def open(cls, archive, type, id):
        obj = cls._open(archive, type, id)
        if not obj:
            raise IOError("could not open resource")
        return cls(obj)
    
    @property
    def name(self):
        return self._name(self)
    
    @property
    def type(self):
        return self._type(self)
    
    @property
    def id(self):
        return self._id(self)
    
    @property
    def size(self):
        return self._size(self)
    
    def read(self, size):
        buf = ctypes.create_string_buffer(size)
        read = self._read(self, size, buf)
        return buf.raw[:read]
    
    def seek(self, seek):
        self._seek(self, seek)
    
    def tell(self):
        return self._tell(self)

##
## vaht_bmp.h
##

class BMP(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        width = (c_uint16, [c_void_p])
        height = (c_uint16, [c_void_p])
        data = (c_void_p, [c_void_p])
        compressed = (c_uint8, [c_void_p])
        truecolor = (c_uint8, [c_void_p])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open tBMP")
        return cls(obj)
    
    @property
    def width(self):
        return self._width(self)
    
    @property
    def height(self):
        return self._height(self)
    
    @property
    def data(self):
        ptr = self._data(self)
        return ctypes.string_at(ptr, self.width * self.height * 3)
    
    @property
    def compressed(self):
        return bool(self._compressed(self))
    
    @property
    def truecolor(self):
        return bool(self._truecolor(self))

##
## vaht_mov.h
##

class MOV(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        read = (c_uint32, [c_void_p, c_uint32, c_void_p])
        seek = (None, [c_void_p, c_uint32])
        tell = (c_uint32, [c_void_p])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open tMOV")
        return cls(obj)
    
    def read(self, size):
        buf = ctypes.create_string_buffer(size)
        read = self._read(self, size, buf)
        return buf.raw[:read]
    
    def seek(self, seek):
        self._seek(self, seek)
    
    def tell(self):
        return self._tell(self)

##
## vaht_wav.h
##

tWAV_UNKNOWN, tWAV_PCM, tWAV_ADPCM, tWAV_MP2 = range(4)

class WAV(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        samplerate = (c_uint16, [c_void_p])
        samplecount = (c_uint32, [c_void_p])
        samplesize = (c_uint8, [c_void_p])
        channels = (c_uint8, [c_void_p])
        encoding = (c_int, [c_void_p])
        read = (c_uint32, [c_void_p, c_uint32, c_void_p])
        reset = (None, [c_void_p])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open tWAV")
        return cls(obj)
    
    @property
    def samplerate(self):
        return self._samplerate(self)
    
    @property
    def samplecount(self):
        return self._samplecount(self)
    
    @property
    def samplesize(self):
        return self._samplesize(self)
    
    @property
    def channels(self):
        return self._channels(self)
    
    @property
    def encoding(self):
        return self._encoding(self)
    
    def read(self, size):
        buf = ctypes.create_string_buffer(size)
        read = self._read(self, size, buf)
        return buf.raw[:read]
    
    def reset(self):
        self._reset(self)

##
## vaht_name.h
##

class NAME(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        count = (c_uint16, [c_void_p])
        get = (c_void_p, [c_void_p, c_uint16])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open NAME")
        return cls(obj)
    
    @property
    def count(self):
        return self._count(self)
    
    def get(self, i):
        cstr = self._get(self, i)
        if not cstr:
            raise IndexError("invalid index")
        s = ctypes.string_at(cstr)
        free(cstr)
        return s
    
    @property
    def names(self):
        ret = []
        for i in range(self.count):
            ret.append(self.get(i))
        return ret

##
## vaht_card.h
##

class CARD(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        name_record = (c_int16, [c_void_p])
        name = (c_char_p, [c_void_p])
        zip_mode = (c_uint16, [c_void_p])
        script = (c_void_p, [c_void_p])
        plst_open = (c_void_p, [c_void_p])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open CARD")
        return cls(obj)
    
    @property
    def name_record(self):
        return self._name_record(self)
    
    @property
    def name(self):
        return self._name(self)
    
    @property
    def zip_mode(self):
        return bool(self._zip_mode(self))
    
    @property
    def script(self):
        return Script(self._script(self), owner=self)
    
    def plst_open(self):
        return PLST(self._plst_open(self))

##
## vaht_plst.h
##

class PLST(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        records = (c_uint16, [c_void_p])
        bitmap_id = (c_int32, [c_void_p, c_uint16])
        bitmap_open = (c_void_p, [c_void_p, c_uint16])
        rect = (None, [c_void_p, c_uint16,
                       c_void_p, c_void_p, c_void_p, c_void_p])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open PLST")
        return cls(obj)
    
    @property
    def records(self):
        return self._records(self)
    
    def bitmap_id(self, i):
        id = self._bitmap_id(self, i)
        if id < 0:
            raise IndexError("invalid index")
        return id
    
    def bitmap_open(self, i):
        ptr = self._bitmap_open(self, i)
        if not ptr:
            raise IndexError("invalid index")
        return BMP(ptr)
    
    def rect(self, i):
        left = c_uint16()
        right = c_uint16()
        top = c_uint16()
        bottom = c_uint16()
        self._rect(self, i, ctypes.byref(left), ctypes.byref(right),
                   ctypes.byref(top), ctypes.byref(bottom))
        return (left.value, right.value, top.value, bottom.value)

##
## vaht_script.h
##

EVENT_MOUSE_DOWN, EVENT_MOUSE_STILL_DOWN, EVENT_MOUSE_UP, EVENT_MOUSE_ENTER, EVENT_MOUSE_WITHIN, EVENT_MOUSE_LEAVE, EVENT_LOAD_CARD, EVENT_CLOSE_CARD, _, EVENT_OPEN_CARD, EVENT_DISPLAY_UPDATE, EVENT_COUNT = range(12)

class Script(VahtObject):
    _destructor_ = "_free"
    class Methods:
        read = (c_void_p, [c_void_p])
        free = (None, [c_void_p])
        handler = (POINTER(c_void_p), [c_void_p, c_int])
    
    @classmethod
    def read(cls, resource):
        obj = cls._read(resource)
        if not obj:
            raise RuntimeError("could not read script")
        return cls(obj)
    
    def handler(self, event):
        ptr = self._handler(self, event)
        return Command._from_ptr_array(ptr, owner=self)

class Command(VahtObject):
    _destructor_ = None
    class Methods:
        branch = (c_uint8, [c_void_p])
        code = (c_uint16, [c_void_p])
        argument_count = (c_uint16, [c_void_p])
        argument = (c_uint16, [c_void_p, c_uint16])
        branch_variable = (c_uint16, [c_void_p])
        branch_count = (c_uint16, [c_void_p])
        branch_value = (c_uint16, [c_void_p, c_uint16])
        branch_body = (POINTER(c_void_p), [c_void_p, c_uint16])
    
    @classmethod
    def _from_ptr_array(cls, ptr, owner=None):
        def gen():
            i = 0
            while ptr[i]:
                yield cls(ptr[i], owner=owner)
                i += 1
        if not ptr:
            return []
        return list(gen())
    
    @property
    def branch(self):
        return bool(self._branch(self))
    
    @property
    def code(self):
        return self._code(self)
    
    @property
    def arguments(self):
        count = self._argument_count(self)
        return [self._argument(self, i) for i in range(count)]
    
    @property
    def branch_variable(self):
        return self._branch_variable(self)
    
    @property
    def branch_values(self):
        count = self._branch_count(self)
        return [self._branch_value(self, i) for i in range(count)]

    @property
    def branch_bodies(self):
        count = self._branch_count(self)
        return [self._from_ptr_array(self._branch_body(self, i), owner=self)
                for i in range(count)]

##
## vaht_blst.h
##

class BLST(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        records = (c_uint16, [c_void_p])
        enabled = (c_uint16, [c_void_p, c_uint16])
        hotspot_id = (c_int32, [c_void_p, c_uint16])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open BLST")
        return cls(obj)
    
    @property
    def records(self):
        return self._records(self)
    
    def enabled(self, i):
        return bool(self._enabled(self, i))
    
    def hotspot_id(self, i):
        id = self._hotspot_id(self, i)
        if id < 0:
            raise IndexError("invalid index")
        return id

##
## vaht_hspt.h
##

class HSPT(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        records = (c_uint16, [c_void_p])
        blst_id = (c_uint16, [c_void_p, c_uint16])
        name_record = (c_int16, [c_void_p, c_uint16])
        name = (c_char_p, [c_void_p, c_uint16])
        rect = (None, [c_void_p, c_uint16,
                       c_void_p, c_void_p, c_void_p, c_void_p])
        cursor = (c_uint16, [c_void_p, c_uint16])
        zip_mode = (c_uint16, [c_void_p, c_uint16])
        script = (c_void_p, [c_void_p, c_uint16])

    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open HSPT")
        return cls(obj)
    
    @property
    def records(self):
        return self._records(self)
    
    def blst_id(self, i):
        return self._blst_id(self, i)
    
    def name_record(self, i):
        return self._name_record(self, i)
    
    def name(self, i):
        return self._name(self, i)
    
    def rect(self, i):
        left = c_int16()
        right = c_int16()
        top = c_int16()
        bottom = c_int16()
        self._rect(self, i, ctypes.byref(left), ctypes.byref(right),
                   ctypes.byref(top), ctypes.byref(bottom))
        return (left.value, right.value, top.value, bottom.value)
    
    def cursor(self, i):
        return self._cursor(self, i)
    
    def zip_mode(self, i):
        return bool(self._zip_mode(self, i))
    
    def script(self, i):
        return Script(self._script(self, i), owner=self)

##
## vaht_rmap.h
##

class RMAP(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        count = (c_uint16, [c_void_p])
        get = (c_uint32, [c_void_p, c_uint16])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open RMAP")
        return cls(obj)
    
    @property
    def count(self):
        return self._count(self)
    
    def get(self, i):
        return self._get(self, i)
    
    @property
    def codes(self):
        ret = []
        for i in range(self.count):
            ret.append(self.get(i))
        return ret

##
## vaht_slst.h
##

SLST_NO_FADE, SLST_FADE_OUT, SLST_FADE_IN, SLST_FADE_IN_OUT = range(4)

class SLST(VahtObject):
    class Methods:
        open = (c_void_p, [c_void_p])
        close = (None, [c_void_p])
        records = (c_uint16, [c_void_p])
        count = (c_uint16, [c_void_p, c_uint16])
        sound_id = (c_uint16, [c_void_p, c_uint16, c_uint16])
        fade = (c_uint16, [c_void_p, c_uint16])
        loop = (c_uint16, [c_void_p, c_uint16])
        global_volume = (c_uint16, [c_void_p, c_uint16])
        volume = (c_uint16, [c_void_p, c_uint16, c_uint16])
        balance = (c_uint16, [c_void_p, c_uint16, c_uint16])
    
    @classmethod
    def open(cls, resource):
        obj = cls._open(resource)
        if not obj:
            raise RuntimeError("could not open SLST")
        return cls(obj)
    
    @property
    def records(self):
        return self._records(self)
    
    def count(self, i):
        return self._count(self, i)

    def sound_id(self, i, j):
        return self._sound_id(self, i, j)

    def fade(self, i):
        return self._fade(self, i)

    def loop(self, i):
        return bool(self._loop(self, i))

    def global_volume(self, i):
        return self._global_volume(self, i)

    def volume(self, i, j):
        return self._volume(self, i, j)

    def balance(self, i, j):
        return self._balance(self, i, j)
