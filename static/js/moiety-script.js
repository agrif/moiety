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
	
	'play-wav': function(wavid, volume, u0) {
		// ignore volume, since in riven it's almost always 255
		var d = jQuery.Deferred();
		loadResource(state.stackname, 'tWAV', wavid).done(function(r) {
			state.playSound(r).fail(d.reject).done(d.resolve);
		});
		return d.promise();
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
