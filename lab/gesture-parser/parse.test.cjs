/** node lab/gesture-parser/parse.test.cjs
 *
 * parse.js is a browser IIFE, and this repo is "type":"module", so it is loaded
 * by evaluating the very same source rather than importing a second copy.
 */
const fs = require("node:fs");
const vm = require("node:vm");
const sandbox = { self: {} };
sandbox.self.self = sandbox.self;
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(__dirname + "/parse.js", "utf8"), sandbox);
const { parse } = sandbox.self.CLXParse;

/** "c+ s+ 1+ 1- s- c-"  ->  edge list.  +=down  -=up */
const edges = (s) =>
  s
    .trim()
    .split(/\s+/)
    .map((tok) => ({
      name: tok.slice(0, -1),
      type: tok.slice(-1) === "+" ? "down" : "up",
    }));

let pass = 0,
  fail = 0;
function is(seq, expect, why, strict, conn) {
  const got = parse(edges(seq), strict, conn).text;
  const ok = got === expect;
  ok ? pass++ : fail++;
  console.log(
    `${ok ? "ok  " : "FAIL"}  ${why}\n      ${seq}\n      -> ${got}` +
      (ok ? "" : `\n      expected: ${expect}`),
  );
}

// ── the core gestures ───────────────────────────────────────────────────────
is(
  "capslock+ space+ 1+ 1- space- capslock-",
  "(capslock+space)[ 1 ]",
  "chord + digit = the proposed F1",
);
is(
  "space+ capslock+ 1+ 1- capslock- space-",
  "(space+capslock)[ 1 ]",
  "same gesture, other press order — groups are commutative",
);
is(
  "capslock+ space+ space- capslock-",
  "(capslock+space).",
  "chord tap: empty body, the lock toggle",
);
is(
  "space+ 3+ 3- space-",
  "space[ 3 ]",
  "single trigger + digit = virtual desktop (unchanged today)",
);
is(
  "space+ shift+ 3+ 3- shift- space-",
  "(space+shift)[ 3 ]",
  "single trigger + shift + digit = move window",
);
is(
  "alt+ capslock+ space+ 4+ 4- space- capslock- alt-",
  "(alt+capslock+space)[ 4 ]",
  "held modifier joins the group, emits Alt+F4",
);

is(
  "a+ b+ e+ e- b- a-",
  "(a+b)[ e ]",
  "any two keys group when one has a body — holding a+b and striking e",
);
is(
  "a+ b+ b- a-",
  "a[ b ]",
  "no body and no role to go on: stays nested rather than guessing a chord",
);
is(
  "space+ h+ x+ x- h- space-",
  "(space+h)[ x ]",
  "trigger plus a held letter, released together, is still a group",
);

// ── open holds ──────────────────────────────────────────────────────────────
is("capslock+ space+", "(capslock+space)[", "chord still down: an unclosed hold");
is(
  "capslock+ space+ 1+ 1-",
  "(capslock+space)[ 1",
  "F1 already fired on the digit key-down; ] is not needed",
);
is("alt+", "alt[", "a key pressed and never released — the stuck-modifier shape (B8)");

// ── ordering: the T1 failure ────────────────────────────────────────────────
is(
  "capslock+ alt+ space+ 1+ 1- space- alt- capslock-",
  "(capslock+alt+space)[ 1 ]",
  "T1 keys collapse to a group when released together",
);
is(
  "capslock+ alt+ space+ 1+ 1- space- alt- capslock-",
  "capslock[ alt[ space[ 1 ] ] ]",
  "T1 in strict mode: the press order the engine actually sees",
  true,
);

// ── partial release must NOT close the group ────────────────────────────────
is(
  "capslock+ space+ 1+ 1- space- 2+ 2- capslock-",
  "capslock[ space[ 1 ] 2 ]",
  "releasing one trigger leaves the nesting: ] needs the WHOLE group",
);

// ── sequences and taps ──────────────────────────────────────────────────────
is(
  "space+ h+ h- j+ j- space-",
  "space[ h j ]",
  "several taps in one hold, each firing on its own key-down",
);
is("space+ space-", "space.", "a bare trigger tap — emits a literal space");
is(
  "capslock+ space+ space- capslock- capslock+ space+ space- capslock-",
  "(capslock+space). (capslock+space).",
  "two gestures in a row stay separate",
);

// ── overlapping holds that cannot nest ──────────────────────────────────────
const cross = parse(edges("a+ b+ a- b-"));
const okCross = cross.crossed === true && cross.text === "(a+b).";
okCross ? pass++ : fail++;
console.log(
  `${okCross ? "ok  " : "FAIL"}  overlapping holds flagged as crossed (T4 shape)\n` +
    `      a+ b+ a- b-  ->  ${cross.text}  crossed=${cross.crossed}`,
);

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail ? 1 : 0);
