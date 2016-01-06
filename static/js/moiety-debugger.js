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
        row.append($('<td class="name">' + v[0] + '</td><td class="value">' + v[1] + '</td>'));
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

DebugView.addTab('CARD', {
    initialize: function() {
        this.listenTo(state, 'change:cardid', this.render);
    },

    render: function() {
        if (!state.card)
            return;
        var name = state.card.name;
        if (name >= 0)
            name = state.cardNames[name];
        this.$el.html(renderVarTable(2,
            ['zip_mode', state.card.zip_mode],
            ['name', name]
        ));
        this.$el.append(renderScript(state.card.script));
    }
});

var debug;
$(function() {
    debug = new DebugView({el: $("#debugger")});
});
