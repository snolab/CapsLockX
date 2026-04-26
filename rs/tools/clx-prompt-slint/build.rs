fn main() {
    let cfg = slint_build::CompilerConfiguration::new().with_style("cupertino-dark".into());
    slint_build::compile_with_config("ui/prompt.slint", cfg).unwrap();
}
