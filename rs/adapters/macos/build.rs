fn main() {
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    println!("cargo:rustc-link-lib=framework=WebKit");
    println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=AudioToolbox");

    // Compile ObjC exception catcher (for catching ObjC exceptions from Rust).
    cc::Build::new()
        .file("objc_try.m")
        .flag("-fobjc-arc")
        .compile("objc_try");
    println!("cargo:rustc-link-lib=framework=Foundation");
}
