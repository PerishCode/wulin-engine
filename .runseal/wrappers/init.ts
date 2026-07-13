function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

async function run(command: string, args: string[]): Promise<string> {
    const output = await new Deno.Command(command, {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    if (!output.success) {
        const error = new TextDecoder().decode(output.stderr).trim();
        fail(`init: ${command} ${args.join(" ")} failed${error ? `: ${error}` : ""}`);
    }
    return new TextDecoder().decode(output.stdout).trim();
}

async function requireFile(path: string): Promise<void> {
    try {
        const info = await Deno.stat(`${root}/${path}`);
        if (!info.isFile) {
            fail(`init: expected a file at ${path}`);
        }
    } catch {
        fail(`init: missing required file: ${path}`);
    }
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :init");
    console.log("");
    console.log("Validate stable project tools and install repository git hooks.");
    Deno.exit(0);
}
if (Deno.args.length > 0) {
    fail(`init: unexpected argument: ${Deno.args[0]}`);
}

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) {
    fail("init: RUNSEAL_PROFILE_PATH is not set");
}
const root = profilePath.replace(/[\\/][^\\/]+$/, "");

for (
    const file of [
        "AGENTS.md",
        "Cargo.lock",
        "Cargo.toml",
        "apps/workbench/Cargo.toml",
        "apps/workbench/src/capture.rs",
        "apps/workbench/src/main.rs",
        "apps/workbench/src/load.rs",
        "apps/workbench/src/perception.rs",
        "apps/workbench/src/rendering/mod.rs",
        "apps/workbench/src/rendering/gpu_capture.rs",
        "apps/workbench/src/rendering/load/pipeline.rs",
        "apps/workbench/src/rendering/load/renderer.rs",
        "apps/workbench/src/rendering/calibration/object_id_target.rs",
        "apps/workbench/src/rendering/renderer/mod.rs",
        "apps/workbench/src/rendering/calibration/scene_renderer.rs",
        "apps/workbench/src/rendering/meshlet_scene/skeletal/mod.rs",
        "apps/workbench/shaders/skeletal_scene.hlsl",
        "crates/animation-catalog/src/lib.rs",
        "apps/workbench/src/scene.rs",
        "apps/workbench/src/window.rs",
        "docs/adr/0003-native-workbench-control-plane.md",
        "docs/adr/0004-frame-artifact-contract.md",
        "docs/adr/0005-capture-collection-contract.md",
        "docs/adr/0006-spatial-and-depth-convention.md",
        "docs/adr/0007-object-id-perception-contract.md",
        "docs/adr/0008-region-addressed-gpu-work.md",
        "docs/adr/0009-resident-region-storage.md",
        "docs/adr/0010-asynchronous-region-publication.md",
        "docs/adr/0011-cooked-region-storage.md",
        "docs/adr/0012-gpu-meshlet-scene-execution.md",
        "docs/adr/0013-gpu-skeletal-crowd-execution.md",
        "experiments/0002-deterministic-visual-loop/README.md",
        "experiments/0003-spatial-calibration-scene/README.md",
        "experiments/0004-object-id-perception/README.md",
        "experiments/0005-gpu-region-compaction/README.md",
        "experiments/0006-resident-region-streaming/README.md",
        "experiments/0007-async-region-publication/README.md",
        "experiments/0008-cooked-region-io/README.md",
        "experiments/0009-gpu-meshlet-scene/README.md",
        "experiments/0010-gpu-skeletal-crowds/README.md",
        "flavor.toml",
        "runseal.toml",
        "sidecar.toml",
        "sidecar.benchmark.toml",
        ".runseal/deno.json",
        ".runseal/deno.lock",
        ".runseal/hooks/pre-commit",
        ".runseal/wrappers/guard.ts",
        ".runseal/wrappers/gpu-lab.ts",
        ".runseal/wrappers/object-id.ts",
        ".runseal/wrappers/region-load.ts",
        ".runseal/wrappers/resident-stream.ts",
        ".runseal/wrappers/async-region.ts",
        ".runseal/wrappers/cooked-region.ts",
        ".runseal/wrappers/meshlet-scene.ts",
        ".runseal/wrappers/visual-loop.ts",
        ".runseal/wrappers/spatial-scene.ts",
        ".runseal/wrappers/skeletal-crowds.ts",
        ".runseal/support/skeletal-crowds.ts",
        ".runseal/support/cooked-region.ts",
        ".runseal/wrappers/workbench.ts",
    ]
) {
    await requireFile(file);
}

console.log("==> stable toolchain");
for (
    const [tool, args] of [
        ["runseal", ["--version"]],
        ["flavor", ["--version"]],
        ["sidecar", ["--version"]],
        ["cargo", ["--version"]],
        ["deno", ["--version"]],
    ] as const
) {
    console.log(await run(tool, [...args]));
}

await run("git", ["config", "core.hooksPath", ".runseal/hooks"]);
const hooksPath = await run("git", ["config", "--get", "core.hooksPath"]);
console.log(`core.hooksPath = ${hooksPath}`);
console.log("development environment ready");
