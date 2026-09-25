/**
 * Static file server for the CapsLockX lab reports.
 *
 * Run it through portless so the reports get a stable, portless URL:
 *
 *   npx portless capslockx bun lab/serve.ts
 *   -> https://capslockx.localhost/lab/<report-name>
 *
 * portless hands us a port in $PORT; fall back to 4550 when run bare.
 */
import { readdir, stat } from "node:fs/promises";
import { join, normalize, resolve } from "node:path";

const LAB_ROOT = resolve(import.meta.dir);
const PORT = Number(process.env.PORT ?? 4550);

// Dev server: never let the browser keep a stale clx.js / run.js next to a
// fresh index.html. Without this, edits look like they "didn't take".
const NO_CACHE = { "cache-control": "no-store" };

/** Resolve a URL path under lab/, refusing anything that escapes the root. */
function safeJoin(rel: string): string | null {
  const p = resolve(join(LAB_ROOT, normalize(rel)));
  return p === LAB_ROOT || p.startsWith(LAB_ROOT + "\\") || p.startsWith(LAB_ROOT + "/") ? p : null;
}

async function indexPage(): Promise<Response> {
  const entries = await readdir(LAB_ROOT, { withFileTypes: true });
  const dirs = entries
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .sort();
  const items = dirs.map((d) => `<li><a href="/lab/${d}/">${d}</a></li>`).join("\n");
  return new Response(
    `<!doctype html><meta charset="utf-8"><title>CapsLockX lab</title>
<style>body{font:16px/1.6 system-ui,sans-serif;max-width:40rem;margin:4rem auto;padding:0 1rem}
a{color:#2563eb}h1{font-size:1.4rem}</style>
<h1>CapsLockX lab</h1><ul>${items}</ul>`,
    { headers: { "content-type": "text/html; charset=utf-8" } },
  );
}

// ── keyboard hand-off ────────────────────────────────────────────────────────
//
// An agent testing clx has to press keys, and injected keys land in whatever
// window has focus — which, on a machine someone is still using, has been their
// editor. So a test starts with the user handing the keyboard over from
// lab/keyboard-handoff: the browser window takes focus, keystrokes land there,
// and the agent reads what arrived from disk instead of asking for a copy-paste.
//
// State lives in one file rather than in memory so the agent can poll it without
// going through HTTP, and so a server restart mid-test is obvious.
const HANDOFF_FILE = resolve(LAB_ROOT, "..", "tmp", "keyboard-handoff.json");

type HandoffState = {
  /** idle -> running (user pressed Start) -> released (agent finished). */
  state: "idle" | "running" | "released";
  label: string;
  startedAt: number | null;
  releasedAt: number | null;
  /** Keystrokes seen by the page, to cross-check against clx's own hook log. */
  events: unknown[];
  /** Set by the agent so the page can tell the user the keyboard is theirs. */
  note: string;
};

const IDLE: HandoffState = {
  state: "idle",
  label: "",
  startedAt: null,
  releasedAt: null,
  events: [],
  note: "",
};

async function readHandoff(): Promise<HandoffState> {
  try {
    return { ...IDLE, ...(await Bun.file(HANDOFF_FILE).json()) };
  } catch {
    return { ...IDLE };
  }
}

async function writeHandoff(next: HandoffState): Promise<void> {
  await Bun.write(HANDOFF_FILE, JSON.stringify(next, null, 2));
}

function json(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json", ...NO_CACHE },
  });
}

async function handoffApi(req: Request, action: string): Promise<Response> {
  const current = await readHandoff();

  if (action === "current") return json(current);

  const body = req.method === "POST" ? await req.json().catch(() => ({})) : {};

  if (action === "start") {
    const next: HandoffState = {
      state: "running",
      label: String((body as any).label ?? ""),
      startedAt: Date.now(),
      releasedAt: null,
      events: [],
      note: "",
    };
    await writeHandoff(next);
    return json(next);
  }

  if (action === "events") {
    // Bounded: a stuck test must not grow this file without limit.
    const incoming = Array.isArray((body as any).events) ? (body as any).events : [];
    current.events = [...current.events, ...incoming].slice(-2000);
    await writeHandoff(current);
    return json({ ok: true, count: current.events.length });
  }

  if (action === "release") {
    current.state = "released";
    current.releasedAt = Date.now();
    current.note = String((body as any).note ?? current.note);
    await writeHandoff(current);
    return json(current);
  }

  if (action === "reset") {
    await writeHandoff({ ...IDLE });
    return json({ ...IDLE });
  }

  return new Response("Unknown action", { status: 404 });
}

Bun.serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);
    let path = decodeURIComponent(url.pathname);

    if (path.startsWith("/api/handoff/")) {
      return handoffApi(req, path.slice("/api/handoff/".length).replace(/\/$/, ""));
    }

    if (path === "/" || path === "/lab" || path === "/lab/") {
      if (path === "/") return Response.redirect("/lab/", 302);
      return indexPage();
    }
    if (!path.startsWith("/lab/")) return new Response("Not found", { status: 404 });

    const rel = path.slice("/lab/".length);
    const abs = safeJoin(rel);
    if (!abs) return new Response("Forbidden", { status: 403 });

    // Directory -> index.html (redirect first so relative links resolve).
    const info = await stat(abs).catch(() => null);
    if (info?.isDirectory()) {
      if (!path.endsWith("/")) return Response.redirect(path + "/", 302);
      const file = Bun.file(join(abs, "index.html"));
      if (await file.exists()) return new Response(file, { headers: NO_CACHE });
      return new Response("Not found", { status: 404 });
    }

    const file = Bun.file(abs);
    if (await file.exists()) return new Response(file, { headers: NO_CACHE });
    return new Response("Not found", { status: 404 });
  },
});

console.log(`lab server listening on http://localhost:${PORT}/lab/`);
