#!/usr/bin/env node
/**
 * .clx conformance runner — runs corpus.json against a compile() implementation.
 *
 *   node lab/clx-lang/parse.test.cjs                 # JS prototype (clx.js)
 *   node lab/clx-lang/parse.test.cjs --impl <file>   # any file exporting compile()
 *   node lab/clx-lang/parse.test.cjs --update        # rewrite expectations from the JS impl
 *   node lab/clx-lang/parse.test.cjs --only <name>   # substring filter
 *
 * The corpus is the spec-by-example: whatever parses .clx in the lab and in
 * the engine (rs/core/src/clx_lang.rs, planned) must produce the same
 * {bindings, variants, actions, diagnostics}. The Rust side can run the same
 * file by printing the canonical form below for each case.
 *
 * Canonical forms compared (order matters everywhere):
 *   bindings : [ "<line>: <normalized pattern> => <action text>" ]
 *   variants : [ [trace, trace, ...] ]            one list per binding
 *   diags    : [ "<level>@<line>: <message prefix>" ]  prefix match
 *
 * Diagnostics match on the message *prefix* written in the corpus, so wording
 * can be tightened without breaking the corpus, while the level and line are
 * exact.
 */
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const args = process.argv.slice(2);
const flag = (n) => args.includes(n);
const val = (n) => {
  const i = args.indexOf(n);
  return i >= 0 ? args[i + 1] : null;
};

const implPath = path.resolve(val("--impl") || path.join(__dirname, "clx.js"));
const corpusPath = path.join(__dirname, "corpus.json");

// clx.js is a browser-first UMD file and the repo's package.json says
// "type": "module", so require() would load it as ESM and choke on the
// footer. Evaluate it in a sandbox that looks like CommonJS instead.
function loadImpl(p) {
  const sb = { module: { exports: {} }, console };
  sb.exports = sb.module.exports;
  vm.runInNewContext(fs.readFileSync(p, "utf8"), sb, { filename: p });
  const api = sb.module.exports;
  if (typeof api.compile !== "function") throw new Error(p + " does not export compile()");
  return api;
}

// ── canonical action text ─────────────────────────────────────────────────
function tgt(t) {
  if (!t) return "";
  switch (t.kind) {
    case "Abs":
      return t.x + (t.pct ? "%" : "") + " " + t.y + (t.pct ? "%" : "");
    case "Rel":
      return (
        t.base +
        " " +
        (t.x >= 0 ? "+" : "") +
        t.x +
        (t.pct ? "%" : "") +
        " " +
        (t.y >= 0 ? "+" : "") +
        t.y +
        (t.pct ? "%" : "")
      );
    case "Cursor":
      return "cur";
    case "Elem":
      return "@" + (t.role ? t.role + ":" : "") + JSON.stringify(t.name);
    case "Mark":
      return "@" + t.name;
    case "Text":
      return JSON.stringify(t.text);
    default:
      return "?" + t.kind;
  }
}
function btn(c) {
  return (c.mods && c.mods.length ? c.mods.join("-") + "-" : "") + c.button;
}
function cmdText(c) {
  switch (c.cmd) {
    case "key":
      return "k " + (c.mods.length ? c.mods.join("-") + "-" : "") + c.key;
    case "type":
      return "k " + JSON.stringify(c.text);
    case "wait":
      return "w " + c.ms + "ms";
    case "waitfor":
      return "wf " + tgt(c.what) + " " + c.timeoutMs + "ms";
    case "move":
      return "m " + tgt(c.target) + (c.click ? " c" : "");
    case "click":
      return (
        "click " +
        btn(c) +
        (c.target ? " " + tgt(c.target) : "") +
        (c.times !== 1 ? " x" + c.times : "")
      );
    case "drag":
      return "drag " + tgt(c.from) + " -> " + tgt(c.to) + " " + btn(c);
    case "mousedown":
      return "md " + btn(c);
    case "mouseup":
      return "mu " + btn(c);
    case "scroll":
      return "scroll " + c.dir + " " + c.amount + c.unit + (c.target ? " " + tgt(c.target) : "");
    case "mark":
      return "mark " + c.name;
    case "builtin":
      return c.name + (c.args.length ? " " + c.args.join(" ") : "");
    default:
      return "?" + c.cmd;
  }
}
function actionText(a) {
  return "{ " + a.cmds.map(cmdText).join("; ") + " }";
}

function canonical(r) {
  return {
    bindings: r.bindings.map((b) => b.line + ": " + b.pattern + " => " + actionText(b.action)),
    variants: r.bindings.map((b) => b.variants),
    diags: r.diagnostics.map((d) => d.level + "@" + d.line + ": " + d.msg),
  };
}

// ── compare ───────────────────────────────────────────────────────────────
function same(exp, got) {
  const errs = [];
  const eq = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  if (!eq(exp.bindings, got.bindings)) errs.push(["bindings", exp.bindings, got.bindings]);
  if (!eq(exp.variants, got.variants)) errs.push(["variants", exp.variants, got.variants]);
  const de = exp.diags,
    dg = got.diags;
  let dOk = de.length === dg.length;
  for (let i = 0; dOk && i < de.length; i++) dOk = dg[i].startsWith(de[i]);
  if (!dOk) errs.push(["diags", de, dg]);
  return errs;
}

// ── main ──────────────────────────────────────────────────────────────────
const api = loadImpl(implPath);
const corpus = JSON.parse(fs.readFileSync(corpusPath, "utf8"));
const only = val("--only");
let pass = 0,
  fail = 0;
for (const c of corpus.cases) {
  if (only && !c.name.includes(only)) continue;
  let got;
  try {
    got = canonical(api.compile(c.src));
  } catch (e) {
    got = { bindings: ["CRASH: " + e.message], variants: [], diags: [] };
  }
  if (flag("--update")) {
    c.expect = got;
    continue;
  }
  const errs = same(c.expect, got);
  if (!errs.length) {
    pass++;
    continue;
  }
  fail++;
  console.log("✗ " + c.name + "\n    src: " + JSON.stringify(c.src));
  for (const [what, e, g] of errs) {
    console.log(
      "    " +
        what +
        "\n      expected: " +
        JSON.stringify(e) +
        "\n      got:      " +
        JSON.stringify(g),
    );
  }
}
if (flag("--update")) {
  fs.writeFileSync(corpusPath, JSON.stringify(corpus, null, 2) + "\n");
  console.log("corpus.json rewritten from " + path.relative(process.cwd(), implPath));
} else {
  console.log(
    (fail ? "✗ " : "✓ ") +
      pass +
      " passed, " +
      fail +
      " failed (" +
      path.relative(process.cwd(), implPath) +
      ")",
  );
  process.exit(fail ? 1 : 0);
}
