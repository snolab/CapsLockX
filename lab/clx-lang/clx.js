/**
 * .clx — draft language, JS prototype.
 *
 *   pattern  =>  action            one binding per line; '#' comments
 *
 *   (capslock+space)[ 1 ]  => k f1
 *   clx[ h ]               => k left
 *   (s+c).                 => lock toggle
 *   clx[ g ]               => { k "hi"; w 200ms; k enter }
 *
 * Pattern side is the gesture notation (lab/chord-fn-row/#notation).
 * Action side is the existing CLX agent command language (CLAUDE.md), which
 * `clx agent --exec` already runs, plus bare built-ins like `lock toggle`.
 *
 * Deliberately a hand-written lexer + recursive-descent parser with spans on
 * every node: it is small enough to port to Rust line-for-line, and the
 * conformance corpus (planned: lab/clx-lang/corpus/) is what will keep the
 * two in step once the design settles.
 *
 * Pipeline:  source -> tokens -> AST -> {edge IR, diagnostics}
 * The edge IR is the trace form ([capslock [space 1 space] capslock]) --
 * the same thing the recognizer emits and the same thing an emitter consumes.
 */
(function (root) {
  "use strict";

  // ── keys ──────────────────────────────────────────────────────────────────
  var KEY_ALIASES = {
    // NOTE: no single-letter aliases (s, c) -- they would shadow the real
    // letter keys, and clx[ c ] is tile-windows. Single letters are letters.
    clx: "(space|capslock)",
    esc: "escape",
    del: "delete",
    bs: "backspace",
    ret: "enter",
    cr: "enter",
    lshift: "shift",
    rshift: "shift",
    lctrl: "ctrl",
    rctrl: "ctrl",
    lalt: "alt",
    ralt: "alt",
    lwin: "win",
    rwin: "win",
    cmd: "win",
    meta: "win",
    "-": "minus",
    "=": "equal",
    lbtn: "mb1",
    rbtn: "mb2",
    mbtn: "mb3",
    lmb: "mb1",
    rmb: "mb2",
    mmb: "mb3",
    xbtn1: "mb4",
    xbtn2: "mb5",
  };
  var NAMED_KEYS = (
    "space capslock insert scrolllock shift ctrl alt win enter tab " +
    "delete backspace left up right down pageup pagedown home end escape " +
    "bracketleft bracketright backslash period comma slash minus equal " +
    "f1 f2 f3 f4 f5 f6 f7 f8 f9 f10 f11 f12 " +
    "mb1 mb2 mb3 mb4 mb5 wheelup wheeldown"
  ).split(" ");
  // Pattern-side pointer keys. Parsed today; the engine has no mouse hook yet
  // (WH_MOUSE_LL / CGEventTap mouse mask), so they are flagged, not bound.
  var POINTER_KEYS = { mb1: 1, mb2: 1, mb3: 1, mb4: 1, mb5: 1, wheelup: 1, wheeldown: 1 };
  var KNOWN = {};
  NAMED_KEYS.forEach(function (k) {
    KNOWN[k] = 1;
  });
  var MODIFIERS = { shift: 1, ctrl: 1, alt: 1, win: 1 };
  var TRIGGERS = { space: 1, capslock: 1, insert: 1, scrolllock: 1 };

  function isKeyName(s) {
    return /^[a-z]$/.test(s) || /^[0-9]$/.test(s) || KNOWN[s] === 1;
  }

  // ── lexer ─────────────────────────────────────────────────────────────────
  // Token: {t, v, line, col, pos, end}
  function lex(src) {
    var toks = [],
      i = 0,
      line = 1,
      col = 1,
      n = src.length;
    function push(t, v, start, startCol) {
      toks.push({ t: t, v: v, line: line, col: startCol, pos: start, end: i });
    }
    while (i < n) {
      var ch = src[i],
        start = i,
        startCol = col;
      if (ch === "\n") {
        push("NL", "\n", start, startCol);
        i++;
        line++;
        col = 1;
        continue;
      }
      if (ch === " " || ch === "\t" || ch === "\r") {
        i++;
        col++;
        continue;
      }
      if (ch === "#") {
        while (i < n && src[i] !== "\n") {
          i++;
          col++;
        }
        continue;
      }
      if (ch === '"') {
        var s = "",
          j = i + 1,
          c2 = col + 1;
        while (j < n && src[j] !== '"') {
          if (src[j] === "\\" && j + 1 < n) {
            var e = src[j + 1];
            s += e === "n" ? "\n" : e === "t" ? "\t" : e;
            j += 2;
            c2 += 2;
          } else {
            if (src[j] === "\n") {
              line++;
              c2 = 0;
            }
            s += src[j];
            j++;
            c2++;
          }
        }
        if (j >= n) {
          i = j;
          col = c2;
          push("ERR", "unterminated string", start, startCol);
          continue;
        }
        i = j + 1;
        col = c2 + 1;
        push("STR", s, start, startCol);
        continue;
      }
      if (src.startsWith("=>", i)) {
        i += 2;
        col += 2;
        push("FAT", "=>", start, startCol);
        continue;
      }
      if (src.startsWith("->", i)) {
        i += 2;
        col += 2;
        push("ARROW", "->", start, startCol);
        continue;
      }
      var one = {
        "(": "LP",
        ")": "RP",
        "[": "LB",
        "]": "RB",
        "{": "LC",
        "}": "RC",
        ".": "DOT",
        "+": "PLUS",
        "|": "PIPE",
        ";": "SEMI",
        "-": "KEY",
        "=": "KEY",
      }[ch];
      if (one) {
        i++;
        col++;
        push(one, ch, start, startCol);
        continue;
      }
      var m = /^[0-9]+(ms|s)\b/.exec(src.slice(i));
      if (m) {
        i += m[0].length;
        col += m[0].length;
        push("DUR", m[0], start, startCol);
        continue;
      }
      m = /^[0-9]+%/.exec(src.slice(i));
      if (m) {
        i += m[0].length;
        col += m[0].length;
        push("PCT", m[0], start, startCol);
        continue;
      }
      m = /^[0-9]+px\b/.exec(src.slice(i));
      if (m) {
        i += m[0].length;
        col += m[0].length;
        push("PX", m[0], start, startCol);
        continue;
      }
      if (ch === "@") {
        i++;
        col++;
        push("AT", "@", start, startCol);
        continue;
      }
      if (ch === ":") {
        i++;
        col++;
        push("COLON", ":", start, startCol);
        continue;
      }
      m = /^-?[0-9]+\b/.exec(src.slice(i));
      if (m && !/^[0-9]$/.test(m[0])) {
        i += m[0].length;
        col += m[0].length;
        push("NUM", m[0], start, startCol);
        continue;
      }
      m = /^(?:[cswa]-)+[A-Za-z0-9]+\b/.exec(src.slice(i)); // agent modkey: c-c, w-p, c-s-a
      if (m) {
        i += m[0].length;
        col += m[0].length;
        push("MODKEY", m[0], start, startCol);
        continue;
      }
      m = /^[A-Za-z_][A-Za-z0-9_]*/.exec(src.slice(i));
      if (m) {
        i += m[0].length;
        col += m[0].length;
        push("ID", m[0], start, startCol);
        continue;
      }
      m = /^[0-9]/.exec(src.slice(i));
      if (m) {
        i++;
        col++;
        push("ID", m[0], start, startCol);
        continue;
      }
      i++;
      col++;
      push("ERR", "unexpected character " + JSON.stringify(ch), start, startCol);
    }
    push("EOF", "", i, col);
    return toks;
  }

  // ── parser ────────────────────────────────────────────────────────────────
  function Parser(src) {
    this.src = src;
    this.toks = lex(src);
    this.i = 0;
    this.diags = [];
  }
  var P = Parser.prototype;
  P.peek = function (k) {
    return this.toks[this.i + (k || 0)];
  };
  P.at = function (t) {
    return this.peek().t === t;
  };
  P.next = function () {
    return this.toks[this.i++];
  };
  P.err = function (tok, msg) {
    this.diags.push({
      level: "error",
      msg: msg,
      line: tok.line,
      col: tok.col,
      pos: tok.pos,
      end: tok.end,
    });
  };
  P.warn = function (tok, msg) {
    this.diags.push({
      level: "warn",
      msg: msg,
      line: tok.line,
      col: tok.col,
      pos: tok.pos,
      end: tok.end,
    });
  };
  P.expect = function (t, what) {
    if (this.at(t)) return this.next();
    this.err(this.peek(), "expected " + (what || t) + " but found " + describe(this.peek()));
    return null;
  };
  function describe(tok) {
    if (tok.t === "EOF") return "end of file";
    if (tok.t === "NL") return "end of line";
    return JSON.stringify(tok.v);
  }
  P.syncLine = function () {
    while (!this.at("NL") && !this.at("EOF")) this.next();
  };

  P.file = function () {
    var items = [];
    while (!this.at("EOF")) {
      if (this.at("NL")) {
        this.next();
        continue;
      }
      var line = this.peek().line;
      var b = this.binding();
      if (b) items.push(b);
      if (!this.at("NL") && !this.at("EOF")) {
        this.err(this.peek(), "unexpected " + describe(this.peek()) + " after binding");
        this.syncLine();
      }
      void line;
    }
    return { kind: "File", items: items };
  };

  P.binding = function () {
    var start = this.peek();
    var pat = this.pattern();
    if (!pat.gestures.length) {
      this.err(start, "expected a gesture pattern");
      this.syncLine();
      return null;
    }
    if (!this.expect("FAT", '"=>"')) {
      this.syncLine();
      return null;
    }
    var act = this.action();
    return { kind: "Binding", pattern: pat, action: act, span: [start.pos, this.peek(-1).end] };
  };

  // pattern := gesture+      (juxtaposition = sequence)
  P.pattern = function (stopAtRB) {
    var gs = [],
      start = this.peek();
    while (true) {
      if (this.at("NL") || this.at("EOF") || this.at("FAT")) break;
      if (stopAtRB && this.at("RB")) break;
      var g = this.gesture();
      if (!g) break;
      gs.push(g);
    }
    return { kind: "Pattern", gestures: gs, span: [start.pos, this.peek(-1).end] };
  };

  // gesture := holdable ( '[' pattern? ']'? | '.' )?
  P.gesture = function () {
    var start = this.peek();
    var h = this.holdable();
    if (!h) return null;
    if (this.at("DOT")) {
      this.next();
      return { kind: "Tap", group: h, span: [start.pos, this.peek(-1).end] };
    }
    if (this.at("LB")) {
      this.next();
      var body = this.pattern(true);
      var closed = false;
      if (this.at("RB")) {
        this.next();
        closed = true;
      } else if (!this.at("NL") && !this.at("EOF") && !this.at("FAT")) {
        this.err(this.peek(), 'expected "]" to close the hold');
      } else {
        this.warn(
          start,
          'hold is never closed — legal (fires on key-down, like locked mode) but check for a missing "]"',
        );
      }
      return {
        kind: "Hold",
        group: h,
        body: body,
        closed: closed,
        span: [start.pos, this.peek(-1).end],
      };
    }
    // a bare key inside a body is a tap; at top level it is a tap too
    if (h.keys.length === 1) return { kind: "Tap", group: h, span: h.span };
    this.err(start, 'a group must be followed by "[" (hold) or "." (tap)');
    return { kind: "Tap", group: h, span: h.span };
  };

  // holdable := key | '(' key ((+||->) key)* ')'
  P.holdable = function () {
    var start = this.peek();
    if (this.at("LP")) {
      this.next();
      var keys = [],
        conn = null;
      var k = this.key();
      if (k) keys.push(k);
      while (this.at("PLUS") || this.at("PIPE") || this.at("ARROW")) {
        var op = this.next();
        var c = op.t === "PLUS" ? "+" : op.t === "PIPE" ? "|" : "->";
        if (conn && conn !== c)
          this.err(
            op,
            'mixed connectors in one group: "' + conn + '" and "' + c + '" — use nested groups',
          );
        conn = c;
        var k2 = this.key();
        if (k2) keys.push(k2);
      }
      this.expect("RP", '")"');
      if (keys.length < 2) this.warn(start, "a group with one key is just that key");
      return { kind: "Group", keys: keys, conn: conn || "+", span: [start.pos, this.peek(-1).end] };
    }
    var single = this.key();
    if (!single) return null;
    // `clx` expands to the alternation of the two triggers
    if (single.name === "(space|capslock)") {
      return {
        kind: "Group",
        keys: [mk("space", single), mk("capslock", single)],
        conn: "|",
        alias: "clx",
        span: single.span,
      };
    }
    return { kind: "Group", keys: [single], conn: "+", span: single.span };
  };
  function mk(name, from) {
    return { kind: "Key", name: name, span: from.span };
  }

  P.key = function () {
    var tok = this.peek();
    if (tok.t !== "ID" && tok.t !== "KEY") return null;
    this.next();
    var raw = tok.v.toLowerCase();
    var name = KEY_ALIASES[raw] || raw;
    if (name !== "(space|capslock)" && !isKeyName(name)) {
      this.err(tok, 'unknown key "' + tok.v + '"');
    }
    return { kind: "Key", name: name, raw: tok.v, span: [tok.pos, tok.end] };
  };

  // action := '{' cmd (';' cmd)* ';'? '}' | cmd
  P.action = function () {
    var start = this.peek();
    if (this.at("LC")) {
      this.next();
      var cmds = [];
      while (!this.at("RC") && !this.at("EOF")) {
        if (this.at("SEMI") || this.at("NL")) {
          this.next();
          continue;
        }
        var c = this.command(true);
        if (c) cmds.push(c);
        else this.next();
      }
      this.expect("RC", '"}"');
      return { kind: "Block", cmds: cmds, span: [start.pos, this.peek(-1).end] };
    }
    var before = this.diags.length;
    var one = this.command(false);
    if (!one) {
      if (this.diags.length === before) this.err(start, 'expected an action after "=>"');
      return { kind: "Block", cmds: [], span: [start.pos, start.end] };
    }
    return { kind: "Block", cmds: [one], span: one.span };
  };

  // The CLX agent command language, plus bare built-ins.
  P.command = function (inBlock) {
    var head = this.peek();
    if (head.t !== "ID") {
      this.err(head, 'expected an action after "=>", found ' + describe(head));
      return null;
    }
    this.next();
    var args = [];
    while (!this.at("NL") && !this.at("EOF") && !this.at("SEMI") && !(inBlock && this.at("RC"))) {
      var a = this.next();
      if (a.t === "ERR") continue; // already reported by compile()
      args.push(a);
    }
    var span = [head.pos, this.peek(-1).end];
    var name = head.v;
    var POINTER = { m: 1, hover: 1, click: 1, dbl: 1, md: 1, mu: 1, drag: 1, scroll: 1, mark: 1 };
    if (POINTER[name]) return this.pointerCommand(name, head, args, span);
    switch (name) {
      case "k": {
        if (args.length === 1 && args[0].t === "STR")
          return { kind: "Cmd", cmd: "type", text: args[0].v, span: span };
        if (
          args.length === 1 &&
          (args[0].t === "ID" || args[0].t === "MODKEY" || args[0].t === "KEY")
        ) {
          var parts = args[0].v.split("-"),
            key = parts.pop().toLowerCase(),
            mods = parts;
          var bad = mods.filter(function (m) {
            return !/^[cswa]$/.test(m);
          });
          if (bad.length) this.err(args[0], 'unknown modifier "' + bad[0] + '" (c s w a)');
          var kn = KEY_ALIASES[key] || key;
          if (!isKeyName(kn)) this.err(args[0], 'unknown key "' + key + '"');
          return { kind: "Cmd", cmd: "key", mods: mods, key: kn, span: span };
        }
        this.err(head, 'k takes one key (k a, k c-c) or one string (k "text")');
        return null;
      }
      case "w": {
        if (args.length !== 1 || args[0].t !== "DUR") {
          this.err(head, "w takes a duration like 200ms or 3s");
          return null;
        }
        return { kind: "Cmd", cmd: "wait", ms: dur(args[0].v), span: span };
      }
      case "wf": {
        // wf "text" 3s     wait for text anywhere in the accessibility tree
        // wf @"OK" 3s      wait for an element (same target grammar as click)
        var wa = new Args(this, args),
          what = null;
        if (wa.is("STR")) what = { kind: "Text", text: wa.next().v };
        else if (wa.is("AT")) what = this.target(wa, head);
        var to = wa.next();
        if (!what || !to || to.t !== "DUR" || !wa.done()) {
          this.err(head, 'wf takes "text" or @"element" and a timeout, e.g. wf "Save" 3s');
          return null;
        }
        return { kind: "Cmd", cmd: "waitfor", what: what, timeoutMs: dur(to.v), span: span };
      }
      default: {
        // built-ins: lock toggle, brainstorm, agent, voice, prefs, desktop 3 ...
        return {
          kind: "Cmd",
          cmd: "builtin",
          name: name,
          args: args.map(function (a) {
            return a.v;
          }),
          span: span,
        };
      }
    }
  };
  function dur(s) {
    return s.endsWith("ms") ? +s.slice(0, -2) : +s.slice(0, -1) * 1000;
  }

  // ── pointer targets ───────────────────────────────────────────────────────
  // A small cursor over the already-collected argument tokens.
  function Args(parser, toks) {
    this.p = parser;
    this.t = toks;
    this.i = 0;
  }
  Args.prototype.peek = function (k) {
    return this.t[this.i + (k || 0)];
  };
  Args.prototype.next = function () {
    return this.t[this.i++];
  };
  Args.prototype.done = function () {
    return this.i >= this.t.length;
  };
  Args.prototype.is = function (type, v, k) {
    var x = this.peek(k);
    return !!x && x.t === type && (v === undefined || x.v === v);
  };

  var BASES = { win: "window", cur: "cursor", screen: "screen" };
  var BUTTONS = {
    l: "left",
    r: "right",
    m: "middle",
    x1: "x1",
    x2: "x2",
    left: "left",
    right: "right",
    middle: "middle",
  };

  /** number | +number | -number | N% — returns {v, pct, rel} or null */
  function coord(a) {
    var sign = 1,
      tok = a.peek();
    if (!tok) return null;
    // '+' / '-' arrive as PLUS / KEY('-') tokens: the lexer keeps '-' a key
    // because in a pattern it IS one (F11 lives on it).
    if (tok.t === "PLUS" || (tok.t === "KEY" && tok.v === "-")) {
      var nx = a.peek(1);
      if (!nx || !(nx.t === "NUM" || (nx.t === "ID" && /^[0-9]+$/.test(nx.v)) || nx.t === "PCT"))
        return null;
      if (tok.v === "-") sign = -1;
      a.next();
      tok = a.peek();
    }
    if (tok.t === "NUM") {
      a.next();
      return { v: sign * +tok.v, pct: false };
    }
    if (tok.t === "ID" && /^[0-9]+$/.test(tok.v)) {
      a.next();
      return { v: sign * +tok.v, pct: false };
    }
    if (tok.t === "PCT") {
      a.next();
      return { v: sign * +tok.v.slice(0, -1), pct: true };
    }
    return null;
  }

  /**
   * target := NUM NUM                      absolute screen px
   *         | PCT PCT                      percent of primary screen
   *         | (win|cur|screen) c c         relative to a base; cur takes signed offsets
   *         | cur                          the pointer where it is
   *         | @NAME                        a saved mark
   *         | @"text" | @role:"text"       a UI element (accessibility tree), at its centre
   */
  P.target = function (a, verbTok) {
    var first = a.peek();
    if (!first) return null;
    if (first.t === "AT") {
      a.next();
      var n = a.peek();
      if (n && n.t === "STR") {
        a.next();
        return { kind: "Elem", name: n.v };
      }
      if (n && n.t === "ID" && a.is("COLON", undefined, 1)) {
        a.next();
        a.next();
        var str = a.next();
        if (!str || str.t !== "STR") {
          this.err(first, '@role: must be followed by a "name"');
          return null;
        }
        return { kind: "Elem", role: n.v, name: str.v };
      }
      if (n && n.t === "ID") {
        a.next();
        return { kind: "Mark", name: n.v };
      }
      this.err(first, '@ must be followed by "text", role:"text", or a mark name');
      return null;
    }
    if (first.t === "ID" && BASES[first.v]) {
      a.next();
      var x = coord(a),
        y = x ? coord(a) : null;
      if (!x && first.v === "cur") return { kind: "Cursor" };
      if (!x || !y) {
        this.err(first, first.v + " needs two coordinates, e.g. " + first.v + " 10 20");
        return null;
      }
      if (x.pct !== y.pct) this.err(first, "mix of px and % in one target");
      return { kind: "Rel", base: BASES[first.v], x: x.v, y: y.v, pct: x.pct };
    }
    var cx = coord(a);
    if (!cx) return null;
    var cy = coord(a);
    if (!cy) {
      this.err(verbTok, "a position needs two coordinates");
      return null;
    }
    if (cx.pct !== cy.pct) this.err(verbTok, "mix of px and % in one target");
    return { kind: "Abs", x: cx.v, y: cy.v, pct: cx.pct };
  };

  /** [mods-]button token like  l  r  s-l  c-s-r  — returns {button, mods} or null (not consumed) */
  function buttonSpec(a) {
    var t = a.peek();
    if (!t || (t.t !== "ID" && t.t !== "MODKEY")) return null;
    var parts = t.v.split("-"),
      b = parts.pop(),
      mods = parts;
    if (!BUTTONS[b]) return null;
    if (
      mods.some(function (m) {
        return !/^[cswa]$/.test(m);
      })
    )
      return null;
    a.next();
    return { button: BUTTONS[b], mods: mods };
  }

  P.pointerCommand = function (name, head, argToks, span) {
    var a = new Args(this, argToks),
      self = this;
    function rest() {
      if (!a.done())
        self.err(a.peek(), "unexpected " + JSON.stringify(a.peek().v) + " after " + name);
    }
    switch (name) {
      case "m":
      case "hover": {
        // legacy:  m x y [c]     -> move (+ click)
        var tgt = this.target(a, head);
        if (!tgt) {
          this.err(
            head,
            name + ' needs a target: x y, N% N%, win/cur/screen x y, @"text", or @mark',
          );
          return null;
        }
        var click = a.is("ID", "c");
        if (click) a.next();
        rest();
        return { kind: "Cmd", cmd: "move", target: tgt, click: click, span: span };
      }
      case "click":
      case "dbl": {
        var bs = buttonSpec(a) || { button: "left", mods: [] };
        var wantTarget = !(a.done() || a.is("ID", "x"));
        var t2 = wantTarget ? this.target(a, head) : null;
        if (wantTarget && !t2) {
          if (!a.done()) this.err(a.peek(), "not a target: " + JSON.stringify(a.peek().v));
          return null;
        }
        var times = name === "dbl" ? 2 : 1;
        if (a.is("ID") && /^x[0-9]+$/.test(a.peek().v)) {
          times = +a.next().v.slice(1);
        }
        rest();
        return {
          kind: "Cmd",
          cmd: "click",
          button: bs.button,
          mods: bs.mods,
          target: t2,
          times: times,
          span: span,
        };
      }
      case "md":
      case "mu": {
        var b2 = buttonSpec(a) || { button: "left", mods: [] };
        rest();
        return {
          kind: "Cmd",
          cmd: name === "md" ? "mousedown" : "mouseup",
          button: b2.button,
          mods: b2.mods,
          span: span,
        };
      }
      case "drag": {
        var from = this.target(a, head);
        if (!from) {
          this.err(head, "drag needs: drag <from> -> <to> [button]");
          return null;
        }
        if (!a.is("ARROW")) {
          this.err(head, 'drag needs "->" between the two targets');
          return null;
        }
        a.next();
        var to = this.target(a, head);
        if (!to) {
          this.err(head, 'drag needs a destination after "->"');
          return null;
        }
        var b3 = buttonSpec(a) || { button: "left", mods: [] };
        rest();
        return {
          kind: "Cmd",
          cmd: "drag",
          from: from,
          to: to,
          button: b3.button,
          mods: b3.mods,
          span: span,
        };
      }
      case "scroll": {
        var d = a.next();
        var dirs = { up: 1, down: 1, left: 1, right: 1 };
        if (!d || d.t !== "ID" || !dirs[d.v]) {
          this.err(head, "scroll needs a direction: up down left right");
          return null;
        }
        var amt = a.next(),
          unit = "lines",
          n = 3;
        if (amt && amt.t === "PX") {
          n = +amt.v.slice(0, -2);
          unit = "px";
        } else if (amt && (amt.t === "NUM" || (amt.t === "ID" && /^[0-9]+$/.test(amt.v)))) {
          n = +amt.v;
        } else if (amt) {
          a.i--;
        }
        var t3 = a.done() ? null : this.target(a, head);
        rest();
        return {
          kind: "Cmd",
          cmd: "scroll",
          dir: d.v,
          amount: n,
          unit: unit,
          target: t3,
          span: span,
        };
      }
      case "mark": {
        var nm = a.next();
        if (!nm || nm.t !== "ID") {
          this.err(head, "mark needs a name: mark foo  (then use @foo)");
          return null;
        }
        rest();
        return { kind: "Cmd", cmd: "mark", name: nm.v, span: span };
      }
    }
    return undefined;
  };

  // ── semantic checks ───────────────────────────────────────────────────────
  function check(ast, diags, src) {
    var seen = {};
    ast.items.forEach(function (b) {
      var key = normalize(b.pattern);
      if (seen[key]) {
        diags.push({
          level: "warn",
          msg: "duplicate pattern, earlier binding on line " + seen[key] + " wins",
          line: lineOf(src, b.span[0]),
          col: 1,
          pos: b.span[0],
          end: b.span[1],
        });
      } else seen[key] = lineOf(src, b.span[0]);
      walkKeys(b.pattern, function (k, g) {
        if (POINTER_KEYS[k.name]) {
          diags.push({
            level: "warn",
            msg:
              'pointer key "' +
              k.name +
              '" in a pattern: parses, but the engine has no mouse hook yet (WH_MOUSE_LL / CGEventTap mouse mask)',
            line: lineOf(src, k.span[0]),
            col: 1,
            pos: k.span[0],
            end: k.span[1],
          });
        }
      });
      var top = b.pattern.gestures;
      if (
        top.length &&
        top.every(function (g) {
          return (
            g.kind === "Tap" &&
            g.group.keys.length === 1 &&
            !TRIGGERS[g.group.keys[0].name] &&
            !MODIFIERS[g.group.keys[0].name]
          );
        })
      ) {
        diags.push({
          level: "warn",
          msg: "pattern has no hold and no trigger — this would fire on plain typing",
          line: lineOf(src, b.span[0]),
          col: 1,
          pos: b.pattern.span[0],
          end: b.pattern.span[1],
        });
      }
    });
  }
  function lineOf(src, pos) {
    return src.slice(0, pos).split("\n").length;
  }
  function walkKeys(pat, f) {
    pat.gestures.forEach(function (g) {
      g.group.keys.forEach(function (k) {
        f(k, g);
      });
      if (g.body) walkKeys(g.body, f);
    });
  }

  // Canonical text of a pattern, with commutative groups sorted so
  // (capslock+space) and (space+capslock) collide as duplicates.
  function normalize(pat, depth) {
    depth = depth || 0;
    return pat.gestures
      .map(function (g) {
        var ks = g.group.keys.map(function (k) {
          return k.name;
        });
        if (g.group.conn !== "->") ks = ks.slice().sort();
        var grp = ks.length > 1 ? "(" + ks.join(g.group.conn) + ")" : ks[0];
        if (g.kind === "Tap") return depth > 0 && ks.length === 1 ? grp : grp + ".";
        var body = normalize(g.body, depth + 1);
        return grp + "[" + (body ? " " + body + " " : " ") + (g.closed ? "]" : "");
      })
      .join(" ");
  }

  // ── edge IR (the trace form) ──────────────────────────────────────────────
  // Returns a list of variants: alternation `|` expands to one edge list per
  // alternative (capped), everything else is one list.
  function edges(pat, cap) {
    cap = cap || 8;
    var variants = [[]];
    pat.gestures.forEach(function (g) {
      var groupsChoices = expandGroup(g.group); // [[k,k], [k,k], ...]
      var next = [];
      variants.forEach(function (prefix) {
        groupsChoices.forEach(function (keys) {
          if (next.length >= cap) return;
          var seq = prefix.slice();
          if (g.kind === "Tap") {
            if (keys.length === 1) seq.push({ e: "tap", k: keys[0] });
            else {
              keys.forEach(function (k) {
                seq.push({ e: "down", k: k });
              });
              keys
                .slice()
                .reverse()
                .forEach(function (k) {
                  seq.push({ e: "up", k: k });
                });
            }
            next.push(seq);
          } else {
            keys.forEach(function (k) {
              seq.push({ e: "down", k: k });
            });
            var bodies = edges(g.body, cap);
            bodies.forEach(function (bodySeq) {
              if (next.length >= cap) return;
              var s2 = seq.concat(bodySeq);
              if (g.closed)
                keys
                  .slice()
                  .reverse()
                  .forEach(function (k) {
                    s2.push({ e: "up", k: k });
                  });
              next.push(s2);
            });
          }
        });
      });
      variants = next;
    });
    return variants;
  }
  function expandGroup(grp) {
    var names = grp.keys.map(function (k) {
      return k.name;
    });
    if (grp.conn === "|")
      return names.map(function (n) {
        return [n];
      });
    return [names];
  }
  function traceText(seq) {
    var out = [];
    for (var i = 0; i < seq.length; i++) {
      var x = seq[i],
        nx = seq[i + 1];
      if (x.e === "down" && nx && nx.e === "up" && nx.k === x.k) {
        out.push(x.k);
        i++;
        continue;
      }
      out.push(x.e === "down" ? "[" + x.k : x.e === "up" ? x.k + "]" : x.k);
    }
    return out.join(" ");
  }

  // ── public ────────────────────────────────────────────────────────────────
  function compile(src) {
    var p = new Parser(src);
    var ast = p.file();
    p.toks.forEach(function (t) {
      if (t.t === "ERR") p.err(t, t.v);
    });
    check(ast, p.diags, src);
    var out = ast.items.map(function (b) {
      var vs = edges(b.pattern);
      return { pattern: normalize(b.pattern), variants: vs.map(traceText), action: b.action };
    });
    p.diags.sort(function (a, b) {
      return a.pos - b.pos;
    });
    return { ast: ast, bindings: out, diagnostics: p.diags, tokens: p.toks };
  }

  var api = {
    lex: lex,
    compile: compile,
    normalize: normalize,
    edges: edges,
    traceText: traceText,
    KEY_ALIASES: KEY_ALIASES,
    NAMED_KEYS: NAMED_KEYS,
  };
  if (typeof module === "object" && module.exports) module.exports = api;
  else root.CLX = api;
})(typeof self !== "undefined" ? self : this);
