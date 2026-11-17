// Build script: compiles the Slint UI files into the binary so the release
// executable is fully self-contained (no runtime .slint loading).
//
// slint-build 1.19 API: `compile` takes only the .slint path; style and
// resource embedding use defaults (the browser shell sets its own colors
// inline, so the built-in widget style does not affect the design).

fn main() {
    slint_build::compile("ui/browser.slint")
        .expect("Failed to compile Slint UI: ui/browser.slint");

    println!("cargo:rerun-if-changed=ui/browser.slint");
}
