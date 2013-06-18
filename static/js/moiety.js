var console;
var state;

var ConsoleView = Backbone.View.extend({
	initialize: function() {
		var viewthis = this;
		this.console = this.$el.console({
			promptLabel: 'moiety> ',
			commandValidate: function(line) {
				if (line == "")
					return false;
				return true;
			},
			commandHandle: function(line, report) {
				return viewthis.handle(line, report);
			},
			animateScroll: true,
			promptHistory: true
		});
	},
	
	handle: function(line, report) {
		args = line.split(/ +/);
		function doerror(msg) {
			report([{msg: args[0] + ': ' + msg,
					 className: "jquery-console-message-error"}]);
		}
		
		if (args[0] in consoleCommands) {
			try {
				consoleCommands[args[0]].apply(consoleCommands, args.slice(1));
			} catch (err) {
				doerror(err);
				return;
			}
		} else {
			doerror("invalid command");
			return;
		}
		
		return "";
	},
	
	message: function(msg) {
		this.console.message(msg);
		this.console.scrollToBottom();
	},
	
	status: function(msg, p) {
		var mesg = $('<div/>').html(msg);
		var status = $('<div/>').addClass('status status-pending').appendTo(mesg);
		console.message(mesg);
		if (!p) {
			p = jQuery.Deferred();
		}
		
		return p.done(function() {
			status.removeClass('status-pending').addClass('status-done');
		}).fail(function() {
			status.removeClass('status-pending').addClass('status-failed');
		});	
	}
});

var consoleCommands = {
	'load': function(stack, type, id) {
		if (arguments.length != 3) throw "invalid arguments";
		loadResource(stack, type, id);
	},
	
	'goto-card': function(stackname, cardid) {
		if (arguments.length != 2) throw "invalid arguments";
		state.gotoCard(stackname, cardid);
	},
	
	'activate-plst': function(plstid) {
		if (arguments.length != 1) throw "invalid arguments";
		state.activatePLST(parseInt(plstid));
	}
};

var scriptCommands = {
	'goto-card': function(cardid) {
		state.gotoCard(state.stackname, cardid);
	},
	
	'call': function(nameid, argumentCount) {
		var name = state.commandNames[nameid];
		var args = Array(arguments).slice(2, 2 + argumentCount);
		if (name in externalCommands) {
			return externalCommands[name].apply(externalCommands, args);
		} else {
			console.message("!!! (stub call) " + name + " " + args.toString());			
		}
	}
};

var externalCommands = {
	'xasetupcomplete': function() {
		state.gotoCard("aspit", 1);
	}
};

var state = {
	// stuff from setup()
	ctx: null,
	canvas: null,
	
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
	
	// the hotspot the mouse is in
	currentHotspot: null,
	
	setup: function(canvas) {
		state.canvas = canvas[0];
		state.ctx = state.canvas.getContext("2d");
		canvas.mousemove(state.onMouseMove);
		canvas.mousedown(state.onMouseDown);
	},
	
	onMouseMove: function(e) {
		var x = e.offsetX;
		var y = e.offsetY;
		var hotspot = state.getHotspot(x, y);
		if (hotspot != state.currentHotspot) {
			if (state.currentHotspot) {
				//var name = state.hotspotNames[state.currentHotspot.name];
				//console.message("leaving hotspot " + name);
				
				state.setCursor(null);
			}
			if (hotspot) {
				//var name = state.hotspotNames[hotspot.name];
				//console.message("entering hotspot " + name);
				
				state.setCursor(hotspot.cursor);
			}
			state.currentHotspot = hotspot;
		}
	},
	
	onMouseDown: function(e) {
		var x = e.offsetX;
		var y = e.offsetY;
		var hotspot = state.getHotspot(x, y);
		if (hotspot) {
			state.runScriptHandler(hotspot.script, "mouse-down");			
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
		if (stackname == state.stackname && cardid == state.cardid)
			return jQuery.Deferred().resolve();
		
		// unload current card
		
		// change stacks
		var d = jQuery.Deferred();
		state.changeStack(stackname).done(function() {
			// load new card
			console.status("moving to card " + cardid, d);
			var pCard = loadResource(stackname, 'CARD', cardid);
			var pPLST = loadResource(stackname, 'PLST', cardid);
			var pBLST = loadResource(stackname, 'BLST', cardid);
			var pHSPT = loadResource(stackname, 'HSPT', cardid);
			var when = jQuery.when(pCard, pPLST, pBLST, pHSPT);
			when.done(function(card, plst, blst, hspt) {
				// set variables
				state.cardid = cardid;
				state.currentHotspot = null;
				state.setCursor(null);
				state.card = card[0];
				state.plst = plst[0];
				state.blst = blst[0];
				state.hspt = hspt[0];
				
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
				
				state.activatePLST(1).done(d.resolve).fail(d.reject);
			}).fail(d.reject);
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
		return deferred;
	},
	
	runScript: function(commands, index, deferred) {
		if (index >= commands.length) {
			deferred.resolve();
			return;
		}
		
		var cmd = commands[index];
		if (cmd.name == "branch") {
			// TODO
			console.message("!!! (stub branch)");
		} else {
			if (cmd.name in scriptCommands) {
				scriptCommands[cmd.name].apply(scriptCommands, cmd.arguments);
			} else {
				console.message("!!! (stub) " + cmd.name + " " + cmd.arguments.toString());
			}
			
			state.runScript(commands, index + 1, deferred);
		}
	},
	
	setCursor: function(cursor) {
		if (cursor == null) {
			state.canvas.style.cursor = "default";
		} else {
			state.canvas.style.cursor = "pointer";
		}
	},
	
	activatePLST: function(i) {
		var record = state.plst[i];
		return loadResource(state.stackname, 'tBMP', record.bitmap).done(function(img) {
			state.ctx.drawImage(img, record.left, record.top, record.right-record.left, record.bottom-record.top);
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
	}
};

function loadResource(stack, type, id) {
	var p;
	var url = "/resources/" + stack + "/" + type + "/" + id
	
	switch (type) {
	case 'tBMP':
		var d = new jQuery.Deferred();
		var img = new Image();
		img.src = url
		img.onload = function() {
			d.resolve(img);
		};
		img.onerror = function() {
			d.reject();
		};
		p = d.promise();
		break;
	default:
		p = jQuery.getJSON(url);
	}
	
	console.status('loading <a href="' + url + '">' + url + '</a>', p);
	
	return p;
}

$(function() {
	console = new ConsoleView({el: $("#console")});	
	state.setup($('#canvas'));
	state.gotoCard('aspit', 1);
});