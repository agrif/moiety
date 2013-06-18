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

var console;
$(function() {
	console = new ConsoleView({el: $("#console")});	
});