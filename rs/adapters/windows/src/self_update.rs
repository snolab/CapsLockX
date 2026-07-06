//! Startup version check + zero-downtime self-rebuild for local git installs.
//!
//! On startup (background thread) when running from a git checkout with
//! `auto_rebuild` enabled:
//!   1. `git fetch` + `git pull --ff-only` — auto-upgrade to upstream (never
//!      discards uncommitted work; a conflicting pull is skipped, not forced).
//!   2. Compare the live `git rev-parse HEAD` against the commit this binary was
//!      built from (`CLX_GIT_HASH`, embedded by build.rs).
//!   3. If they differ, rebuild `cargo build -p capslockx-windows --release`
//!      WITHOUT taking hotkeys offline: the running clx.exe is renamed aside
//!      (allowed for a live image on Windows), the linker writes a fresh exe to
//!      the now-free path, the new binary is installed to the repo-root path the
//!      npm launcher runs, then the fresh binary is spawned and takes over
//!      (kill_previous terminates us). Hotkeys stay live the entire build.
//!
//! Nothing here runs for released / npm-downloaded binaries (no `.git`), and it
//! is a no-op when the build had no git (`CLX_GIT_HASH == "unknown"`).

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Don't flash a console window when spawning git/cargo from this GUI app.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Commit this binary was built from (build.rs), or "unknown".
pub const GIT_HASH: &str = env!("CLX_GIT_HASH");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Max self-update relaunch generations before we refuse further rebuilds.
const MAX_UPDATE_GEN: u32 = 3;

/// This process's position in a self-update relaunch chain (0 = launched normally).
fn current_gen() -> u32 {
    std::env::var("CLX_UPDATE_GEN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn short(hash: &str) -> &str {
    &hash[..hash.len().min(9)]
}

/// Human-readable version line: "CapsLockX v1.35.0 (a2a91ec0e) [rust/windows]".
pub fn version_string() -> String {
    format!(
        "CapsLockX v{PKG_VERSION} ({}) [rust/windows]",
        short(GIT_HASH)
    )
}

/// Handle `clx --version` / `clx version`. Prints to stdout (inherited from the
/// npm launcher / parent shell) and returns — caller should exit afterwards.
pub fn print_version() {
    println!("{}", version_string());
}

/// Append to the always-on update log (`%TEMP%\capslockx_update.log`) and to the
/// synchronous hook log, so the trail survives even with CLX_DEBUG off.
fn ulog(msg: &str) {
    if let Ok(tmp) = std::env::var("TEMP") {
        use std::io::Write as _;
        let path = PathBuf::from(format!(r"{tmp}\capslockx_update.log"));
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{msg}");
        }
    }
    crate::hook::debug_log_sync(&format!("[update] {msg}"));
}

/// Walk up from the running exe to find the enclosing **CapsLockX git checkout**.
///
/// This is the gate that keeps auto-pull/rebuild exclusive to a git install:
/// a directory qualifies only if it has BOTH a `.git` AND our source sentinel
/// (`rs/adapters/windows/Cargo.toml`, the very crate we'd rebuild). So:
///   - an npm-global / release-downloaded binary (no `.git`, no source) → None
///   - a released binary that merely happens to sit inside some *unrelated* git
///     repo → that ancestor lacks the sentinel, so we keep walking and return
///     None rather than pulling/rebuilding a foreign repository.
/// Only a genuine, buildable CapsLockX clone returns `Some`.
fn find_repo_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let mut dir = exe.parent()?.to_path_buf();
    loop {
        if dir.join(".git").exists() && is_capslockx_checkout(&dir) {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// True when `dir` is the root of a CapsLockX source tree we can rebuild.
fn is_capslockx_checkout(dir: &Path) -> bool {
    dir.join("rs")
        .join("adapters")
        .join("windows")
        .join("Cargo.toml")
        .exists()
}

/// A `git` command in `root`, configured to never block on interaction and to
/// fail fast on network stalls — critical for `fetch`/`pull`, which would
/// otherwise hang this background thread forever on an SSH prompt or dead link.
fn git_cmd(root: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(root)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        // Never prompt for HTTPS credentials.
        .env("GIT_TERMINAL_PROMPT", "0")
        // Never prompt for SSH passphrase / host key; give up quickly.
        .env(
            "GIT_SSH_COMMAND",
            "ssh -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=accept-new",
        )
        // Bail on slow/dead HTTPS transfers instead of hanging.
        .env("GIT_HTTP_LOW_SPEED_LIMIT", "1000")
        .env("GIT_HTTP_LOW_SPEED_TIME", "15");
    cmd
}

/// Run a git command in `root`; returns trimmed stdout on success, logging failures.
/// For local, instant queries (rev-parse, etc.) — do NOT use for network ops.
fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = git_cmd(root, args).output().ok()?;
    if !out.status.success() {
        ulog(&format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

enum Bounded {
    Ok,
    Failed,
    TimedOut,
    SpawnErr(String),
}

/// Run a git NETWORK command (fetch/pull) with a hard wall-clock deadline.
///
/// `ConnectTimeout` alone doesn't bound SSH once TCP connects, and a default
/// `git fetch` here recurses submodules (a large, slow pack pull) — so on
/// startup we force `--no-recurse-submodules` at the call site and cap total
/// time here, killing the entire git/ssh/index-pack process TREE on timeout so
/// nothing lingers. stdio is nulled: we only care about exit status, and piped
/// output could dead-lock the poll loop if a buffer fills.
fn git_net(root: &Path, args: &[&str], secs: u64) -> Bounded {
    let mut child = match git_cmd(root, args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Bounded::SpawnErr(e.to_string()),
    };
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Bounded::Ok
                } else {
                    Bounded::Failed
                };
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    kill_tree(child.id());
                    let _ = child.wait();
                    return Bounded::TimedOut;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(_) => return Bounded::Failed,
        }
    }
}

/// Force-kill a process and all its descendants (git spawns nested git + ssh +
/// index-pack; killing only the parent orphans them).
fn kill_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &pid.to_string()])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
}

/// Delete leftover `clx.exe.old-*` images from a previous self-update swap.
/// The old image is only deletable once its process has fully exited, so this
/// runs each startup and silently ignores files still locked.
pub fn cleanup_old_binaries() {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            dirs.push(d.to_path_buf());
        }
    }
    if let Some(root) = find_repo_root() {
        // Both swap targets: the repo-root path (npm launcher) and the build
        // output dir. A swap can leave a sidecar in either, and the surviving
        // instance may run from the other, so scan both regardless.
        dirs.push(root.join("rs").join("target").join("release"));
        dirs.push(root);
    }
    // De-dup so we don't scan the same dir twice (e.g. run from release dir).
    dirs.sort();
    dirs.dedup();
    for d in dirs {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in rd.flatten() {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with("clx.exe.old-") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}

/// Spawn the background version-check / auto-rebuild thread. Never blocks startup.
pub fn spawn_check(auto_rebuild: bool) {
    let _ = std::thread::Builder::new()
        .name("clx-self-update".into())
        .spawn(move || run_check(auto_rebuild));
}

fn run_check(auto_rebuild: bool) {
    ulog(&format!("startup {}", version_string()));

    // `CLX_SELF_UPDATE_FORCE=1` forces the rebuild+swap+relaunch path regardless
    // of whether the binary is behind HEAD — a manual "upgrade & rebuild now"
    // trigger (also how the swap path is exercised deterministically in tests).
    let forced = std::env::var("CLX_SELF_UPDATE_FORCE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    // Auto-pull/rebuild is exclusive to a git install: only proceed from a real
    // CapsLockX source clone. Released / npm-downloaded binaries stop here.
    let Some(root) = find_repo_root() else {
        ulog("not a CapsLockX git checkout (installed binary) — skipping auto-update");
        return;
    };
    if GIT_HASH == "unknown" && !forced {
        ulog("binary built without git metadata — skipping self-update");
        return;
    }
    if !auto_rebuild && !forced {
        ulog("auto_rebuild disabled in config — skipping");
        return;
    }
    // Runaway guard: each relaunch bumps CLX_UPDATE_GEN. Cap the chain so a
    // pathological loop (a forced flag that leaked into the child, HEAD racing
    // ahead every cycle, a build that never matches HEAD) can never spin
    // forever spawning processes.
    if current_gen() >= MAX_UPDATE_GEN {
        ulog(&format!(
            "self-update generation cap ({MAX_UPDATE_GEN}) reached — skipping to avoid a rebuild loop"
        ));
        return;
    }

    // ── 1. Auto-upgrade: fetch + fast-forward-only pull ──────────────────
    // --ff-only never rewrites history and refuses (leaving the tree intact)
    // rather than clobbering local edits, so it's safe with a dirty worktree.
    // --no-recurse-submodules keeps this a lightweight main-repo sync (a default
    // fetch here recursively pulls large submodule packs on every startup).
    // Each network op is hard-bounded so a stalled SSH can't leak processes.
    ulog("git fetch --no-recurse-submodules …");
    match git_net(&root, &["fetch", "--quiet", "--no-recurse-submodules"], 45) {
        Bounded::Ok => ulog("git fetch: ok"),
        Bounded::Failed => ulog("git fetch: failed (offline / no upstream?) — continuing"),
        Bounded::TimedOut => ulog("git fetch: timed out — killed tree, continuing"),
        Bounded::SpawnErr(e) => ulog(&format!("git fetch: spawn error {e} — continuing")),
    }
    match git_net(
        &root,
        &["pull", "--ff-only", "--quiet", "--no-recurse-submodules"],
        45,
    ) {
        Bounded::Ok => ulog("git pull --ff-only: ok"),
        Bounded::Failed => ulog("git pull --ff-only: skipped (not fast-forward / dirty)"),
        Bounded::TimedOut => ulog("git pull: timed out — killed tree, continuing"),
        Bounded::SpawnErr(e) => ulog(&format!("git pull: spawn error {e}")),
    }

    // ── 2. Is the binary behind the checkout? ────────────────────────────
    let Some(head) = git(&root, &["rev-parse", "HEAD"]) else {
        ulog("rev-parse HEAD failed — skipping");
        return;
    };
    if head == GIT_HASH && !forced {
        ulog(&format!("up to date (HEAD {})", short(&head)));
        return;
    }
    if forced && head == GIT_HASH {
        ulog(&format!(
            "CLX_SELF_UPDATE_FORCE set — rebuilding at HEAD {} anyway",
            short(&head)
        ));
    } else {
        ulog(&format!(
            "outdated: built from {} but HEAD is {} — rebuilding",
            short(GIT_HASH),
            short(&head)
        ));
    }

    // ── 3. Zero-downtime rebuild + swap + relaunch ───────────────────────
    if let Err(e) = rebuild_and_swap(&root) {
        ulog(&format!("self-update failed: {e}"));
    }
}

/// Rebuild in place while staying live, install the fresh binary, relaunch it.
///
/// Crash-safety: the path the npm launcher runs (`root_exe`) is a valid binary
/// at every instant except a sub-second window during the final copy. We never
/// rename it aside until *after* the build succeeds, so a mid-build death (e.g.
/// our controlling session tearing down) leaves a launchable clx behind. The
/// only path we free before building is `release_exe`, and only when we are
/// literally running from it.
fn rebuild_and_swap(root: &Path) -> Result<(), String> {
    let rs = root.join("rs");
    let release_exe = rs.join("target").join("release").join("clx.exe");
    let root_exe = root.join("clx.exe");
    let cur = std::env::current_exe().map_err(|e| e.to_string())?;

    // Only free a path before building if it IS the build output AND we occupy
    // it. Renaming a running image is permitted on Windows — our process keeps
    // executing from the renamed file. root_exe is deliberately left untouched
    // here so `clx` stays launchable throughout the (minutes-long) build.
    let running_is_build_target = cur == release_exe;
    let build_aside = if running_is_build_target {
        let aside = aside_path(&cur);
        let _ = std::fs::remove_file(&aside);
        std::fs::rename(&cur, &aside)
            .map_err(|e| format!("cannot free build path ({}): {e}", cur.display()))?;
        ulog(&format!(
            "freed build path, running from {}",
            aside.display()
        ));
        Some(aside)
    } else {
        None
    };

    // Long build — hotkeys keep working because this is a separate process from
    // our still-live hook. Captured so failures land in the update log.
    ulog("cargo build -p capslockx-windows --release … (hotkeys stay live)");
    let build = Command::new("cargo")
        .current_dir(&rs)
        .args(["build", "-p", "capslockx-windows", "--release"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let ok = matches!(&build, Ok(o) if o.status.success()) && release_exe.exists();
    if !ok {
        // Restore the previous binary so we remain launchable.
        if let Some(aside) = &build_aside {
            let _ = std::fs::remove_file(&cur);
            let _ = std::fs::rename(aside, &cur);
        }
        let detail = match build {
            Ok(o) if !o.status.success() => String::from_utf8_lossy(&o.stderr)
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .join(" | "),
            Ok(_) => "release exe missing after build".into(),
            Err(e) => e.to_string(),
        };
        return Err(format!(
            "cargo build failed — restored previous binary: {detail}"
        ));
    }
    ulog("build ok");

    // Install the fresh binary to the repo-root path the npm launcher runs.
    // This is the only moment root_exe is briefly absent (rename→copy, <1s).
    if release_exe != root_exe {
        if cur == root_exe {
            // We run from root_exe; free it (running image survives the rename).
            let aside = aside_path(&root_exe);
            let _ = std::fs::remove_file(&aside);
            if let Err(e) = std::fs::rename(&root_exe, &aside) {
                return Err(format!("cannot free repo-root exe: {e}"));
            }
        }
        match std::fs::copy(&release_exe, &root_exe) {
            Ok(_) => ulog(&format!("installed → {}", root_exe.display())),
            Err(e) => ulog(&format!("warn: could not install to repo root: {e}")),
        }
    }

    // Relaunch the fresh binary FULLY DETACHED from our console / process group /
    // job. This is the load-bearing safety property: whatever launched us (an
    // agent shell, a Job Object with kill-on-close, a shared console) must not be
    // coupled to the replacement's lifecycle, and the replacement must survive us
    // exiting. See the crash post-mortem in this file's module docs.
    let launch = if root_exe.exists() {
        &root_exe
    } else {
        &release_exe
    };
    spawn_detached(launch, root).map_err(|e| format!("relaunch: {e}"))?;
    ulog(&format!(
        "relaunched {} (detached) — exiting old instance",
        launch.display()
    ));

    // The renamed-aside old image is NOT matched by kill_previous's "clx.exe"
    // name filter, so the child will not terminate us — we must exit ourselves.
    // Give the child a moment to come up first.
    std::thread::sleep(std::time::Duration::from_millis(400));
    std::process::exit(0);
}

/// `<exe>.old-<pid>` sidecar path used to hold a running image out of the way.
fn aside_path(exe: &Path) -> PathBuf {
    exe.with_extension(format!("exe.old-{}", std::process::id()))
}

/// Spawn the replacement clx fully decoupled from our environment.
///
/// - `CREATE_BREAKAWAY_FROM_JOB`: escape any Job Object we're in (e.g. an agent
///   shell's job with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) so the new clx isn't
///   killed when that job closes, and its death can't cascade back to the job's
///   owner. Falls back to detached-only if the job forbids breakaway.
/// - `DETACHED_PROCESS`: no inherited console — no console control events flow
///   between the new clx and whatever terminal launched us.
/// - `CREATE_NEW_PROCESS_GROUP`: Ctrl+C / Ctrl+Break in the parent group can't
///   reach it.
fn spawn_detached(exe: &Path, cwd: &Path) -> std::io::Result<()> {
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
    let base = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;
    let next_gen = (current_gen() + 1).to_string();

    // Build the child env once: advance the generation counter and DROP the
    // force flag so a manual `CLX_SELF_UPDATE_FORCE=1` run rebuilds exactly once
    // and settles, rather than the flag propagating into an endless relaunch loop.
    let spawn_with = |flags: u32| {
        Command::new(exe)
            .current_dir(cwd)
            .env("CLX_UPDATE_GEN", &next_gen)
            .env_remove("CLX_SELF_UPDATE_FORCE")
            .creation_flags(flags)
            .spawn()
    };

    match spawn_with(base | CREATE_BREAKAWAY_FROM_JOB) {
        Ok(_) => Ok(()),
        Err(e) => {
            ulog(&format!(
                "breakaway spawn failed ({e}); retrying detached-only"
            ));
            spawn_with(base).map(|_| ())
        }
    }
}
