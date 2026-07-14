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
        if (!info.isFile) fail(`init: expected a file at ${path}`);
    } catch {
        fail(`init: missing required file: ${path}`);
    }
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :init");
    console.log("\nValidate the canonical project surface and install repository hooks.");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`init: unexpected argument: ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("init: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");

for (
    const file of [
        "AGENTS.md",
        "README.md",
        "Cargo.lock",
        "Cargo.toml",
        "apps/workbench/Cargo.toml",
        "apps/workbench/src/main.rs",
        "apps/workbench/src/inspect/protocol.rs",
        "crates/engine-runtime/Cargo.toml",
        "crates/engine-runtime/src/lib.rs",
        "crates/engine-runtime/src/runtime.rs",
        "crates/engine-runtime/src/timeline.rs",
        "crates/engine-runtime/src/rendering/composition/mod.rs",
        "crates/engine-runtime/src/streaming/objects/mod.rs",
        "crates/engine-runtime/src/streaming/terrain/mod.rs",
        "crates/region-format/src/global.rs",
        "crates/terrain-format/src/global.rs",
        "tools/region-cooker/src/main.rs",
        "tools/terrain-cooker/src/main.rs",
        "experiments/0031-canonical-runtime-convergence/README.md",
        "experiments/0032-authored-object-presentation/README.md",
        "experiments/0033-deterministic-temporal-presentation/README.md",
        "experiments/0034-cooked-gltf-geometry/README.md",
        "experiments/0035-cooked-gltf-material/README.md",
        "experiments/0036-cooked-gltf-skeletal-animation/README.md",
        "experiments/0037-source-duration-playback/README.md",
        "experiments/0038-camera-visible-directional-shadows/README.md",
        "experiments/0039-canonical-runtime-host-separation/README.md",
        "experiments/0040-runtime-frame-transaction/README.md",
        "assets/third-party/khronos-fox/README.md",
        "docs/adr/0035-authored-object-presentation.md",
        "docs/adr/0036-deterministic-temporal-presentation.md",
        "docs/adr/0037-cooked-gltf-geometry.md",
        "docs/adr/0038-cooked-gltf-material.md",
        "docs/adr/0039-cooked-gltf-skeletal-animation.md",
        "docs/adr/0040-source-duration-presentation-time.md",
        "docs/adr/0041-camera-visible-directional-shadows.md",
        "docs/adr/0042-canonical-runtime-host-separation.md",
        "docs/adr/0043-runtime-frame-transaction.md",
        "flavor.toml",
        "runseal.toml",
        "sidecar.toml",
        "sidecar.benchmark.toml",
        ".runseal/deno.json",
        ".runseal/deno.lock",
        ".runseal/hooks/pre-commit",
        ".runseal/wrappers/init.ts",
        ".runseal/wrappers/guard.ts",
        ".runseal/wrappers/gpu-lab.ts",
        ".runseal/wrappers/workbench.ts",
        ".runseal/wrappers/canonical-runtime.ts",
        ".runseal/support/canonical-runtime.ts",
        ".runseal/support/cooked-gltf-presentation.ts",
        ".runseal/support/temporal-presentation.ts",
    ]
) await requireFile(file);

console.log("==> stable toolchain");
for (
    const [tool, args] of [
        ["runseal", ["--version"]],
        ["flavor", ["--version"]],
        ["sidecar", ["--version"]],
        ["cargo", ["--version"]],
        ["deno", ["--version"]],
    ] as const
) console.log(await run(tool, [...args]));

await run("git", ["config", "core.hooksPath", ".runseal/hooks"]);
console.log(`core.hooksPath = ${await run("git", ["config", "--get", "core.hooksPath"])}`);
console.log("development environment ready");
