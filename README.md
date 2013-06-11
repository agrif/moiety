Moiety
======

*an HTML5 implementation of the Riven game engine*

This does not work right now, and even when it does work it takes
effort to set up.

At the moment, you'll need:

* [libvaht](https://github.com/agrif/libvaht/)
* [flask](http://flask.pocoo.org/)
* the MHK files from the Riven 5 CD version

Edit `datadir` in *moiety.py* to point to your MHK files. You might
also want to change the `app.run()` call near the bottom.
