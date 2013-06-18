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
