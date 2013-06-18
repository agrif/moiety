var stackNames = [
	'aspit', 'bspit', 'gspit', 'jspit', 'ospit',
	'pspit', 'rspit', 'tspit'
];

var resourceTypes = [
	'BLST', 'CARD', 'FLST', 'HSPT',
	'MLST', 'NAME', 'PLST', 'RMAP',
	'SFXE', 'SLST', 'tBMP', 'tMOV',
	'tWAV', 'VARS', 'VERS', 'ZIPS'
];

var consoleCommands = {
	'load': function(stack, type, id) {
		if (arguments.length != 3) throw "invalid arguments";
		loadResource(stack, type, id);
	},
	'complete:load': function(i, part) {
		switch (i) {
		case 0:
			return console.completionsFrom(part, stackNames);
		case 1:
			return console.completionsFrom(part, resourceTypes);
		}
		return [];
	},
	
	'goto-card': function(stackname, cardid) {
		if (arguments.length != 2) throw "invalid arguments";
		state.gotoCard(stackname, cardid);
	},
	'complete:goto-card': function(i, part) {
		if (i == 0)
			return console.completionsFrom(part, stackNames);
		return [];
	},
	
	'activate-plst': function(plstid) {
		if (arguments.length != 1) throw "invalid arguments";
		state.activatePLST(parseInt(plstid));
	}
};

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
			completeHandle: function(line) {
				return viewthis.complete(line);
			},
			cols: 80,
			animateScroll: true,
			promptHistory: true
		});
	},
	
	splitCommand: function(line) {
		return line.split(/ +/);
	},
	
	handle: function(line, report) {
		args = this.splitCommand(line);
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
	
	getCommands: function() {
		var cmds = jQuery.map(consoleCommands, function(v, k) {
			if (k.indexOf("complete:") == 0) {
				return null;
			} else {
				return k;				
			}
		});
		return jQuery.grep(cmds, function(cmd, i) {
			return (cmd != null);
		});
	},
	
	completionsFrom: function(needle, haystack) {
		var completions = [];
		for (i in haystack) {
			if (haystack[i].indexOf(needle) == 0) {
				var completion = haystack[i].slice(needle.length);
				completions.push(completion + ' ');
			}
		}
		return completions;
	},
	
	complete: function(line) {
		var commands = this.getCommands();
		if (line) {
			args = this.splitCommand(line);
			if (args.length > 1) {
				// argument completion
				if (!('complete:' + args[0] in consoleCommands))
					return [];
				completer = consoleCommands['complete:' + args[0]];
				return completer(args.length - 2, args[args.length - 1]);
			} else {
				// command complition
				return this.completionsFrom(args[0], commands);
			}
		} else {
			// list all commands
			return this.completionsFrom('', commands);
		}
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

var console;
$(function() {
	console = new ConsoleView({el: $("#console")});	
});