fn main() {
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    println!("cargo:rustc-link-lib=framework=WebKit");
    println!("cargo:rustc-link-lib=framework=IOKit");

    println!("cargo:rustc-link-lib=framework=Foundation");

    // CapsLock toggle helper — IOKit calls in ObjC to avoid Rust FFI ABI
    // subtleties (mach_task_self_ vs mach_task_self(), bool ABI, etc.).
    cc::Build::new()
        .file("capslock_iokit.m")
        .flag("-fobjc-arc")
        .compile("capslock_iokit");
}
