var scriptCommands = {
	'activate-plst': function(record) {
		state.activatePLST(record);
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
	
	'goto-card': function(cardid) {
		state.gotoCard(state.stackname, cardid);
	},
	
	'set-cursor': function(cursorid) {
		state.setCursor(cursorid);
	},
	
	'set-var': function(nameid, value) {
		var name = state.variableNames[nameid];
		state.setVariable(name, value);
	}
};

var externalCommands = {
	'xasetupcomplete': function() {
		state.gotoCard("aspit", 1);
	}
};
