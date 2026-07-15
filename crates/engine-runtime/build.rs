use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const AGILITY_PACKAGE_VERSION: &str = "1.619.4";
const DEFAULT_DXC: &str = r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\dxc.exe";

fn main() {
    println!("cargo:rerun-if-changed=src/agility_exports.c");
    for source in [
        "skeletal_scene.hlsl",
        "surface_resolve.hlsl",
        "occlusion.hlsl",
        "terrain.hlsl",
    ] {
        println!("cargo:rerun-if-changed=shaders/{source}");
    }
    println!("cargo:rerun-if-env-changed=AGILITY_SDK_ROOT");
    println!("cargo:rerun-if-env-changed=DXC");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.join("../..").canonicalize().unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    cc::Build::new()
        .file(manifest_dir.join("src/agility_exports.c"))
        .warnings_into_errors(true)
        .compile("runtime_agility_exports");
    for (entry, output) in [
        ("reset_main", "skeletal_scene.reset.dxil"),
        ("cull_main", "skeletal_scene.cull.dxil"),
        ("compact_main", "skeletal_scene.compact.dxil"),
        ("pose_main", "skeletal_scene.pose.dxil"),
    ] {
        compile_shader(
            &manifest_dir,
            &out_dir,
            "skeletal_scene.hlsl",
            entry,
            "cs_6_6",
            output,
        );
    }
    for (entry, profile, output) in [
        ("reset_main", "cs_6_6", "terrain.reset.dxil"),
        ("seam_main", "cs_6_6", "terrain.seam.dxil"),
        ("lod_seam_main", "cs_6_6", "terrain.lod_seam.dxil"),
        ("as_main", "as_6_6", "terrain.as.dxil"),
        ("ms_main", "ms_6_6", "terrain.ms.dxil"),
        ("ps_main", "ps_6_6", "terrain.ps.dxil"),
    ] {
        compile_shader(
            &manifest_dir,
            &out_dir,
            "terrain.hlsl",
            entry,
            profile,
            output,
        );
    }
    for (entry, output) in [
        ("occlusion_classify_main", "occlusion.classify.dxil"),
        ("occlusion_prefix_main", "occlusion.prefix.dxil"),
        ("occlusion_scatter_main", "occlusion.scatter.dxil"),
        ("hiz_mip0_main", "occlusion.mip0.dxil"),
        ("hiz_reduce_main", "occlusion.reduce.dxil"),
    ] {
        compile_shader(
            &manifest_dir,
            &out_dir,
            "occlusion.hlsl",
            entry,
            "cs_6_6",
            output,
        );
    }
    for (entry, profile, output) in [
        ("as_main", "as_6_6", "surface_resolve.as.dxil"),
        ("ms_main", "ms_6_6", "surface_resolve.ms.dxil"),
        ("ps_main", "ps_6_6", "surface_resolve.ps.dxil"),
        ("shadow_as_main", "as_6_6", "surface_resolve.shadow_as.dxil"),
        ("shadow_ms_main", "ms_6_6", "surface_resolve.shadow_ms.dxil"),
        ("shade_main", "cs_6_6", "surface_resolve.shade.dxil"),
    ] {
        compile_shader(
            &manifest_dir,
            &out_dir,
            "surface_resolve.hlsl",
            entry,
            profile,
            output,
        );
    }
    stage_agility_sdk(&repo_root, &out_dir);
}

fn compile_shader(
    manifest_dir: &Path,
    out_dir: &Path,
    source_name: &str,
    entry: &str,
    profile: &str,
    output_name: &str,
) {
    let dxc = env::var_os("DXC")
        .map(PathBuf::from)
        .unwrap_or_else(|| DEFAULT_DXC.into());
    if !dxc.is_file() {
        panic!(
            "DXC was not found at {}. Set DXC to an x64 dxc.exe.",
            dxc.display()
        );
    }
    let source = manifest_dir.join("shaders").join(source_name);
    let output = out_dir.join(output_name);
    let result = Command::new(dxc)
        .args([
            "-T",
            profile,
            "-E",
            entry,
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
            "DXC failed for {entry}:\n{}\n{}",
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
                "Agility SDK {AGILITY_PACKAGE_VERSION} is missing. Run runseal :gpu-lab correctness first."
            );
        }
        fs::copy(&from, destination.join(file)).unwrap_or_else(|error| {
            panic!("failed to copy {}: {error}", from.display());
        });
    }
}
