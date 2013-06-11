$(function() {
	var ctx = $("#canvas")[0].getContext("2d");
	
	var img = new Image();
	img.src = "/resources/aspit/tBMP/0";
	img.onload = function() {
		ctx.drawImage(img, 0, 0);
	};
	
	var music = new Audio();
	music.src = "/resources/tspit/tWAV/1";
	music.onplay = function() {
		alert("oncanplay!");
	};
	music.load();
	music.play();
});