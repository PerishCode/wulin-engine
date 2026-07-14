function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

async function run(label: string, command: string, args: string[]): Promise<void> {
    console.log(`==> ${label}`);
    const status = await new Deno.Command(command, {
        args,
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) {
        fail(`guard: ${label} failed with exit code ${status.code}`);
    }
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :guard");
    console.log("");
    console.log("Run the canonical repository validation path.");
    Deno.exit(0);
}
if (Deno.args.length > 0) {
    fail(`guard: unexpected argument: ${Deno.args[0]}`);
}

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) {
    fail("guard: RUNSEAL_PROFILE_PATH is not set");
}
const root = profilePath.replace(/[\\/][^\\/]+$/, "");

await run("cargo fmt", "cargo", ["fmt", "--all", "--check"]);
await run("cargo clippy", "cargo", [
    "clippy",
    "--locked",
    "--workspace",
    "--all-targets",
    "--",
    "-D",
    "warnings",
]);
await run("cargo test", "cargo", ["test", "--locked", "--workspace", "--release"]);
await run("deno fmt", "deno", ["fmt", "--check", ".runseal"]);
await run("deno check", "deno", [
    "check",
    "--config",
    ".runseal/deno.json",
    "--lock",
    ".runseal/deno.lock",
    "--frozen=true",
    ".runseal/wrappers/init.ts",
    ".runseal/wrappers/guard.ts",
    ".runseal/wrappers/gpu-lab.ts",
    ".runseal/wrappers/visual-loop.ts",
    ".runseal/wrappers/workbench.ts",
    ".runseal/wrappers/resident-stream.ts",
    ".runseal/wrappers/async-region.ts",
    ".runseal/wrappers/skeletal-crowds.ts",
    ".runseal/wrappers/surface-resolve.ts",
    ".runseal/wrappers/occlusion.ts",
    ".runseal/wrappers/terrain.ts",
    ".runseal/wrappers/terrain-lod.ts",
    ".runseal/wrappers/composition.ts",
    ".runseal/wrappers/terrain-sampling.ts",
    ".runseal/wrappers/lod-composition.ts",
    ".runseal/wrappers/region-traversal.ts",
    ".runseal/wrappers/global-space.ts",
    ".runseal/wrappers/global-terrain.ts",
    ".runseal/wrappers/global-composition.ts",
    ".runseal/wrappers/global-traversal.ts",
    ".runseal/wrappers/signed-terrain-storage.ts",
    ".runseal/wrappers/camera-relative-terrain.ts",
    ".runseal/wrappers/canonical-object-composition.ts",
    ".runseal/wrappers/canonical-origin-rollover.ts",
    ".runseal/wrappers/canonical-traversal-prefetch.ts",
    ".runseal/wrappers/cooked-canonical-objects.ts",
    ".runseal/support/global-terrain.ts",
    ".runseal/support/global-composition.ts",
    ".runseal/support/global-traversal.ts",
    ".runseal/support/signed-terrain-storage.ts",
    ".runseal/support/camera-relative-terrain.ts",
    ".runseal/support/canonical-object-composition.ts",
    ".runseal/support/canonical-origin-rollover.ts",
    ".runseal/support/canonical-origin-rollover-evidence.ts",
    ".runseal/support/canonical-origin-rollover-scenarios.ts",
    ".runseal/support/canonical-traversal-prefetch.ts",
    ".runseal/support/canonical-traversal-prefetch-evidence.ts",
    ".runseal/support/canonical-traversal-prefetch-scenarios.ts",
    ".runseal/support/cooked-canonical-objects/mod.ts",
    ".runseal/support/workbench/composition.ts",
    ".runseal/support/workbench/terrain.ts",
    ".runseal/support/workbench/world.ts",
]);
await run("flavor", "flavor", ["check", "--root", ".", "--config", "flavor.toml"]);
await run("sidecar doctor", "sidecar", ["doctor", "--config", "sidecar.toml"]);
await run("sidecar plan", "sidecar", ["plan", "--config", "sidecar.toml", "--format", "json"]);
await run("sidecar benchmark doctor", "sidecar", [
    "doctor",
    "--config",
    "sidecar.benchmark.toml",
]);
await run("sidecar benchmark plan", "sidecar", [
    "plan",
    "--config",
    "sidecar.benchmark.toml",
    "--format",
    "json",
]);
