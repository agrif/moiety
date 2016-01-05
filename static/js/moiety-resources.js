var resources = null;

function loadResource(stack, type, id) {
	var defaultPriority = 2;
	var p = loadResourceWithPriority(defaultPriority, stack, type, id);
	prefetchWithPriority(defaultPriority, stack, type, id);
	return p;
}

function loadResourceWithPriority(priority, stack, type, id) {
	var p;
	var ext = '.json';
	switch (type) {
	case 'tBMP':
		ext = '.png';
		break;
	case 'tMOV':
		ext = '.mov';
		break;
	case 'tWAV':
		ext = '.wav';
		break;
	}
	var url = "resources/" + stack + "/" + type + "/" + id + ext;
	
	var cached = resources.getItem(url);
	if (cached != null) {
		resources.setItem(url, cached, {priority: priority});
		return cached;		
	}
	
	switch (type) {
	case 'tBMP':
		var d = new jQuery.Deferred();
		var img = new Image();
		img.src = url;
		img.onload = function() {
			d.resolve(img);
		};
		img.onerror = function() {
			d.reject();
		};
		p = d.promise();
		break;
	case 'tMOV':
		var d = new jQuery.Deferred();
		var mov = document.createElement("video");
		var src = document.createElement("source");
		$(src).attr('type', 'video/quicktime');
		$(src).attr('src', url);
		$(mov).append(src);
		$(mov).attr('preload', 'auto');
		
		$(mov).on("canplay", function() {
			d.resolve(mov);
		});
		$(mov).on("error", function() {
			d.reject();
		});
		//mov.load();
		p = d.promise();
		break;
	case 'tWAV':
		var d = new jQuery.Deferred();
		var snd = new Audio();
		snd.src = url;
		// huh, onloadeddata doesn't work on chrome
		$(snd).on("canplay", function() {
			d.resolve(snd);
		});
		$(snd).on("error", function() {
			d.reject();
		});
		snd.load();
		p = d.promise();
		break;
	default:
		p = jQuery.getJSON(url);
	}
	
	log.status('loading <a href="' + url + '">' + url + '</a>', p);
	resources.setItem(url, p, {priority: priority});
	return p;
}

function prefetchStackWithPriority(priority, stack) {
	prefetchWithPriority(priority, stack, 'NAME', 1);
	prefetchWithPriority(priority, stack, 'NAME', 2);
	prefetchWithPriority(priority, stack, 'NAME', 3);
	prefetchWithPriority(priority, stack, 'NAME', 4);
	prefetchWithPriority(priority, stack, 'NAME', 5);
}

function prefetchCardWithPriority(priority, stack, cardid) {
	prefetchWithPriority(priority, stack, 'CARD', cardid);
	prefetchWithPriority(priority, stack, 'PLST', cardid);
	prefetchWithPriority(priority, stack, 'BLST', cardid);
	prefetchWithPriority(priority, stack, 'HSPT', cardid);
	prefetchWithPriority(priority, stack, 'SLST', cardid);
}

function prefetchScriptWithPriority(priority, script, stack, cardid) {
	jQuery.each(script, function(i, cmd) {
		switch (cmd.name) {
		case 'branch':
			jQuery.each(cmd.cases, function(v, s) {
				prefetchScriptWithPriority(priority, s, stack, cardid);
			});
			break;
		case 'goto-card':
			var c = cmd.arguments[0];
			prefetchCardWithPriority(priority - 1, stack, c);
			break;
		case 'goto-stack':
			// only resolve this if we're on the same stack as player
			if (stack != state.stack)
				return;
			
			var stackid = cmd.arguments[0];
			var code = (cmd.arguments[1] << 16) | cmd.arguments[2];
			var stackname = state.stackNames[stackid];
			var pRMAP = loadResourceWithPriority(priority - 1, stackname, 'RMAP', 1);
			pRMAP.done(function(rmap) {
				var c = null;
				for (i in rmap) {
					if (rmap[i] == code) {
						c = i;
						break;
					}
				}
				if (c != null) {
					prefetchStackWithPriority(priority - 1, stackname);
					prefetchCardWithPriority(priority - 1, stackname, c);
				}
			});
			break;
		case 'play-wav':
			var wavid = cmd.arguments[0];
			loadResourceWithPriority(priority, stack, 'tWAV', wavid);
			break;
		};
	});
}

function prefetchWithPriority(priority, stack, type, id) {
	if (priority <= 0)
		return;
	
	var res = loadResourceWithPriority(priority, stack, type, id);
	
	switch (type) {
	case 'PLST':
		res.done(function(plst) {
			jQuery.each(plst.slice(1), function(i, v) {
				loadResourceWithPriority(priority, stack, 'tBMP', v.bitmap);
			});
		});
		break;
	case 'CARD':
		res.done(function(card) {
			jQuery.each(card.script, function(h, s) {
				prefetchScriptWithPriority(priority, s, stack, id);
			});
		});
		break;
	case 'HSPT':
		res.done(function(hspt) {
			jQuery.each(hspt.slice(1), function(i, v) {
				jQuery.each(v.script, function(h, s) {
					prefetchScriptWithPriority(priority, s, stack, id);
				});
			});
		});
		break;
    case 'SLST':
        res.done(function(slst) {
            jQuery.each(slst.slice(1), function(i, v) {
                jQuery.each(v.sounds, function(h, s) {
                    loadResourceWithPriority(priority, stack, 'tWAV', s.sound_id);
                });
            });
        });
        break;
	};
}

$(function() {
	resources = new Cache(300);
});
