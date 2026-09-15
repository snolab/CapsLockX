/**
 * .clx simulator — drive the compiled bindings with key events and show
 * what would come out the other side.
 *
 *   input   real keydown/keyup on the pad, virtual hold toggles (for keys the
 *           browser handles badly, CapsLock above all), or a typed edge trace
 *           like  [capslock [space 1 space] capslock]
 *   output  a transcript of what the focused app would receive: typed text
 *           inline, everything else as ⟨markers⟩, plus a per-gesture log.
 *
 * Matching mirrors the engine: a body key fires on its own key-DOWN while its
 * holders are down; a chord tap fires when the whole group is back up with
 * nothing struck in between. Bindings are indexed by a flattened match key
 * (sorted holder set + struck key), so `capslock[ alt[ 4 ] ]` and a live
 * press of alt+capslock then 4 meet in the middle.
 *
 * Deliberate simplifications, shown on the page: `->` order is matched as
 * `+`; multi-gesture patterns and multi-key bodies are not simulated.
 */
(function (root) {
  "use strict";

  var MODS = { shift: 1, ctrl: 1, alt: 1, win: 1 };
  var TRIGGERS = { space: 1, capslock: 1, insert: 1, scrolllock: 1 };
  var POINTER = { mb1: 1, mb2: 1, mb3: 1, mb4: 1, mb5: 1, wheelup: 1, wheeldown: 1 };
  function groupable(k) {
    return MODS[k] || TRIGGERS[k] || POINTER[k];
  }

  // ── binding index ─────────────────────────────────────────────────────────
  function expandGroup(g) {
    if (g.conn === "|")
      return g.keys.map(function (k) {
        return [k.name];
      });
    return [
      g.keys.map(function (k) {
        return k.name;
      }),
    ];
  }
  /** -> [{holders:[..], struck: name|null}] or [] if not simulable */
  function flatten(g) {
    if (g.kind === "Tap") {
      return expandGroup(g.group).map(function (keys) {
        return { holders: keys, struck: null };
      });
    }
    var gv = expandGroup(g.group),
      body = g.body.gestures;
    if (body.length !== 1) return [];
    var inner = body[0];
    if (inner.kind === "Tap" && inner.group.keys.length === 1) {
      var name = inner.group.keys[0].name;
      return gv.map(function (keys) {
        return { holders: keys, struck: name };
      });
    }
    if (inner.kind === "Hold") {
      var sub = flatten(inner),
        out = [];
      gv.forEach(function (keys) {
        sub.forEach(function (s) {
          out.push({ holders: keys.concat(s.holders), struck: s.struck });
        });
      });
      return out;
    }
    return [];
  }
  function keyOf(holders, struck) {
    var h = holders.slice().sort().join("+");
    return struck ? h + "[" + struck + "]" : h + ".";
  }
  function index(ast, src, diags) {
    var idx = {},
      skipped = [],
      errLines = {};
    (diags || []).forEach(function (d) {
      if (d.level === "error") errLines[d.line] = true;
    });
    ast.items.forEach(function (b) {
      var line = src.slice(0, b.span[0]).split("\n").length;
      // Same rule as compile(): an errored line is not a binding.
      if (errLines[line]) {
        skipped.push({ line: line, why: "has errors" });
        return;
      }
      if (b.pattern.gestures.length !== 1) {
        skipped.push({ line: line, why: "multi-gesture pattern" });
        return;
      }
      var vs = flatten(b.pattern.gestures[0]);
      if (!vs.length) {
        skipped.push({ line: line, why: "multi-key body" });
        return;
      }
      vs.forEach(function (v) {
        var k = keyOf(v.holders, v.struck);
        if (!idx[k]) idx[k] = { binding: b, line: line }; // first wins, like the engine
      });
    });
    return { idx: idx, skipped: skipped };
  }

  // ── what the app receives ─────────────────────────────────────────────────
  var PRINTABLE = {
    space: " ",
    enter: "\n",
    tab: "\t",
    minus: "-",
    equal: "=",
    comma: ",",
    period: ".",
    slash: "/",
    backslash: "\\",
    bracketleft: "[",
    bracketright: "]",
  };
  function charFor(k, shift) {
    if (/^[a-z]$/.test(k)) return shift ? k.toUpperCase() : k;
    if (/^[0-9]$/.test(k)) return shift ? ")!@#$%^&*("[+k] : k;
    if (PRINTABLE[k] !== undefined) return PRINTABLE[k];
    return null;
  }
  function label(k) {
    return (
      {
        capslock: "CapsLock",
        space: "Space",
        enter: "Enter",
        backspace: "Backspace",
        escape: "Esc",
        shift: "Shift",
        ctrl: "Ctrl",
        alt: "Alt",
        win: "Win",
      }[k] || (k.length === 1 ? k.toUpperCase() : k[0].toUpperCase() + k.slice(1))
    );
  }
  var MODNAME = { c: "Ctrl", s: "Shift", w: "Win", a: "Alt" };
  function tgt(t) {
    if (!t) return "";
    var u = t.pct ? "%" : "";
    switch (t.kind) {
      case "Abs":
        return t.x + u + " " + t.y + u;
      case "Rel":
        return (
          { window: "win", cursor: "cur", screen: "screen" }[t.base] + " " + t.x + u + " " + t.y + u
        );
      case "Cursor":
        return "cur";
      case "Elem":
        return "@" + (t.role ? t.role + ":" : "") + JSON.stringify(t.name);
      case "Mark":
        return "@" + t.name;
    }
    return "?";
  }

  function Sim(opts) {
    this.opts = opts; // {onOutput(text|marker), onLog(entry), onState(held)}
    this.idx = {};
    this.skipped = [];
    this.held = []; // press order
    this.session = null; // {keys:{}, struck:false}
    this.shiftDown = false;
  }
  Sim.prototype.setCompiled = function (r, src) {
    var ix = index(r.ast, src, r.diagnostics);
    this.idx = ix.idx;
    this.skipped = ix.skipped;
  };
  Sim.prototype.emit = function (text, cls) {
    this.opts.onOutput(text, cls || "text");
  };
  Sim.prototype.marker = function (s) {
    this.opts.onOutput("⟨" + s + "⟩", "mark");
  };

  Sim.prototype.runActions = function (block) {
    var self = this;
    block.cmds.forEach(function (c) {
      switch (c.cmd) {
        case "type":
          self.emit(c.text);
          break;
        case "key": {
          var ch = c.mods.length ? null : charFor(c.key, false);
          if (c.key === "backspace" && !c.mods.length) {
            self.opts.onOutput(null, "backspace");
            break;
          }
          if (ch !== null) self.emit(ch);
          else
            self.marker(
              c.mods
                .map(function (m) {
                  return MODNAME[m];
                })
                .concat([label(c.key)])
                .join("+"),
            );
          break;
        }
        case "wait":
          self.marker("wait " + c.ms + "ms");
          break;
        case "waitfor":
          self.marker(
            "wait for " +
              (c.what.kind === "Text" ? JSON.stringify(c.what.text) : tgt(c.what)) +
              " ≤" +
              c.timeoutMs +
              "ms",
          );
          break;
        case "move":
          self.marker("move " + tgt(c.target) + (c.click ? ", click" : ""));
          break;
        case "click":
          self.marker(
            c.mods
              .map(function (m) {
                return MODNAME[m];
              })
              .concat([c.button])
              .join("+") +
              " click" +
              (c.times > 1 ? " ×" + c.times : "") +
              (c.target ? " " + tgt(c.target) : ""),
          );
          break;
        case "mousedown":
          self.marker(c.button + " down");
          break;
        case "mouseup":
          self.marker(c.button + " up");
          break;
        case "drag":
          self.marker("drag " + tgt(c.from) + " → " + tgt(c.to));
          break;
        case "scroll":
          self.marker(
            "scroll " +
              c.dir +
              " " +
              c.amount +
              (c.unit === "px" ? "px" : "") +
              (c.target ? " " + tgt(c.target) : ""),
          );
          break;
        case "mark":
          self.marker("mark " + c.name);
          break;
        default:
          self.marker(c.name + (c.args.length ? " " + c.args.join(" ") : ""));
          break;
      }
    });
  };

  Sim.prototype.fire = function (matchKey, gestureText) {
    var hit = this.idx[matchKey];
    if (hit) {
      this.opts.onLog({
        gesture: gestureText,
        line: hit.line,
        action: hit.binding.action,
        hit: true,
      });
      this.runActions(hit.binding.action);
      return true;
    }
    return false;
  };

  Sim.prototype.down = function (k) {
    if (this.held.indexOf(k) >= 0) return; // repeat
    if (k === "shift") this.shiftDown = true;
    var holders = this.held.filter(groupable);
    this.held.push(k);
    if (groupable(k)) {
      if (!this.session) this.session = { keys: {}, struck: false };
      this.session.keys[k] = 1;
      this.opts.onState(this.held);
      return; // holds fire on release
    }
    if (holders.length === 0) {
      // plain typing, passes through
      var ch = charFor(k, this.shiftDown);
      if (k === "backspace") this.opts.onOutput(null, "backspace");
      else if (ch !== null) this.emit(ch);
      else this.marker(label(k));
      this.opts.onLog({ gesture: k, passthrough: true });
      this.opts.onState(this.held);
      return;
    }
    if (this.session) this.session.struck = true;
    var text =
      (holders.length > 1 ? "(" + holders.slice().sort().join("+") + ")" : holders[0]) +
      "[ " +
      k +
      " ]";
    if (!this.fire(keyOf(holders, k), text)) {
      // unmapped inside a hold: the engine passes it through
      var ch2 = charFor(k, this.shiftDown);
      if (ch2 !== null) this.emit(ch2);
      else this.marker(label(k));
      this.opts.onLog({ gesture: text, miss: true });
    }
    this.opts.onState(this.held);
  };

  Sim.prototype.up = function (k) {
    var i = this.held.indexOf(k);
    if (i < 0) return;
    this.held.splice(i, 1);
    if (k === "shift") this.shiftDown = false;
    if (groupable(k) && this.session) {
      var anyLeft = Object.keys(this.session.keys).some(function (x) {
        return this.held.indexOf(x) >= 0;
      }, this);
      if (!anyLeft) {
        var keys = Object.keys(this.session.keys).sort(),
          struck = this.session.struck;
        this.session = null;
        if (!struck) {
          var text = (keys.length > 1 ? "(" + keys.join("+") + ")" : keys[0]) + ".";
          if (!this.fire(keyOf(keys, null), text)) {
            // bare trigger taps emit their native key, modifiers alone do nothing
            if (keys.length === 1 && keys[0] === "space") {
              this.emit(" ");
              this.opts.onLog({ gesture: text, passthrough: true });
            } else if (keys.length === 1 && TRIGGERS[keys[0]]) {
              this.marker(label(keys[0]));
              this.opts.onLog({ gesture: text, passthrough: true });
            } else this.opts.onLog({ gesture: text, miss: true, silent: true });
          }
        }
      }
    }
    this.opts.onState(this.held);
  };

  /** Replay a typed trace:  [capslock [space 1 space] capslock]  or  space[ 3 ]-style spec */
  Sim.prototype.replayTrace = function (text) {
    var toks = text.trim().split(/\s+/).filter(Boolean),
      self = this,
      n = 0;
    toks.forEach(function (t) {
      if (t[0] === "[") {
        self.down(t.slice(1));
        n++;
      } else if (t[t.length - 1] === "]") {
        self.up(t.slice(0, -1));
        n++;
      } else {
        self.down(t);
        self.up(t);
        n++;
      }
    });
    return n;
  };
  Sim.prototype.releaseAll = function () {
    var self = this;
    this.held
      .slice()
      .reverse()
      .forEach(function (k) {
        self.up(k);
      });
  };

  // browser event.code -> key name (same table as gesture-parser)
  var CODE = {
    ShiftLeft: "shift",
    ShiftRight: "shift",
    ControlLeft: "ctrl",
    ControlRight: "ctrl",
    AltLeft: "alt",
    AltRight: "alt",
    MetaLeft: "win",
    MetaRight: "win",
    Space: "space",
    CapsLock: "capslock",
    Enter: "enter",
    Tab: "tab",
    Escape: "escape",
    Backspace: "backspace",
    Delete: "delete",
    Insert: "insert",
    ScrollLock: "scrolllock",
    Minus: "minus",
    Equal: "equal",
    BracketLeft: "bracketleft",
    BracketRight: "bracketright",
    Backslash: "backslash",
    Comma: "comma",
    Period: "period",
    Slash: "slash",
    ArrowUp: "up",
    ArrowDown: "down",
    ArrowLeft: "left",
    ArrowRight: "right",
    Home: "home",
    End: "end",
    PageUp: "pageup",
    PageDown: "pagedown",
  };
  function nameOf(code) {
    if (CODE[code]) return CODE[code];
    if (/^Key[A-Z]$/.test(code)) return code.slice(3).toLowerCase();
    if (/^Digit[0-9]$/.test(code)) return code.slice(5);
    if (/^F\d{1,2}$/.test(code)) return code.toLowerCase();
    return code.toLowerCase();
  }

  var api = { Sim: Sim, nameOf: nameOf, keyOf: keyOf, flatten: flatten, groupable: groupable };
  if (typeof module === "object" && module.exports) module.exports = api;
  else root.CLXRun = api;
})(typeof self !== "undefined" ? self : this);
