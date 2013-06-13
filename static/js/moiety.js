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
		switch (args[0]) {
		default:
			report([{msg: "invalid command: " + args[0],
					 className: "jquery-console-message-error"}]);
		}
	},
	
	message: function(msg) {
		this.console.message(msg);
	}
});

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