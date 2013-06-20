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
	
	// variable storage
	variables: {
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
		canvas.mousedown(state.onMouseUp);
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
			state.ignoreMouse = true;
			
			// workaround for webkit
			var t = setTimeout(function() {
				state.setCursor(9000);				
			}, 100);
			
			var p = state.runScriptHandler(hotspot.script, "mouse-down");
			jQuery.when(p, t).done(function() {
				setTimeout(function() {
					state.setCursor(savedCursor);
				}, 100);
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
		var stat = console.status("changing to stack " + stackname);
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
			stat.resolve();
		}).fail(stat.reject);
		
		return stat;
	},

	gotoCard: function(stackname, cardid) {
		// currently gotoCard is used for reload, so *don't do this!*
		//if (stackname == state.stackname && cardid == state.cardid)
		//	return jQuery.Deferred().resolve();
		
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
				console.status("moving to card " + cardid, d);
				var pCard = loadResource(stackname, 'CARD', cardid);
				var pPLST = loadResource(stackname, 'PLST', cardid);
				var pBLST = loadResource(stackname, 'BLST', cardid);
				var pHSPT = loadResource(stackname, 'HSPT', cardid);
				var when = jQuery.when(pCard, pPLST, pBLST, pHSPT);
				when.fail(d.reject).done(function(card, plst, blst, hspt) {
					// set variables
					state.cardid = cardid;
					state.currentHotspot = null;
					state.setCursor(null);
					state.card = card[0];
					state.plst = plst[0];
					state.blst = blst[0];
					state.hspt = hspt[0];
					
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
					
					// activate plst 1 by default
					state.activatePLST(1).fail(d.reject).done(function() {
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
				console.message("!!! (stub) " + cmd.name + " " + cmd.arguments.toString());
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
			console.message("reading " + name + " = " + state.variables[name]);
			return state.variables[name];
		}
		console.message("reading " + name + " = 0 (default)");
		return 0;
	},
	
	setVariable: function(name, value) {
		console.message("setting " + name + " = " + value);
		state.variables[name] = value;
	},
	
	flip: function() {
		// move offscreen to ctx, with transition if needed
		var d = jQuery.Deferred();
		
		if (state.transition != null) {
			console.message("!!! (stub transition) " + state.transition);
			state.transition = null;
		}
		
		state.ctx.drawImage(state.offscreen, 0, 0);
		
		return d.resolve();
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
		var url = "/static/cursors/" + cursor + ".png";
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
	
	getHotspot: function(x, y) {
		var ret = null;
		jQuery.each(state.hspt, function(i, h) {
			if (h.enabled &&
				h.left <= x && x < h.right &&
				h.top <= y && y < h.bottom) {
				ret = h;
			}
		});
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

function loadResource(stack, type, id) {
	var p;
	var url = "/resources/" + stack + "/" + type + "/" + id
	
	switch (type) {
	case 'tBMP':
		var d = new jQuery.Deferred();
		var img = new Image();
		img.src = url;
		img.onload = function() {
			d.resolve(img);
		};
		img.onerror = function() {
			d.reject();
		};
		p = d.promise();
		break;
	case 'tWAV':
		var d = new jQuery.Deferred();
		var snd = new Audio();
		snd.src = url;
		// huh, onloadeddata doesn't work on chrome
		$(snd).on("canplay", function() {
			d.resolve(snd);
		});
		$(snd).on("error", function() {
			d.reject();
		});
		snd.load();
		p = d.promise();
		break;
	default:
		p = jQuery.getJSON(url);
	}
	
	console.status('loading <a href="' + url + '">' + url + '</a>', p);
	
	return p;
}

$(function() {
	state.setup($('#canvas'));
	state.gotoCard('aspit', 1);
});