// Build script: compiles the Slint UI files into the binary so the release
// executable is fully self-contained (no runtime .slint loading).

fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark")
        .embed_sources(slint_build::EmbedSourcesKind::All);

    slint_build::compile("ui/browser.slint", config)
        .expect("Failed to compile Slint UI: ui/browser.slint");

    println!("cargo:rerun-if-changed=ui/browser.slint");
}
