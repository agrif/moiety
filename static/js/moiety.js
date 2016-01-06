var state = {
    // stuff from setup()
    ctx: null,
    canvas: null,
    offscreenCtx: null,
    offscreen: null,
    
    // state (in the large)
    stackname: null,
    cardid: null,
    
    // stack stuff
    cardNames: null,
    hotspotNames: null,
    commandNames: null,
    variableNames: null,
    stackNames: null,
    
    // card stuff
    card: null,
    plst: null,
    blst: null,
    hspt: null,
    slst: null,

    // playing background sounds
    bgSounds: {},
    
    // variable storage
    variables: {
        // initial state of the telescope
        ttelescope: 3,
        
        // initial state of the gateroom
        tgatestate: 1,
        
        // turn off the temple island dome elevator
        tdomeelev: 1,
        
        // initial state of the lake sub access points
        // (turn on first, and control tower only)
        jbridge1: 1,
        jbridge4: 1,
    },
    
    // offscreen contexts, and whether to use it
    // and transition info
    useOffscreen: false,
    transition: null,
    
    // whether to ignore mouse events
    ignoreMouse: false,
    
    // the hotspot the mouse is in
    currentHotspot: null,
    
    // the last set cursor
    cursor: 3000,

    setup: function(canvas) {
        state.canvas = canvas[0];
        state.ctx = state.canvas.getContext('2d');
        
        state.offscreen = document.createElement('canvas');
        state.offscreen.width = state.canvas.width;
        state.offscreen.height = state.canvas.height;
        state.offscreenCtx = state.offscreen.getContext('2d');
        
        canvas.mousemove(state.onMouseMove);
        canvas.mousedown(state.onMouseDown);
        canvas.mouseup(state.onMouseUp);
    },

    // helper to fire a change event
    change: function(name) {
        state.trigger('change:' + name, state);
    },

    onMouseMove: function(e) {
        if (state.ignoreMouse)
            return;
        
        var x = e.offsetX;
        var y = e.offsetY;
        var hotspot = state.getHotspot(x, y);
        if (hotspot != state.currentHotspot) {
            if (state.currentHotspot) {
                // leaving state.currentHotspot
                state.setCursor(null);
                state.runScriptHandler(state.currentHotspot.script, "mouse-leave");
            }
            if (hotspot) {
                // entering hotspot
                state.setCursor(hotspot.cursor);
                state.runScriptHandler(hotspot.script, "mouse-enter");
            }
            state.currentHotspot = hotspot;
        }
        
        if (hotspot) {
            state.runScriptHandler(hotspot.script, "mouse-within");
        }
    },
    
    onMouseDown: function(e) {
        if (state.ignoreMouse)
            return;
        
        var x = e.offsetX;
        var y = e.offsetY;
        var hotspot = state.getHotspot(x, y);
        if (hotspot) {
            // hide mouse and ignore it until script is done
            var savedCursor = state.cursor;
            
            // workaround for webkit
            var t = jQuery.Deferred();
            setTimeout(function() {
                state.ignoreMouse = true;
                state.setCursor(9000);
                t.resolve();
            }, 100);
            
            var p = state.runScriptHandler(hotspot.script, "mouse-down");
            jQuery.when(p, t).done(function() {
                state.setCursor(savedCursor);
                state.ignoreMouse = false;
            });
        }
    },
    
    onMouseUp: function(e) {
        if (state.ignoreMouse)
            return;
        
        var x = e.offsetX;
        var y = e.offsetY;
        var hotspot = state.getHotspot(x, y);
        if (hotspot) {
            state.runScriptHandler(hotspot.script, "mouse-up");         
        }
    },

    changeStack: function(stackname) {
        if (stackname == state.stackname)
            return jQuery.Deferred().resolve();
        
        // load up global stack resources
        var stat = log.status("changing to stack " + stackname);
        var pCards = loadResource(stackname, 'NAME', 1);
        var pHotspots = loadResource(stackname, 'NAME', 2);
        var pCommands = loadResource(stackname, 'NAME', 3);
        var pVariables = loadResource(stackname, 'NAME', 4);
        var pStacks = loadResource(stackname, 'NAME', 5);
        var d = jQuery.when(pCards, pHotspots, pCommands, pVariables, pStacks)
        d.done(function(cards, hotspots, commands, variables, stacks) {
            state.stackname = stackname;
            state.cardid = null;
            state.cardNames = cards[0];
            state.hotspotNames = hotspots[0];
            state.commandNames = commands[0];
            state.variableNames = variables[0];
            state.stackNames = stacks[0];
            jQuery.each(state.bgSounds, function(i, v) {
                v.pause();
            });
            state.bgSounds = {};
            state.change('stackname');
            state.change('cardid');
            state.change('bgSounds')
            stat.resolve();
        }).fail(stat.reject);
        
        return stat;
    },

    gotoCard: function(stackname, cardid) {
        // currently gotoCard is used for reload, so *don't do this!*
        //if (stackname == state.stackname && cardid == state.cardid)
        //  return jQuery.Deferred().resolve();
        
        // disable updates
        state.disableScreenUpdate();
        
        // unload current card
        var unload;
        if (state.cardid) {
            unload = state.runScriptHandler(state.card.script, 'close-card');
        } else {
            unload = jQuery.Deferred();
            unload.resolve();
        }
        
        var d = jQuery.Deferred();
        unload.fail(d.reject).done(function() {
            state.cardid = null;
            
            // change stacks
            state.changeStack(stackname).done(function() {
                // load new card
                log.status("moving to card " + cardid, d);
                var pCard = loadResource(stackname, 'CARD', cardid);
                pCard.fail(d.reject).done(function(card) {
                    // set variables
                    state.cardid = cardid;
                    state.currentHotspot = null;
                    state.setCursor(null);
                    state.card = card.card;
                    state.plst = card.plst;
                    state.blst = card.blst;
                    state.hspt = card.hspt;
                    state.slst = card.slst;

                    state.change('cardid');
                    rtr.navigate(stackname + '/' + cardid, {replace: true});
                    
                    // set up plst state
                    jQuery.each(state.plst, function(index, p) {
                        p.enabled = false;
                    });
                    
                    // set up button state
                    var blst_ids = [];
                    jQuery.each(state.blst, function(index, b) {
                        if (index == 0)
                            return;
                        if (jQuery.inArray(b.hotspot_id, blst_ids) == -1)
                            blst_ids.push(b.hotspot_id);
                    });
                    jQuery.each(state.hspt, function(index, h) {
                        if (jQuery.inArray(h.blst_id, blst_ids) == -1) {
                            if (!h.zip_mode)
                                h.enabled = true;
                        } else {
                            h.enabled = false;
                        }
                    });
                                        
                    // activate plst 1, slst 1 by default
                    var pDef = jQuery.when(state.activatePLST(1), state.activateSLST(1));
                    pDef.fail(d.reject).done(function() {
                        // run load-card
                        var lc = state.runScriptHandler(state.card.script, "load-card");                        
                        lc.fail(d.reject).done(function() {
                            // enable updates again
                            state.enableScreenUpdate().fail(d.reject).done(function() {
                                // run open-card
                                var oc = state.runScriptHandler(state.card.script, "open-card");
                                oc.fail(d.reject).done(d.resolve);

                            });
                        });
                    });
                });
            });
        });
        
        return d;
    },
    
    runScriptHandler: function(script, handler) {
        var deferred = jQuery.Deferred();
        if (handler in script) {
            state.runScript(script[handler], 0, deferred);
        } else {
            deferred.resolve();
        }
        return deferred.promise();
    },
    
    runScript: function(commands, index, deferred) {
        if (index >= commands.length) {
            deferred.resolve();
            return;
        }
        
        var cmd = commands[index];
        if (cmd.name == "branch") {
            var varname = state.variableNames[cmd.variable];
            var value = state.getVariable(varname);
            var branch = [];
            if (value in cmd.cases) {
                branch = cmd.cases[value];
            } else if (0xffff in cmd.cases) {
                branch = cmd.cases[0xffff];
            }
            
            var branchend = jQuery.Deferred();
            state.runScript(branch, 0, branchend);
            branchend.fail(deferred.reject).done(function() {
                state.runScript(commands, index + 1, deferred);
            });
        } else {
            var p = null;
            if (cmd.name in scriptCommands) {
                p = scriptCommands[cmd.name].apply(scriptCommands, cmd.arguments);
            } else {
                log.message("!!! (stub) " + cmd.name + " " + cmd.arguments.toString());
            }
            
            if (p) {
                p.fail(deferred.reject).done(function() {
                    state.runScript(commands, index + 1, deferred);
                });
            } else {
                state.runScript(commands, index + 1, deferred);
            }
        }
    },
    
    getVariable: function(name) {
        if (name in state.variables) {
            log.message("reading " + name + " = " + state.variables[name]);
            return state.variables[name];
        }
        log.message("reading " + name + " = 0 (default)");
        return 0;
    },
    
    setVariable: function(name, value) {
        log.message("setting " + name + " = " + value);
        state.variables[name] = value;
        state.change('variables');
    },
    
    flip: function() {
        // move offscreen to ctx, with transition if needed
        var d = jQuery.Deferred();
        
        if (state.transition == null) {
            // simple
            state.ctx.drawImage(state.offscreen, 0, 0);
            return d.resolve();
        }

        // transition
        var width = state.canvas.width;
        var height = state.canvas.height;
        var first = state.ctx.getImageData(0, 0, width, height);
        var second = state.offscreen;
        
        var start = {
            firstX: 0,
            firstY: 0,
            firstDirtyX: 0,
            firstDirtyY: 0,
            firstDirtyWidth: width,
            firstDirtyHeight: height,
            
            secondX: 0,
            secondY: 0,
            secondSX: 0,
            secondSY: 0,
            secondSWidth: width,
            secondSHeight: height,
            secondAlpha: 1.0,
        };
        
        var end = jQuery.extend(true, {}, start);
        var firstOnTop = false;
        
        function posForDirection(dir, invert) {
            if (invert) {
                dir = dir ^ 0x1;
            }
            switch (dir) {
            case 0: return {x: width, y: 0};
            case 1: return {x: -width, y: 0};
            case 2: return {x: 0, y: height};
            case 3: return {x: 0, y: -height};
            }
        }
        
        // set up start/end based on state.transition
        if (state.transition < 16) {
            // directional!
            var direction = state.transition & 0x3;
            var secondMoves = state.transition & 0x4;
            var firstMoves = state.transition & 0x8;
            if (!secondMoves)
                firstOnTop = true;
            
            if (!firstMoves && !secondMoves) {
                // wipe!
                log.message("!!! (stub wipe transition) " + state.transition);
            } else {
                // individually slide around!
                if (firstMoves) {
                    var firstPos = posForDirection(direction, true);
                    end.firstX = firstPos.x;
                    end.firstY = firstPos.y;
                }
                if (secondMoves) {
                    var secondPos = posForDirection(direction, false);
                    start.secondX = secondPos.x;
                    start.secondY = secondPos.y;
                }
            }
        } else {
            // fade!
            start.secondAlpha = 0.0;
        }
        state.transition = null;
        
        function setprops(props) {
            if (!firstOnTop)
                state.ctx.putImageData(first, props.firstX, props.firstY, props.firstDirtyX, props.firstDirtyY, props.firstDirtyWidth, props.firstDirtyHeight);
            state.ctx.globalAlpha = props.secondAlpha;
            state.ctx.drawImage(second, props.secondSX, props.secondSY, props.secondSWidth, props.secondSHeight, props.secondX, props.secondY, props.secondSWidth, props.secondSHeight);
            state.ctx.globalAlpha = 1.0;
            if (firstOnTop)
                state.ctx.putImageData(first, props.firstX, props.firstY, props.firstDirtyX, props.firstDirtyY, props.firstDirtyWidth, props.firstDirtyHeight);
        }
        
        $(start).animate(end, {
            duration: 250,
            easing: 'linear',
            step: function() {
                setprops(this);
            },
            complete: function() {
                // force the endstate to be second, always
                state.ctx.drawImage(second, 0, 0);
                d.resolve();
            }
        });
        return d.promise();
    },
    
    scheduleTransition: function(transition) {
        state.transition = transition;
    },
    
    disableScreenUpdate: function() {
        state.offscreenCtx.drawImage(state.canvas, 0, 0);
        state.useOffscreen = true;
    },
    
    enableScreenUpdate: function() {
        var d = state.runScriptHandler(state.card.script, 'display-update');
        return d.then(function() {
            state.useOffscreen = false;
            return state.flip();            
        });
    },
    
    draw: function(drawfun) {
        // screen update disabled
        if (state.useOffscreen) {
            drawfun(state.offscreenCtx);
            return jQuery.Deferred().resolve();
        }
        
        // scheduled transition, screen update enabled
        if (state.transition != null) {
            state.disableScreenUpdate();
            drawfun(state.offscreenCtx);
            return state.enableScreenUpdate();
        }
        
        // no transition, screen update enabled
        drawfun(state.ctx);
        return jQuery.Deferred().resolve();
    },
    
    setCursor: function(cursor) {
        if (!cursor)
            cursor = 3000;
        var url = "static/cursors/" + cursor + ".png";
        state.canvas.style.cursor = "url(" + url + "), auto";
        state.cursor = cursor;
    },
    
    activateBLST: function(i) {
        var record = state.blst[i];
        jQuery.each(state.hspt, function(index, h) {
            if (h.blst_id == record.hotspot_id) {
                if (!h.zip_mode)
                    h.enabled = record.enabled;
            }
        });
    },
    
    activatePLST: function(i) {
        var record = state.plst[i];
        if (record.enabled) {
            return jQuery.Deferred().resolve();
        }
        
        return loadResource(state.stackname, 'tBMP', record.bitmap).then(function(img) {
            record.enabled = true;
            return state.draw(function(ctx) {
                ctx.drawImage(img, record.left, record.top, record.right-record.left, record.bottom-record.top);
            });
        });
    },
    
    activateSLST: function(i) {
        var fadeTime = 3000;
        var record = state.slst[i];
        if (record == undefined)
            record = {volume: 0, loop: false, fade: 'out', sounds: []};
        
        var pSounds = jQuery.map(record.sounds, function(s) {
            return loadResource(state.stackname, 'tWAV', s.sound_id).then(function(wav) {
                return {sound: wav, volume: s.volume, balance: s.balance, sound_id: s.sound_id};
            });
        });
        var allSounds = jQuery.when.apply(jQuery, pSounds);
        return allSounds.then(function() {
            var fade = record.fade;
            var added = {};
            jQuery.each(arguments, function(i, s) {
                var volume = (record.volume / 255) * (s.volume / 255);
                var balance = s.balance / 128; // these are both guesses
                if (volume > 1.0)
                    volume = 1.0;
                if (volume < 0.0)
                    volume = 0.0;
                var loop = record.loop;
                var sound = s.sound;
                if (s.sound_id in state.bgSounds) {
                    sound = state.bgSounds[s.sound_id];
                }
                
                if (volume > 0) {
                    if (!(s.sound_id in state.bgSounds)) {
                        sound.volume = 0;
                        sound.play();
                        state.bgSounds[s.sound_id] = sound;
                    }
                    $(sound).stop();
                    if (fade == "in" || fade == "inout") {
                        $(sound).animate({volume: volume}, fadeTime);
                    } else {
                        sound.volume = volume;
                    }
                    sound.loop = loop; // FIXME balance
                    added[s.sound_id] = sound;
                }
            });
            
            jQuery.each(state.bgSounds, function(i, s) {
                if (!(i in added)) {
                    $(s).stop();
                    if (fade == "out" || fade == "inout") {
                        $(s).animate({volume: 0}, fadeTime);
                        setTimeout(function() {
                            if (!(i in state.bgSounds)) {
                                s.pause();
                            }
                        }, fadeTime);
                    } else {
                        s.pause();
                    }
                    delete state.bgSounds[i];
                }
            });
        });
    },
    
    getHotspot: function(x, y) {
        var ret = null;
        if (state.hspt) {
            jQuery.each(state.hspt, function(i, h) {
                if (h.enabled &&
                    h.left <= x && x < h.right &&
                    h.top <= y && y < h.bottom) {
                    ret = h;
                }
            });
        }
        return ret;
    },
    
    playSound: function(resource) {
        var d = jQuery.Deferred();
        resource.play();
        $(resource).on("ended", function() {
            d.resolve();
        });
        return d.promise();
    }
};

_.extend(state, Backbone.Events);

var Router = Backbone.Router.extend({
    routes: {
        ":stackname/:cardid": "gotoCard"
    },
    
    gotoCard: function(stackname, cardid) {
        state.gotoCard(stackname, cardid);
    }
});

var rtr;
$(function() {
    state.setup($('#canvas'));
    rtr = new Router;
    if (!Backbone.history.start())
        state.gotoCard('aspit', 1);
});
