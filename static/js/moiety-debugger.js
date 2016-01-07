var DebugView = Backbone.View.extend({
    events: {
        'click .tab': 'onTabClick'
    },

    initialize: function() {
        this.activeTab = 0;
        this.switchToTab(0);
    },

    switchToTab: function(n) {
        if (this.activeTab != n) {
            DebugView.tabs[this.activeTab].view.setElement(undefined);
        }
        this.activeTab = n;
        this.render();
    },

    onTabClick: function(e) {
        var el = $(e.currentTarget);
        this.switchToTab(el.index());
    },

    render: function() {
        var activeTab = DebugView.tabs[this.activeTab];
        this.$el.empty();
        var tabs = $('<ul class="tabs"></ul>');
        this.$el.append(tabs)
        jQuery.each(DebugView.tabs, function(i, tab) {
            if (i == activeTab.i) {
                tabs.append('<li class="tab selected">' + tab.label + '</li>');
            } else {
                tabs.append('<li class="tab">' + tab.label + '</li>');
            }
        });

        var panel = $('<div id="debugpanel"></div>');
        this.$el.append(panel);
        activeTab.view.setElement(panel).render();
    }
});

DebugView.tabs = [];
DebugView.addTab = function(label, viewproto) {
    var Proto = Backbone.View.extend(viewproto);
    DebugView.tabs.push({
        i: DebugView.tabs.length,
        label: label,
        view: new Proto()
    });
}

function renderVarTable() {
    var table = $('<table class="variables"></table>');
    var width = undefined;
    var row = undefined;
    jQuery.each(arguments, function(i, v) {
        if (i == 0) {
            width = v;
            return;
        }
        if ((i - 1) % width == 0) {
            if (row) {
                table.append(row);
            }
            row = $('<tr></tr>');
        }        
        row.append($('<td class="name">' + v[0] + '</td><td class="value">' + JSON.stringify(v[1]) + '</td>'));
    });
    if (row) {
        table.append(row);
    }
    return table;
}

function renderScriptText(ind, script) {
    var s = "";
    var indent = function(end) {
        return "  ".repeat(ind) + end;
    };
    jQuery.each(script, function(handler, instrs) {
        s += indent(handler + ':\n');
        ind += 1;
        jQuery.each(instrs, function(i, inst) {
            s += indent(inst.name);
            if (inst.name == 'branch') {
                s += ' ' + state.variableNames[inst.variable] + ':\n';
                s += renderScriptText(ind + 1, inst.cases);
            }
            if (!inst.arguments) {
                s += '\n';
                return;
            }
            jQuery.each(inst.arguments, function(j, arg) {
                s += ' ';
                if (inst.name == 'call' && j == 0) {
                    s += state.commandNames[arg];
                } else if ((inst.name == 'set-var' || inst.name == 'increment') && j == 0) {
                    s += state.variableNames[arg];
                } else if (inst.name == 'goto-stack' && j == 0) {
                    s += state.stackNames[arg];
                } else {
                    s += arg;
                }
            });
            s += '\n';
        });
        ind -= 1;
    });
    return s;
}

function renderScript(script) {
    return $('<pre></pre>').text(renderScriptText(0, script));
}

DebugView.addTab('Summary', {
    initialize: function() {
        this.listenTo(state, 'change:stackname', this.render);
        this.listenTo(state, 'change:cardid', this.render);
    },

    render: function() {
        this.$el.html(renderVarTable(2,
            ['stackname', state.stackname],
            ['cardid', state.cardid]
        ));
    }
});

DebugView.addTab('Variables', {
    initialize: function() {
        this.listenTo(state, 'change:variables', this.render);
    },

    render: function() {
        var vars = [5];
        var names = [];
        jQuery.each(state.variables, function(k, v) {
            names.push(k);
        });
        names.sort();
        jQuery.each(names, function(i, k) {
            vars.push([k, state.variables[k]]);
        });
        this.$el.html(renderVarTable.apply(null, vars));
    }
});

DebugView.addTab('Resources', {
    initialize: function() {
        this.listenTo(state, 'change:cardid', this.render);
        this.stack = null;
        this.type = null;
        this.id = null;
        this.res = null;
        this.raw = false;
        this.openResource('aspit', 'tBMP', 1);
    },

    cardTypes: ['CARD', 'PLST', 'BLST', 'HSPT', 'SLST'],
    
    openResource: function(stack, type, id) {
        var view = this;
        var effType = type;
        if (jQuery.inArray(type, this.cardTypes) > -1) {
            effType = 'CARD';
        }
        view.stack = stack;
        view.type = type;
        view.id = id;
        loadResource(stack, effType, id).fail(function() {
            view.res = null;
            view.render();
        }).then(function(r) {
            view.res = r;
            if (effType == 'CARD') {
                view.res = view.res[type.toLowerCase()];
            }
            view.render();
        });
    },

    rendertBMP: function(el, r) {
        el.append(renderVarTable(2,
            ['width', r.width],
            ['height', r.height]
        ));
        el.append(r);
    },

    rendertWAV: function(el, r) {
        el.append(renderVarTable(2,
            ['duration', r.duration]
        ));
        r.controls = true;
        el.append(r);
    },

    rendertMOV: function(el, r) {
        el.append(renderVarTable(2,
            ['width', r.width],
            ['height', r.height],
            ['duration', r.duration]
        ));
        r.controls = true;
        el.append(r);
    },

    renderRaw: function(el, r) {
        var view = this;
        var top = $('<div id="rawtoggle"></div>');
        el.append(top);
        if (this.raw) {
            var b = $('<a class="link">(view pretty)</a>');
            b.click(function() {
                view.raw = false;
                view.render();
            });
            top.append(b);
            
            var raw = $('<pre></pre>');
            raw.text(JSON.stringify(r, null, 2));
            el.append(raw);
        } else {
            var b = $('<a class="link">(view raw)</a>');
            b.click(function() {
                view.raw = true;
                view.render();
            });
            top.append(b);
        }
        return this.raw;
    },

    renderCARD: function(el, r) {
        if (this.renderRaw(el, r))
            return;
        var name = r.name;
        if (name >= 0 && this.stack == state.stackname)
            name = state.cardNames[name];
        el.append(renderVarTable(2,
            ['zip_mode', r.zip_mode],
            ['name', name]
        ));
        el.append(renderScript(r.script));
    },

    renderPLST: function(el, r) {
        if (this.renderRaw(el, r))
            return;
        el.append('No pretty renderer for this resource.');
    },

    renderBLST: function(el, r) {
        if (this.renderRaw(el, r))
            return;
        el.append('No pretty renderer for this resource.');
    },

    renderHSPT: function(el, r) {
        if (this.renderRaw(el, r))
            return;
        el.append('No pretty renderer for this resource.');
    },

    renderSLST: function(el, r) {
        if (this.renderRaw(el, r))
            return;
        el.append('No pretty renderer for this resource.');
    },

    renderNAME: function(el, r) {
        if (this.renderRaw(el, r))
            return;
        el.append('No pretty renderer for this resource.');
    },

    renderRMAP: function(el, r) {
        if (this.renderRaw(el, r))
            return;
        el.append('No pretty renderer for this resource.');
    },

    render: function() {
        var view = this;
        this.$el.empty();
        var form = $('<form id="resource"></form>');
        form.append($('<label for="type">Look up:</label>'));
        form.append($('<input type="text" id="stack" placeholder="stack" value="' + this.stack + '"></input>'));
        form.append($('<input type="text" id="type" placeholder="TYPE" value="' + this.type + '"></input>'));
        form.append($('<input type="number" id="id" min="1" value="' + this.id + '"></input>'));
        form.append($('<input type="submit" value="Go">'));
        form.submit(function(ev) {
            ev.preventDefault();
            var stack = $('form#resource > input#stack').val();
            var type = $('form#resource > input#type').val();
            var id = $('form#resource > input#id').val();
            view.openResource(stack, type, id);
        });
        this.$el.append(form);
        
        if (state.cardid) {
            var list = $('<div id="thiscard">This card: </div>');
            jQuery.each(this.cardTypes, function(i, ty) {
                var el = $('<a class="link"></a>');
                el.text(ty);
                list.append(el);
                list.append(' ');
                el.click(function() {
                    view.openResource(state.stackname, ty, state.cardid);
                });
            });
            this.$el.append(list);
        }
        
        var res = $('<div id="resource"></div>');
        if (this.res) {
            var renderer = this['render' + this.type];
            if (renderer) {
                renderer.apply(this, [res, this.res]);
            } else {
                res.text("No renderer for " + this.type + ".");
            }
        } else {
            res.text("Resource not found.");
        }
        this.$el.append(res);
    }
});

var debug;
$(function() {
    debug = new DebugView({el: $("#debugger")});
});
