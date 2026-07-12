use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const AGILITY_PACKAGE_VERSION: &str = "1.619.4";

fn main() {
    println!("cargo:rerun-if-changed=src/agility_exports.c");
    println!("cargo:rerun-if-env-changed=AGILITY_SDK_ROOT");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.join("../..").canonicalize().unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    cc::Build::new()
        .file(manifest_dir.join("src/agility_exports.c"))
        .warnings_into_errors(true)
        .compile("workbench_agility_exports");
    stage_agility_sdk(&repo_root, &out_dir);
}

fn stage_agility_sdk(repo_root: &Path, out_dir: &Path) {
    let package_root = env::var_os("AGILITY_SDK_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo_root.join(format!("out/deps/agility-sdk/{AGILITY_PACKAGE_VERSION}"))
        });
    let source = package_root.join("build/native/bin/x64");
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("unexpected Cargo OUT_DIR layout");
    let destination = profile_dir.join("D3D12");

    fs::create_dir_all(&destination).expect("failed to create the Agility SDK output directory");
    for file in ["D3D12Core.dll", "d3d12SDKLayers.dll"] {
        let from = source.join(file);
        if !from.is_file() {
            panic!(
                "Agility SDK {AGILITY_PACKAGE_VERSION} is missing. Run runseal :gpu-lab correctness first."
            );
        }
        fs::copy(&from, destination.join(file)).unwrap_or_else(|error| {
            panic!("failed to copy {}: {error}", from.display());
        });
    }
}
