var console;
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
			report([{msg: msg, className: "jquery-console-message-error"}]);
		}
		
		switch (args[0]) {
		case "load":
			if (args.length != 4) {
				doerror("incorrect arguments");
			} else {
				loadResource(args[1], args[2], args[3]);
				return ""
			}
			break;
		default:
			doerror("invalid command");
		}
	},
	
	message: function(msg) {
		this.console.message(msg);
	}
});

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
	
	var msg = $('<div/>').html('loading <a href="' + url + '">' + url + '</a>');
	var status = $('<div/>').addClass('status status-pending').appendTo(msg);
	console.message(msg);
	p.done(function() {
		status.removeClass('status-pending').addClass('status-done');
	});
	p.fail(function() {
		status.removeClass('status-pending').addClass('status-failed');
	});
	
	return p;
}

$(function() {
	console = new ConsoleView({el: $("#console")});
	
	var ctx = $("#canvas")[0].getContext("2d");
	
	var img = new Image();
	img.src = "/resources/aspit/tBMP/0";
	img.onload = function() {
		ctx.drawImage(img, 0, 0);
	};
	
	// var music = new Audio();
	// music.src = "/resources/tspit/tWAV/1";
	// music.onplay = function() {
	// 	alert("oncanplay!");
	// };
	// music.load();
	// music.play();
});