#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]

fn main() {
    println!("cargo::rustc-check-cfg=cfg(gles)");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        #[cfg(feature = "windows-manifest")]
        embed_resource();
    }
}

#[cfg(feature = "windows-manifest")]
fn embed_resource() {
    let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("resources/windows/gpui.manifest.xml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let rc_file = out_dir.join("gpui.rc");
    let manifest_path = manifest.display().to_string().replace('\\', "/");
    std::fs::write(&rc_file, format!("#define RT_MANIFEST 24\n1 RT_MANIFEST \"{}\"\n", manifest_path)).unwrap();
    embed_resource::compile(&rc_file, embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
