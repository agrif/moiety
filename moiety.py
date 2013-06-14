import os.path
import functools
import StringIO
import Image
import wave
import json

import vaht

from flask import Flask, abort, make_response, url_for
from flask import render_template
app = Flask(__name__)

datadir = "/home/agrif/local/riven"
stack_files = {
    'aspit' : ['a_Data.MHK', 'a_Sounds.MHK'],
    'bspit' : ['b_Data.MHK', 'b2_data.MHK', 'b_Sounds.MHK'],
    'gspit' : ['g_Data.MHK', 'g_Sounds.MHK'],
    'jspit' : ['j_Data1.MHK', 'j_Data2.MHK', 'j_Sounds.MHK'],
    'ospit' : ['o_Data.MHK', 'o_Sounds.MHK'],
    'pspit' : ['p_Data.MHK', 'p_Sounds.MHK'],
    'rspit' : ['r_Data.MHK', 'r_Sounds.MHK'],
    'tspit' : ['t_Data.MHK', 't_Sounds.MHK'],
}

archive_cache = {}
def get_archive(fname):
    try:
        return archive_cache[fname]
    except KeyError:
        a = vaht.Archive.open(os.path.join(datadir, fname))
        archive_cache[fname] = a
        return a

def get_resource(stack, type, id):
    try:
        files = stack_files[stack]
    except KeyError:
        raise IOError("could not load stack")
    for fname in files:
        a = get_archive(fname)
        try:
            return a.open_resource(type, id)
        except IOError:
            pass
        except RuntimeError:
            pass
    raise IOError("could not load resource")

resource_url_format = '/resources/<stack>/{type}/<int:id>'
def resource_url(resource_type):
    def _resource_url(func):
        @app.route(resource_url_format.format(type=resource_type))
        @functools.wraps(func)
        def inner(stack, id):
            try:
                r = get_resource(stack, resource_type, id)
            except IOError:
                abort(404)
            return func(r, stack, id)
        return inner
    return _resource_url

def content_type(mime):
    def _content_type(func):
        @functools.wraps(func)
        def inner(*args, **kwargs):
            resp = func(*args, **kwargs)
            resp = make_response(resp)
            resp.headers['Content-type'] = mime
            return resp
        return inner
    return _content_type

def json_view(func):
    @content_type('application/json')
    @functools.wraps(func)
    def inner(*args, **kwargs):
        resp = func(*args, **kwargs)
        return json.dumps(resp, ensure_ascii=False)
    return inner

@resource_url('tBMP')
@content_type('image/png')
def tBMP(r, stack, id):
    im = Image.fromstring('RGB', (r.width, r.height), r.data)
    buf = StringIO.StringIO()
    im.save(buf, 'png')
    return buf.getvalue()

@resource_url('tMOV')
@content_type('video/quicktime')
def tMOV(r, stack, id):
    buf = ""
    while True:
        d = r.read(4096)
        buf += d
        if not d:
            break
    return buf

@resource_url('tWAV')
@content_type('audio/wav')
def tWAV(r, stack, id):
    inbuf = ""
    while True:
        d = r.read(4096)
        inbuf += d
        if not d:
            break
    
    outbuf = StringIO.StringIO()
    wav = wave.open(outbuf, 'wb')
    wav.setnchannels(r.channels)
    wav.setsampwidth(r.samplesize / 8)
    wav.setframerate(r.samplerate)
    wav.setnframes(r.samplecount)
    wav.writeframes(inbuf)
    wav.close()
    
    return outbuf.getvalue()

@resource_url('NAME')
@json_view
def NAME(r, stack, id):
    return r.names

event_names = {
    vaht.EVENT_MOUSE_DOWN: 'mouse-down',
    vaht.EVENT_MOUSE_STILL_DOWN: 'mouse-still-down',
    vaht.EVENT_MOUSE_UP: 'mouse-up',
    vaht.EVENT_MOUSE_ENTER: 'mouse-enter',
    vaht.EVENT_MOUSE_WITHIN: 'mouse-within',
    vaht.EVENT_MOUSE_LEAVE: 'mouse-leave',
    vaht.EVENT_LOAD_CARD: 'load-card',
    vaht.EVENT_CLOSE_CARD: 'close-card',
    vaht.EVENT_OPEN_CARD: 'open-card',
    vaht.EVENT_DISPLAY_UPDATE: 'display-update',
}

command_names = {
    1: "draw-bmp",
    2: "goto-card",
    3: "inline-slst",
    4: "play-wav",
    7: "set-var",
    8: "conditional",
    9: "enable-hotspot",
    10: "disable-hotspot",
    13: "set-cursor",
    14: "pause",
    17: "call",
    18: "transition",
    19: "reload",
    20: "disable-update",
    21: "enable-update",
    24: "increment",
    27: "goto-stack",
    32: "play-foreground-movie",
    33: "play-background-movie",
    39: "activate-plst",
    40: "activate-slst",
    43: "activate-blst",
    44: "activate-flst",
    45: "zip",
    46: "activate-mlst",
}

def structure_commands(cmds):
    for cmd in cmds:
        if cmd.branch:
            name = "branch"
            variable = cmd.branch_variable
            values = cmd.branch_values
            bodies = [list(structure_commands(c)) for c in cmd.branch_bodies]
            cases = dict(zip(values, bodies))
            yield dict(name=name, variable=variable, cases=cases)
        else:
            code = cmd.code
            name = command_names.get(code, code)
            yield dict(name=name, arguments=cmd.arguments)

def structure_script(script):
    r = {}
    for event in range(vaht.EVENT_COUNT):
        cmds = script.handler(event)
        if cmds:
            r[event_names.get(event, event)] = list(structure_commands(cmds))
    return r

@resource_url('CARD')
@json_view
def CARD(r, stack, id):
    resp = {}
    resp['name'] = r.name_record
    resp['zip_mode'] = r.zip_mode
    resp['script'] = structure_script(r.script)
    return resp

@resource_url('PLST')
@json_view
def PLST(r, stack, id):
    # PLST records start at 1, so use a dummy object for 0
    resp = [{}]
    for i in range(1, r.records + 1):
        left, right, top, bottom = r.rect(i)
        bitmap_id = r.bitmap_id(i)
        obj = dict(left=left, right=right, top=top, bottom=bottom)
        obj['bitmap'] = bitmap_id
        resp.append(obj)
    return resp

@resource_url('BLST')
@json_view
def BLST(r, stack, id):
    # BLST records start at 1, so use a dummy object for 0
    resp = [{}]
    for i in range(1, r.records + 1):
        enabled = r.enabled(i)
        hotspot_id = r.hotspot_id(i)
        obj = dict(enabled=enabled, hotspot_id=hotspot_id)
        resp.append(obj)
    return resp

@resource_url('HSPT')
@json_view
def HSPT(r, stack, id):
    # HSPT records start at 1, so use a dummy object for 0
    resp = [{}]
    for i in range(1, r.records + 1):
        blst_id = r.blst_id(i)
        name_record = r.name_record(i)
        left, right, top, bottom = r.rect(i)
        cursor = r.cursor(i)
        zip_mode = r.zip_mode(i)
        script = r.script(i)
        obj = dict(left=left, right=right, top=top, bottom=bottom)
        obj['blst_id'] = blst_id
        obj['name'] = name_record
        obj['cursor'] = cursor
        obj['zip_mode'] = zip_mode
        obj['script'] = structure_script(script)
        resp.append(obj)
    return resp

@app.route("/")
def main():
    return render_template('index.html')

if __name__ == '__main__':
    app.run(debug=True, host="::", processes=3)
