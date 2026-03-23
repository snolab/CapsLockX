#!/usr/bin/env node
// Cross-platform launcher for CapsLockX.
// Detects OS and architecture, then spawns the correct Rust binary.

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { chmod } from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { get as httpsGet } from "node:https";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { pipeline } from "node:stream/promises";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

// Map (platform, arch) → binary info.
const BINARIES = {
  "win32-x64": { name: "clx.exe", pkg: "capslockx-windows" },
  "darwin-arm64": { name: "clx", pkg: "capslockx-macos" },
  "darwin-x64": { name: "clx", pkg: "capslockx-macos" },
  "linux-x64": { name: "clx", pkg: "capslockx-linux" },
};

const key = `${process.platform}-${process.arch}`;
const info = BINARIES[key];

if (!info) {
  console.error(
    `[CapsLockX] Unsupported platform: ${process.platform}-${process.arch}`,
  );
  console.error(
    `[CapsLockX] Supported: ${Object.keys(BINARIES).join(", ")}`,
  );
  process.exit(1);
}

// Look for the binary in several locations.
const candidates = [
  join(root, info.name), // repo root (built or downloaded)
];

let binary = candidates.find((p) => existsSync(p));

// If not found locally, try to download from the latest GitHub release.
if (!binary) {
  const tag = await getLatestTag();
  if (tag) {
    const url = `https://github.com/snolab/CapsLockX/releases/download/${tag}/${info.name}`;
    const dest = join(root, info.name);
    console.error(`[CapsLockX] Downloading ${info.name} from ${tag}...`);
    try {
      await download(url, dest);
      if (process.platform !== "win32") {
        await chmod(dest, 0o755);
      }
      binary = dest;
      console.error(`[CapsLockX] Downloaded to ${dest}`);
    } catch (err) {
      console.error(`[CapsLockX] Download failed: ${err.message}`);
    }
  }
}

if (!binary) {
  console.error(`[CapsLockX] Binary not found for ${key}.`);
  console.error(`[CapsLockX] Build it with: cd rs && cargo build -p ${info.pkg} --release`);
  console.error(`[CapsLockX] Or download from: https://github.com/snolab/CapsLockX/releases`);
  process.exit(1);
}

// Spawn the binary, forwarding args and stdio.
const child = spawn(binary, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

child.on("error", (err) => {
  console.error(`[CapsLockX] Failed to start: ${err.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});

// ── Helpers ──────────────────────────────────────────────────────────────────

async function getLatestTag() {
  try {
    const res = await fetch(
      "https://api.github.com/repos/snolab/CapsLockX/releases/latest",
      { headers: { Accept: "application/vnd.github+json" } },
    );
    if (!res.ok) return null;
    const data = await res.json();
    return data.tag_name || null;
  } catch {
    return null;
  }
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const follow = (u) => {
      httpsGet(u, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          follow(res.headers.location);
          return;
        }
        if (res.statusCode !== 200) {
          reject(new Error(`HTTP ${res.statusCode}`));
          return;
        }
        const ws = createWriteStream(dest);
        res.pipe(ws);
        ws.on("finish", () => ws.close(resolve));
        ws.on("error", reject);
      }).on("error", reject);
    };
    follow(url);
  });
}
