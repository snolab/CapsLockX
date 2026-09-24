/** node lab/clx-lang/parse.test.cjs
 *
 * clx.js is a browser IIFE, and this repo is "type":"module", so it is loaded
 * by evaluating the very same source rather than importing a second copy —
 * same trick as lab/gesture-parser/parse.test.cjs.
 *
 * Covers the `when` scope construct and the `js { }` action added alongside
 * §3 and §4 of index.html. The older constructs are exercised by the page's
 * own examples; this file is the conformance corpus for the new ones.
 */
const fs = require("node:fs");
const vm = require("node:vm");
const sandbox = { self: {} };
sandbox.self.self = sandbox.self;
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(__dirname + "/clx.js", "utf8"), sandbox);
const C = sandbox.self.CLX;

let pass = 0,
  fail = 0;

function check(why, src, fn) {
  const r = C.compile(src);
  let err = null;
  try {
    fn(r);
  } catch (e) {
    err = e;
  }
  err ? fail++ : pass++;
  console.log(`${err ? "FAIL" : "ok  "}  ${why}` + (err ? `\n      ${err.message}` : ""));
}

const eq = (got, want, what) => {
  const a = JSON.stringify(got),
    b = JSON.stringify(want);
  if (a !== b) throw new Error(`${what}: got ${a}, expected ${b}`);
};
const errors = (r) => r.diagnostics.filter((d) => d.level === "error");
const warns = (r) => r.diagnostics.filter((d) => d.level === "warn");

// ── when: basic scoping ─────────────────────────────────────────────────────
check(
  "block scope attaches to every binding inside",
  ['when app:"ONENOTE.EXE" {', "  clx[ d ] => k f1", "  clx[ e ] => k f2", "}"].join("\n"),
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(r.bindings.length, 2, "binding count");
    eq(
      r.bindings.map((b) => b.scopeText),
      ['app:"ONENOTE.EXE"', 'app:"ONENOTE.EXE"'],
      "scopes",
    );
  },
);

check("single-line scope needs no braces", 'when app:"ONENOTE.EXE" clx[ d ] => k f1', (r) => {
  eq(errors(r).length, 0, "errors");
  eq(r.bindings[0].scopeText, 'app:"ONENOTE.EXE"', "scope");
});

check("conjunction with +", 'when app:"X"+title:"*Y*" clx[ q ] => k esc', (r) => {
  eq(errors(r).length, 0, "errors");
  eq(
    r.bindings[0].scope.map((m) => m.on),
    ["app", "title"],
    "matcher kinds",
  );
  eq(r.bindings[0].scopeText, 'app:"X"+title:"*Y*"', "scope text");
});

check(
  "nested when ANDs with the outer predicate",
  [
    'when app:"ONENOTE.EXE" {',
    "  clx[ d ] => k f1",
    '  when title:"*search*" {',
    "    clx[ g ] => k enter",
    "  }",
    "}",
  ].join("\n"),
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(r.bindings.length, 2, "binding count");
    eq(r.bindings[1].scope.length, 2, "inner scope depth");
    eq(r.bindings[1].scopeText, 'app:"ONENOTE.EXE"+title:"*search*"', "inner scope");
  },
);

check("unscoped bindings keep an empty scope", "clx[ h ] => k left", (r) => {
  eq(r.bindings[0].scope, [], "scope");
  eq(r.bindings[0].scopeText, "", "scope text");
});

check(
  "scope closes: a binding after the block is global again",
  ['when app:"X" { clx[ d ] => k f1', "}", "clx[ h ] => k left"].join("\n"),
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(
      r.bindings.map((b) => b.scopeText),
      ['app:"X"', ""],
      "scopes",
    );
  },
);

// ── when: diagnostics ───────────────────────────────────────────────────────
check("unknown matcher is an error", 'when exe:"X" clx[ d ] => k f1', (r) => {
  if (!errors(r).some((d) => /unknown scope matcher/.test(d.msg)))
    throw new Error("expected an unknown-matcher error, got " + JSON.stringify(r.diagnostics));
});

check("matcher without a string is an error", "when app:ONENOTE clx[ d ] => k f1", (r) => {
  if (!errors(r).some((d) => /expected a quoted string/.test(d.msg)))
    throw new Error("expected a quoted-string error, got " + JSON.stringify(r.diagnostics));
});

check("when with neither brace nor binding is an error", 'when app:"X"', (r) => {
  if (!errors(r).some((d) => /needs either "\{" or a binding/.test(d.msg)))
    throw new Error("expected a dangling-when error, got " + JSON.stringify(r.diagnostics));
});

check(
  "unclosed when block is an error",
  ['when app:"X" {', "  clx[ d ] => k f1"].join("\n"),
  (r) => {
    if (!errors(r).some((d) => /unclosed "when \{" block/.test(d.msg)))
      throw new Error("expected an unclosed-block error, got " + JSON.stringify(r.diagnostics));
  },
);

// ── when: duplicate detection is scope-aware ────────────────────────────────
check(
  "same pattern in two different scopes is not a duplicate",
  ['when app:"A" clx[ d ] => k f1', 'when app:"B" clx[ d ] => k f2'].join("\n"),
  (r) => {
    eq(warns(r).filter((d) => /duplicate/.test(d.msg)).length, 0, "duplicate warnings");
  },
);

check(
  "same pattern in the same scope is a duplicate",
  ['when app:"A" clx[ d ] => k f1', 'when app:"A" clx[ d ] => k f2'].join("\n"),
  (r) => {
    eq(
      warns(r).filter((d) => /duplicate pattern in the same scope/.test(d.msg)).length,
      1,
      "duplicate warnings",
    );
  },
);

check(
  "matcher order does not change scope identity",
  ['when app:"A"+title:"T" clx[ d ] => k f1', 'when title:"T"+app:"A" clx[ d ] => k f2'].join("\n"),
  (r) => {
    eq(warns(r).filter((d) => /duplicate/.test(d.msg)).length, 1, "duplicate warnings");
  },
);

check(
  "a scoped binding does not collide with the global one",
  ["clx[ d ] => k f1", 'when app:"A" clx[ d ] => k f2'].join("\n"),
  (r) => {
    eq(warns(r).filter((d) => /duplicate/.test(d.msg)).length, 0, "duplicate warnings");
  },
);

// ── js { }: the body is opaque ──────────────────────────────────────────────
check("js body is captured raw", "clx[ l ] => js { await onenote.insertLink() }", (r) => {
  eq(errors(r).length, 0, "errors");
  eq(r.bindings[0].action.kind, "Script", "action kind");
  eq(r.bindings[0].action.code.trim(), "await onenote.insertLink()", "code");
});

check(
  "braces inside a js string do not end the body",
  'clx[ l ] => js { const s = "}"; f(s) }',
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(r.bindings[0].action.code.trim(), 'const s = "}"; f(s)', "code");
  },
);

check(
  "braces inside a js line comment do not end the body",
  ["clx[ l ] => js { f(); // a } here", "}"].join("\n"),
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(r.bindings[0].action.code.includes("// a } here"), true, "comment preserved");
  },
);

check(
  "braces inside a js block comment do not end the body",
  "clx[ l ] => js { f(); /* } */ g() }",
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(r.bindings[0].action.code.trim(), "f(); /* } */ g()", "code");
  },
);

check("nested js braces balance", "clx[ l ] => js { if (a) { b() } else { c() } }", (r) => {
  eq(errors(r).length, 0, "errors");
  eq(r.bindings[0].action.code.trim(), "if (a) { b() } else { c() }", "code");
});

check(
  "a js arrow function does not confuse the => of a binding",
  "clx[ l ] => js { xs.map(x => x + 1) }",
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(r.bindings.length, 1, "binding count");
    eq(r.bindings[0].action.code.trim(), "xs.map(x => x + 1)", "code");
  },
);

check(
  "a multi-line js body keeps later bindings parsing",
  [
    "clx[ l ] => js {",
    "  const t = await clx.clipboard.text();",
    "  await clx.type(t.toUpperCase());",
    "}",
    "clx[ h ] => k left",
  ].join("\n"),
  (r) => {
    eq(errors(r).length, 0, "errors");
    eq(r.bindings.length, 2, "binding count");
    eq(r.bindings[1].action.kind, "Block", "second action kind");
  },
);

check("unclosed js block is an error", "clx[ l ] => js { f(", (r) => {
  if (!errors(r).some((d) => /unclosed "js \{" block/.test(d.msg)))
    throw new Error("expected an unclosed-js error, got " + JSON.stringify(r.diagnostics));
});

check("empty js body warns", "clx[ l ] => js {  }", (r) => {
  if (!warns(r).some((d) => /empty "js \{ \}" body/.test(d.msg)))
    throw new Error("expected an empty-body warning, got " + JSON.stringify(r.diagnostics));
});

check(
  "a script on a pattern with no hold warns about suppression",
  'when app:"X" (a->b) => js { f() }',
  (r) => {
    if (!warns(r).some((d) => /cannot suppress the key/.test(d.msg)))
      throw new Error("expected a suppression warning, got " + JSON.stringify(r.diagnostics));
  },
);

check("a script under a hold does not warn about suppression", "clx[ l ] => js { f() }", (r) => {
  eq(warns(r).filter((d) => /cannot suppress/.test(d.msg)).length, 0, "suppression warnings");
});

// ── the two combined, as the lab page writes them ───────────────────────────
check(
  "the §3 worked example compiles clean",
  [
    "# bindings.clx",
    'when app:"ONENOTE.EXE" {',
    '  clx[ d ]  => k "{date}"',
    "  clx[ l ]  => js { await onenote.insertLink() }",
    "",
    '  when title:"*search*" {',
    "    clx[ g ] => k enter",
    "  }",
    "}",
  ].join("\n"),
  (r) => {
    eq(errors(r).length, 0, "errors: " + JSON.stringify(errors(r)));
    eq(r.bindings.length, 3, "binding count");
    eq(
      r.bindings.map((b) => b.scopeText),
      ['app:"ONENOTE.EXE"', 'app:"ONENOTE.EXE"', 'app:"ONENOTE.EXE"+title:"*search*"'],
      "scopes",
    );
  },
);

// ── regressions: none of this may disturb the existing language ─────────────
check(
  "the shipped-bindings example still compiles clean",
  [
    "(capslock+space)[ 1 ]   => k f1",
    "(capslock+space).       => lock toggle",
    "clx[ h ]                => k left",
    "capslock[ alt[ 4 ] ]    => k a-f4",
    'clx[ 9 ]   => { mark here; click @"OK"; m @here }',
  ].join("\n"),
  (r) => {
    eq(errors(r).length, 0, "errors: " + JSON.stringify(errors(r)));
    eq(r.bindings.length, 5, "binding count");
    eq(
      r.bindings.every((b) => b.scopeText === ""),
      true,
      "all global",
    );
  },
);

console.log(`\n${pass} passed, ${fail} failed`);
process.exit(fail ? 1 : 0);
