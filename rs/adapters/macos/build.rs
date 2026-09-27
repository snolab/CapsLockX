fn main() {
    // Keep the tray version aligned with the version used by package-mac.sh.
    let package_json =
        std::fs::read_to_string("../../../package.json").expect("failed to read root package.json");
    let version = package_json
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("\"version\"")?
                .split_once(':')?
                .1
                .trim()
                .trim_end_matches(',')
                .strip_prefix('"')?
                .strip_suffix('"')
        })
        .expect("failed to parse version from root package.json");
    println!("cargo:rustc-env=CLX_VERSION={version}");
    println!("cargo:rerun-if-changed=../../../package.json");

    // Published CI artifacts keep a clean version label. Local builds include
    // their git branch so developers can distinguish them from downloaded apps.
    let is_ci = std::env::var_os("CI").is_some() || std::env::var_os("GITHUB_ACTIONS").is_some();
    let build_source = if is_ci {
        String::new()
    } else {
        git_revision_label().unwrap_or_else(|| "git:unknown".to_string())
    };
    println!("cargo:rustc-env=CLX_BUILD_SOURCE={build_source}");
    println!("cargo:rerun-if-env-changed=CI");
    println!("cargo:rerun-if-env-changed=GITHUB_ACTIONS");
    println!("cargo:rerun-if-changed=../../../.git/HEAD");

    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    println!("cargo:rustc-link-lib=framework=WebKit");
    println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");
    println!("cargo:rustc-link-lib=framework=AVFoundation");
    println!("cargo:rustc-link-lib=framework=IOKit");

    // Compile ObjC exception catcher (for catching ObjC exceptions from Rust).
    cc::Build::new()
        .file("objc_try.m")
        .flag("-fobjc-arc")
        .compile("objc_try");
    println!("cargo:rustc-link-lib=framework=Foundation");

    // CapsLock toggle helper — IOKit calls in ObjC to avoid Rust FFI ABI
    // subtleties (mach_task_self_ vs mach_task_self(), bool ABI, etc.).
    cc::Build::new()
        .file("capslock_iokit.m")
        .flag("-fobjc-arc")
        .compile("capslock_iokit");
}

fn git_revision_label() -> Option<String> {
    let branch = git_output(&["branch", "--show-current"])?;
    if !branch.is_empty() {
        return Some(format!("git:{branch}"));
    }

    let revision = git_output(&["rev-parse", "--short", "HEAD"])?;
    (!revision.is_empty()).then(|| format!("git:{revision}"))
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg("../../..")
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}
