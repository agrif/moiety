var scriptCommands = {
	'activate-blst': function(record) {
		return state.activateBLST(record);
	},
	
	'activate-plst': function(record) {
		return state.activatePLST(record);
	},
	
	'call': function(nameid, argumentCount) {
		var name = state.commandNames[nameid];
		var args = Array.prototype.slice.call(arguments, 2, 2 + argumentCount);
		if (name in externalCommands) {
			return externalCommands[name].apply(externalCommands, args);
		} else {
			console.message("!!! (stub call) " + name + " " + args.toString());			
		}
	},
	
	'disable-update': function() {
		state.disableScreenUpdate();
	},
	
	'enable-update': function() {
		return state.enableScreenUpdate();
	},
	
	'goto-card': function(cardid) {
		return state.gotoCard(state.stackname, cardid);
	},
	
	'goto-stack': function(stackid, code_hi, code_low) {
		var stackname = state.stackNames[stackid];
		var code = (code_hi << 16) | code_low;
		return loadResource(stackname, 'RMAP', 1).then(function(rmaps) {
			var cardid = null;
			for (i in rmaps) {
				if (rmaps[i] == code) {
					cardid = i;
					break;
				}
			}
			if (cardid == null)
				return jQuery.Deferred.reject();
			return state.gotoCard(stackname, cardid);
		});
	},
	
	'increment': function(varid, value) {
		var name = state.variableNames[varid];
		var v = state.getVariable(name);
		state.setVariable(name, v + value);
	},
	
	'pause': function(ms, u0) {
		return jQuery.Deferred(function(d) {
			setTimeout(function() {
				d.resolve();
			}, ms);
		});
	},
	
	'play-wav': function(wavid, volume, u0) {
		// ignore volume, since in riven it's almost always 256
		// this command is also asynchronous
		// (sound plays in background)
		return loadResource(state.stackname, 'tWAV', wavid).done(function(r) {
			state.playSound(r);
		});
	},
	
	'reload': function() {
		return state.gotoCard(state.stackname, state.cardid);
	},
	
	'set-cursor': function(cursorid) {
		state.setCursor(cursorid);
	},
	
	'set-var': function(nameid, value) {
		var name = state.variableNames[nameid];
		state.setVariable(name, value);
	},
	
	'transition': function(transition, left, top, right, bottom) {
		// we ignore the rectangle, because it's mostly never used in riven
		state.scheduleTransition(transition);
	}
};

var externalCommands = {
	'xasetupcomplete': function() {
		state.scheduleTransition(16);
		return state.gotoCard("aspit", 1);
	}
};
