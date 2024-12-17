use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    println!("cargo:rerun-if-changed=assets/");
    std::fs::remove_dir_all("./static").unwrap_or(());
    std::fs::create_dir_all("./static/assets").unwrap();
    copy_dir(Path::new("assets"), Path::new("./static/assets"));

    build_wasm_pkg("game/", "./static/game");
    build_wasm_pkg("editor/", "./static/editor");
}

fn build_wasm_pkg(in_dir: &str, pkg_out_dir: &str) {
    println!("cargo:rerun-if-changed={}", in_dir);
    let in_dir = Path::new(in_dir);

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("in_dir");
    let profile = std::env::var("PROFILE").unwrap();

    // build the web client
    if !std::env::var("SKIP_CLIENT_BUILD").is_ok() {
        let mut build_wasm = Command::new("wasm-pack");
        build_wasm.arg("build");

        if profile == "release" {
            build_wasm.arg("--release");
        } else {
            build_wasm.arg("--debug");
        }

        build_wasm.arg("--out-dir").arg(&out_dir);
        build_wasm.arg("--target").arg("web");
        build_wasm.env("CARGO_TARGET_DIR", &out_dir.join("target"));
        build_wasm.current_dir(in_dir);
        let status = build_wasm.status().unwrap();

        assert!(status.success(), "Failed to build web client");
    }

    // copy client files to server
    let server_asset_dir = Path::new(pkg_out_dir);

    std::fs::create_dir_all(&server_asset_dir).unwrap();
    copy_dir(&out_dir, &server_asset_dir);
    copy_dir(&in_dir.join("static"), &server_asset_dir);
}

fn copy_dir(src: &Path, dest: &Path) {
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dest_path = dest.join(path.file_name().unwrap());
        if path.is_dir() {
            std::fs::create_dir(&dest_path).unwrap();
            copy_dir(&path, &dest_path);
        } else {
            std::fs::copy(&path, &dest_path).unwrap();
        }
    }
}
