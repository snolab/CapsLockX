/**
 * CLX gesture notation — the pure parse core.
 *
 * Input:  an ordered list of key edges, {type:'down'|'up', name}
 * Output: a tree, and its rendering in the notation defined at
 *         /lab/chord-fn-row/#notation
 *
 *   (a->b)    group held together, pressed in that order  (observed)
 *   (a+b)     group held together, order asserted irrelevant  (a claim)
 *   X[ … ]    X held across the body
 *   X[        a hold that never closes (locked mode, or a leaked key)
 *   X.        tapped: down and straight back up, empty body
 *   k         a bare name inside a body is a tap
 *
 * Two renderings of the same capture:
 *   spec   (a->b)[ e ]     compact, a binding, lossy about release order
 *   trace  [a [b e b] a]   every edge, lossless, round-trips to key events
 *
 * No timers: structure comes from edge ORDER alone, which is exactly what the
 * engine reasons about. Loaded by index.html and by parse.test.js.
 */
(function (root) {
  "use strict";

  // Which release order your fingers happen to use is not something a person
  // controls, so nesting-vs-crossing is NOISE in captured input: the same
  // intended gesture would render two ways. The recognizer therefore
  // normalises every group to `+`. Nothing is lost -- the trace form still
  // records the exact edges, so observation and binding stay separated:
  //   trace  [a [b e b] a]   what happened, in full
  //   spec   (a+b)[ e ]      what it means as a binding
  // Render with `->` to see the order as pressed instead.
  var COMMUTATIVE = "+"; // normalised: a group, order irrelevant
  var OBSERVED = "->"; // as pressed: nesting keeps its order

  /** Pass 1 — containment tree. A key pressed while another is held nests inside it. */
  function buildTree(events) {
    var tree = { name: null, items: [], down: -1, up: null, isRoot: true };
    var stack = [tree];
    var open = Object.create(null);
    var crossed = false;

    events.forEach(function (ev, i) {
      if (ev.type === "down") {
        var node = { name: ev.name, items: [], down: i, up: null, group: null };
        stack[stack.length - 1].items.push(node);
        stack.push(node);
        open[ev.name] = node;
      } else {
        var n = open[ev.name];
        if (!n) return;
        n.up = i;
        delete open[ev.name];
        var idx = stack.indexOf(n);
        if (idx < 0) return;
        var above = stack.splice(idx); // n, plus anything still held inside it
        var orphans = above.slice(1);
        if (orphans.length) {
          // A CROSSING: n came up while o, pressed after it, is still down.
          // Neither interval contains the other, so the two keys were held
          // symmetrically -- positive evidence that their order did not
          // matter. They merge into one commutative group, `+`, which stays
          // open until o is released.
          crossed = true;
          var parent = stack[stack.length - 1];
          var o = orphans[0];
          o.group = (n.group || [n.name]).concat(o.group || [o.name]);
          o.name = null;
          o.conn = COMMUTATIVE;
          o.items = n.items
            .filter(function (x) {
              return x !== o;
            })
            .concat(o.items);
          var pi = parent.items.indexOf(n);
          if (pi >= 0) parent.items[pi] = o;
          else parent.items.push(o);
          stack.push(o);
          orphans.slice(1).forEach(function (x) {
            // 3+ way crossings: flatten
            var k = o.items.indexOf(x);
            if (k >= 0) o.items.splice(k, 1);
            parent.items.push(x);
            stack.push(x);
          });
        }
      }
    });
    tree.crossed = crossed;
    return tree;
  }

  /**
   * Role table, used only to break a genuine ambiguity (see `mergeable`).
   * `capslock- space- ` and `space+ 3+ 3- space-` are the SAME shape in the
   * edge stream, so when neither key shows a body there is no evidence in the
   * timeline to separate a chord from a trigger-plus-tap; the roles decide.
   */
  var GROUPABLE = {
    shift: 1,
    ctrl: 1,
    alt: 1,
    win: 1,
    capslock: 1,
    space: 1,
    insert: 1,
    scrolllock: 1,
    ralt: 1,
  };
  function roleGroupable(node, set) {
    return (node.group || [node.name]).every(function (k) {
      return set[k];
    });
  }

  /**
   * May `c` merge into its parent as a co-held group member?
   *
   * A child that has a BODY is demonstrably a hold: something was struck while
   * it was down, so it was never a tap. That is evidence straight from the
   * timeline and needs no key table and no clock — holding a and b and then
   * striking e gives `(a->b)[ e ]` for any keys at all.
   *
   * A childless child is the ambiguous case: `x[ y ]` could be a two-key chord
   * tapped, or x held while y was struck. Only there do the roles decide.
   */
  function mergeable(node, c, set) {
    if (c.items.length) return true;
    return roleGroupable(node, set) && roleGroupable(c, set);
  }

  /**
   * Pass 2 — collapse a single-child chain into a group when the two up-edges
   * are ADJACENT in the log, i.e. nothing happened between them. That is the
   * "] needs the whole group released" rule: a partial release leaves an event
   * in between, so the nesting survives instead of collapsing.
   */
  function collapse(node, strict, set) {
    set = set || GROUPABLE;
    node.items.forEach(function (c) {
      collapse(c, strict, set);
    });
    if (strict || node.isRoot) return node; // the root is a sequence, not a hold
    while (node.items.length === 1) {
      var c = node.items[0];
      var bothOpen = node.up === null && c.up === null;
      var adjacent = node.up !== null && c.up !== null && node.up === c.up + 1;
      if (!bothOpen && !adjacent) break;
      if (!mergeable(node, c, set)) break; // a bare tap never joins a group
      node.group = (node.group || [node.name]).concat(c.group || [c.name]);
      node.name = null;
      node.items = c.items;
    }
    return node;
  }

  /** Pass 3 — render one node. depth 0 is a whole gesture; deeper is a body item. */
  function render(node, depth, fmt) {
    var keys = node.group || [node.name];
    var g = keys.length > 1 ? fmt.group(keys, node.conn) : fmt.key(keys[0], true);
    if (!node.items.length) {
      if (node.up === null) return g + fmt.open(); // held, still open
      if (depth > 0 && keys.length === 1) return fmt.key(keys[0], false);
      return g + fmt.dot(); // tapped
    }
    var body = node.items
      .map(function (c) {
        return render(c, depth + 1, fmt);
      })
      .join(" ");
    return node.up === null
      ? g + fmt.open() + " " + body
      : g + fmt.open() + " " + body + " " + fmt.close();
  }

  /**
   * Group connector. A capture observes ONE press order and cannot know the
   * order is irrelevant, so the default is `->` (what happened). `+` is a
   * claim about the binding — that order does not matter — and belongs to a
   * human writing a spec, or to a capture that has seen both orders.
   */
  function textFmt(conn) {
    return {
      conn: conn || COMMUTATIVE,
      key: function (k) {
        return k;
      },
      group: function (ks, nodeConn) {
        return "(" + ks.join(nodeConn || this.conn) + ")";
      },
      open: function () {
        return "[";
      },
      close: function () {
        return "]";
      },
      dot: function () {
        return ".";
      },
    };
  }
  var TEXT = textFmt(COMMUTATIVE);

  function renderTree(tree, fmt) {
    return tree.items
      .map(function (c) {
        return render(c, 0, fmt || TEXT);
      })
      .join(" ");
  }

  /**
   * TRACE form — every edge written down, in order: `[a` is a going down,
   * `a]` is a coming up. A key that goes down and straight back up is written
   * bare, which stays lossless because a bare name can only mean an adjacent
   * down/up pair.
   *
   *   a v b v b ^ a ^   ->  [a b a]        b tapped inside a
   *   a v b v e v e ^ b ^ a ^  ->  [a [b e b] a]
   *   a v b v a ^ b ^   ->  [a [b a] b]    crossing: neither contains the other
   *
   * Unlike the spec form this is complete: it can write a crossing, it needs
   * no role table and no heuristic, and it round-trips back to the edge list —
   * which is what a keystroke EMITTER would consume.
   */
  function trace(events) {
    var out = [],
      skip = -1;
    events.forEach(function (ev, i) {
      if (i === skip) return;
      if (ev.type === "down") {
        var nx = events[i + 1];
        if (nx && nx.type === "up" && nx.name === ev.name) {
          // a tap
          out.push(ev.name);
          skip = i + 1;
        } else {
          out.push("[" + ev.name);
        }
      } else {
        out.push(ev.name + "]");
      }
    });
    return out.join(" ");
  }

  /** Convenience: edges -> notation string. `conn` defaults to `->`. */
  function parse(events, strict, conn) {
    var tree = collapse(buildTree(events), strict);
    return {
      tree: tree,
      text: renderTree(tree, textFmt(conn)),
      trace: trace(events),
      crossed: tree.crossed,
    };
  }

  var api = {
    buildTree: buildTree,
    collapse: collapse,
    render: render,
    renderTree: renderTree,
    parse: parse,
    trace: trace,
    TEXT: TEXT,
    textFmt: textFmt,
    GROUPABLE: GROUPABLE,
    OBSERVED: OBSERVED,
    COMMUTATIVE: COMMUTATIVE,
  };

  if (typeof module === "object" && module.exports) module.exports = api;
  else root.CLXParse = api;
})(typeof self !== "undefined" ? self : this);
