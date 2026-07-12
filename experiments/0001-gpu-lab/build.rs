use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGILITY_PACKAGE_VERSION: &str = "1.619.4";
const AGILITY_SDK_VERSION: &str = "619";
const DEFAULT_DXC: &str = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\dxc.exe";

fn main() {
    println!("cargo:rerun-if-changed=shaders/fill.hlsl");
    println!("cargo:rerun-if-env-changed=DXC");
    println!("cargo:rerun-if-env-changed=AGILITY_SDK_ROOT");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.join("../..").canonicalize().unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    compile_agility_exports(&manifest_dir);
    compile_shader(&manifest_dir, &out_dir);
    stage_agility_sdk(&repo_root, &out_dir);

    println!("cargo:rustc-env=GPU_LAB_AGILITY_PACKAGE={AGILITY_PACKAGE_VERSION}");
    println!("cargo:rustc-env=GPU_LAB_AGILITY_SDK={AGILITY_SDK_VERSION}");
    println!("cargo:rustc-env=GPU_LAB_DXC_VERSION={}", dxc_version());
    println!(
        "cargo:rustc-env=GPU_LAB_RUSTC_VERSION={}",
        tool_version("rustc", "--version")
    );
    println!(
        "cargo:rustc-env=GPU_LAB_GIT_REVISION={}",
        git_revision(&repo_root)
    );
}

fn compile_agility_exports(manifest_dir: &Path) {
    println!("cargo:rerun-if-changed=src/agility_exports.c");
    cc::Build::new()
        .file(manifest_dir.join("src/agility_exports.c"))
        .warnings_into_errors(true)
        .compile("gpu_lab_agility_exports");
}

fn compile_shader(manifest_dir: &Path, out_dir: &Path) {
    let dxc = env::var_os("DXC")
        .map(PathBuf::from)
        .unwrap_or_else(|| DEFAULT_DXC.into());
    if !dxc.is_file() {
        panic!(
            "DXC was not found at {}. Set DXC to an x64 dxc.exe.",
            dxc.display()
        );
    }

    let source = manifest_dir.join("shaders/fill.hlsl");
    let output = out_dir.join("fill.dxil");
    let result = Command::new(&dxc)
        .args([
            "-T",
            "cs_6_6",
            "-E",
            "main",
            "-HV",
            "2021",
            "-O3",
            "-Qstrip_debug",
            "-Fo",
        ])
        .arg(&output)
        .arg(&source)
        .output()
        .expect("failed to launch DXC");

    if !result.status.success() {
        panic!(
            "DXC failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
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
                "Agility SDK {AGILITY_PACKAGE_VERSION} is missing. Run experiments/0001-gpu-lab/scripts/bootstrap.ps1 first."
            );
        }
        fs::copy(&from, destination.join(file)).unwrap_or_else(|error| {
            panic!("failed to copy {}: {error}", from.display());
        });
    }
}

fn dxc_version() -> String {
    let dxc = env::var_os("DXC")
        .map(PathBuf::from)
        .unwrap_or_else(|| DEFAULT_DXC.into());
    let output = Command::new(dxc)
        .arg("--version")
        .output()
        .expect("failed to query DXC");
    first_line(&output.stdout)
}

fn tool_version(tool: &str, argument: &str) -> String {
    let output = Command::new(tool)
        .arg(argument)
        .output()
        .expect("failed to query tool version");
    first_line(&output.stdout)
}

fn git_revision(repo_root: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .current_dir(repo_root)
        .output();
    output
        .ok()
        .filter(|value| value.status.success())
        .map(|value| first_line(&value.stdout))
        .unwrap_or_else(|| "unknown".into())
}

fn first_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .next()
        .unwrap_or("unknown")
        .trim()
        .to_owned()
}
