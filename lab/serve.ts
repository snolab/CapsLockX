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

Bun.serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);
    let path = decodeURIComponent(url.pathname);

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
      if (await file.exists()) return new Response(file);
      return new Response("Not found", { status: 404 });
    }

    const file = Bun.file(abs);
    if (await file.exists()) return new Response(file);
    return new Response("Not found", { status: 404 });
  },
});

console.log(`lab server listening on http://localhost:${PORT}/lab/`);
