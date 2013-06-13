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
		
		switch (args[0]) {
		case "load":
			if (args.length != 4) {
				doerror("incorrect arguments");
			} else {
				loadResource(args[1], args[2], args[3]);
			}
			break;
		case "goto-card":
			var stackname, cardid;
			if (args.length == 2) {
				stackname = state.stackname;
				cardid = parseInt(args[1]);
				state.gotoCard(stackname, cardid);
			} else if (args.length == 3) {
				stackname = args[1];
				cardid = parseInt(args[2]);
				state.gotoCard(stackname, cardid);
			} else {
				doerror("incorrect arguments");
			}
			break;
		case "activate-plst":
			if (args.length != 2) {
				doerror("incorrect arguments");
			} else {
				state.activatePLST(parseInt(args[1]));
			}
			break;
		default:
			doerror("invalid command");
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

var state = {
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

	changeStack: function(stackname) {
		if (stackname == this.stackname)
			return jQuery.Deferred().resolve();
		
		// load up global stack resources
		var stat = console.status("changing to stack " + stackname);
		var pCards = loadResource(stackname, 'NAME', 1);
		var pHotspots = loadResource(stackname, 'NAME', 2);
		var pCommands = loadResource(stackname, 'NAME', 3);
		var pVariables = loadResource(stackname, 'NAME', 4);
		var pStacks = loadResource(stackname, 'NAME', 5);
		var d = jQuery.when(pCards, pHotspots, pCommands, pVariables, pStacks)
		var state = this;
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
		if (stackname == this.stackname && cardid == this.cardid)
			return jQuery.Deferred().resolve();
		
		// unload current card
		
		// change stacks
		var d = jQuery.Deferred();
		var state = this;
		this.changeStack(stackname).done(function() {
			// load new card
			console.status("moving to card " + cardid, d);
			var pCard = loadResource(stackname, 'CARD', cardid);
			var pPLST = loadResource(stackname, 'PLST', cardid);
			var when = jQuery.when(pCard, pPLST);
			when.done(function(card, plst) {
				state.cardid = cardid;
				state.card = card[0];
				state.plst = plst[0];
				state.activatePLST(1).done(d.resolve).fail(d.reject);
			}).fail(d.reject);
		});
		
		return d;
	},
	
	activatePLST: function(i) {
		var ctx = $("#canvas")[0].getContext("2d");
		var record = this.plst[i];
		return loadResource(this.stackname, 'tBMP', record.bitmap).done(function(img) {
			ctx.drawImage(img, record.left, record.top, record.right-record.left, record.bottom-record.top);
		});
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
	state.gotoCard('aspit', 1);
});