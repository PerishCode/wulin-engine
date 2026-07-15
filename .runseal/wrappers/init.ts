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
        "apps/prototype/Cargo.toml",
        "apps/prototype/src/actor.rs",
        "apps/prototype/src/camera.rs",
        "apps/prototype/src/locomotion.rs",
        "apps/prototype/src/main.rs",
        "apps/prototype/src/presentation.rs",
        "apps/prototype/src/time.rs",
        "apps/workbench/Cargo.toml",
        "apps/workbench/src/main.rs",
        "apps/workbench/src/inspect/protocol.rs",
        "apps/workbench/src/inspect/app/actor.rs",
        "crates/engine-runtime/Cargo.toml",
        "crates/engine-runtime/src/lib.rs",
        "crates/engine-runtime/src/runtime/mod.rs",
        "crates/engine-runtime/src/runtime/actor.rs",
        "crates/engine-runtime/src/runtime/motion_batch.rs",
        "crates/engine-runtime/src/runtime/simulation_actor.rs",
        "crates/engine-runtime/src/scene/mod.rs",
        "crates/engine-runtime/src/timeline/mod.rs",
        "crates/engine-runtime/src/rendering/renderer/actor_projection.rs",
        "crates/engine-runtime/src/rendering/composition/mod.rs",
        "crates/engine-runtime/src/rendering/meshlet_scene/skeletal/resources/mod.rs",
        "crates/engine-runtime/src/rendering/meshlet_scene/skeletal/resources/actor.rs",
        "crates/engine-runtime/src/streaming/objects/mod.rs",
        "crates/engine-runtime/src/streaming/terrain/mod.rs",
        "crates/reference-host/Cargo.toml",
        "crates/reference-host/src/activation.rs",
        "crates/reference-host/src/bootstrap.rs",
        "crates/reference-host/src/clock.rs",
        "crates/reference-host/src/input.rs",
        "crates/reference-host/src/window.rs",
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
        "experiments/0041-deterministic-host-input/README.md",
        "experiments/0042-declarative-runtime-bootstrap/README.md",
        "experiments/0043-thin-prototype-host/README.md",
        "experiments/0044-exact-canonical-terrain-query/README.md",
        "experiments/0053-retained-terrain-body-lifecycle/README.md",
        "experiments/0054-transactional-retained-body-advance/README.md",
        "experiments/0055-mandatory-terrain-transaction-cleanup/README.md",
        "experiments/0056-transactional-retained-body-batch/README.md",
        "experiments/0057-transactional-simulation-body-advance/README.md",
        "experiments/0058-bounded-host-elapsed-clock/README.md",
        "experiments/0059-bounded-host-activation/README.md",
        "experiments/0060-mandatory-simulation-control-cleanup/README.md",
        "experiments/0061-composed-host-time-admission/README.md",
        "experiments/0062-prototype-body-bootstrap/README.md",
        "experiments/0063-live-prototype-time-driver/README.md",
        "experiments/0064-retained-runtime-actor-authority/README.md",
        "experiments/0065-mandatory-canonical-operator-cleanup/README.md",
        "experiments/0066-bounded-actor-render-projection/README.md",
        "experiments/0067-self-contained-visible-record/README.md",
        "experiments/0068-frame-safe-actor-gpu-admission/README.md",
        "experiments/0069-prototype-gravity-admission/README.md",
        "experiments/0070-mandatory-actor-projection-cleanup/README.md",
        "experiments/0071-actor-relative-camera-anchor/README.md",
        "experiments/0072-transactional-actor-render-admission/README.md",
        "experiments/0073-typed-actor-render-backpressure/README.md",
        "experiments/0074-prototype-horizontal-locomotion/README.md",
        "experiments/0075-mandatory-prototype-readiness-cleanup/README.md",
        "experiments/0076-prototype-traversal-activation/README.md",
        "experiments/0077-transactional-locomotion-presentation/README.md",
        "experiments/0078-committed-locomotion-facing/README.md",
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
        "docs/adr/0044-normalized-host-input-journal.md",
        "docs/adr/0045-canonical-bootstrap-readiness.md",
        "docs/adr/0046-reference-platform-host.md",
        "docs/adr/0047-canonical-terrain-query.md",
        "docs/adr/0056-retained-terrain-body-lifecycle.md",
        "docs/adr/0057-transactional-retained-body-advance.md",
        "docs/adr/0058-retired-caller-owned-terrain-transactions.md",
        "docs/adr/0059-transactional-retained-body-batch.md",
        "docs/adr/0060-transactional-simulation-body-advance.md",
        "docs/adr/0061-bounded-host-elapsed-clock.md",
        "docs/adr/0062-bounded-win32-activation.md",
        "docs/adr/0063-retired-independent-simulation-controls.md",
        "docs/adr/0064-composed-host-time-admission.md",
        "docs/adr/0065-prototype-body-bootstrap.md",
        "docs/adr/0066-live-prototype-time-driver.md",
        "docs/adr/0067-retained-runtime-actor-authority.md",
        "docs/adr/0068-neutral-canonical-operator-identity.md",
        "docs/adr/0069-bounded-actor-render-projection.md",
        "docs/adr/0070-self-contained-visible-record.md",
        "docs/adr/0071-frame-safe-actor-gpu-admission.md",
        "docs/adr/0072-prototype-gravity-admission.md",
        "docs/adr/0073-retired-standalone-actor-projection.md",
        "docs/adr/0074-actor-relative-camera-mutation.md",
        "docs/adr/0075-transactional-actor-render-admission.md",
        "docs/adr/0076-typed-actor-render-backpressure.md",
        "docs/adr/0077-prototype-fixed-horizontal-locomotion.md",
        "docs/adr/0078-current-prototype-readiness-authority.md",
        "docs/adr/0079-prototype-traversal-activation.md",
        "docs/adr/0080-transactional-actor-presentation-command.md",
        "docs/adr/0081-committed-prototype-locomotion-facing.md",
        "flavor.toml",
        "runseal.toml",
        "sidecar.toml",
        "sidecar.benchmark.toml",
        "sidecar.bootstrap.toml",
        "sidecar.prototype.toml",
        ".runseal/deno.json",
        ".runseal/deno.lock",
        ".runseal/hooks/pre-commit",
        ".runseal/wrappers/init.ts",
        ".runseal/wrappers/guard.ts",
        ".runseal/wrappers/gpu-lab.ts",
        ".runseal/wrappers/workbench.ts",
        ".runseal/wrappers/canonical-actor.ts",
        ".runseal/wrappers/canonical-frame.ts",
        ".runseal/wrappers/canonical-prototype.ts",
        ".runseal/wrappers/canonical-resources.ts",
        ".runseal/wrappers/canonical-runtime.ts",
        ".runseal/support/canonical-frame.ts",
        ".runseal/support/canonical-runtime.ts",
        ".runseal/support/canonical-setup.ts",
        ".runseal/support/guard/simulation-control-removal.ts",
        ".runseal/support/guard/canonical-operator.ts",
        ".runseal/support/guard/terrain-transaction-removal.ts",
        ".runseal/support/host-input-replay.ts",
        ".runseal/support/runtime-bootstrap.ts",
        ".runseal/support/prototype/host.ts",
        ".runseal/support/prototype/input.ts",
        ".runseal/support/prototype/presentation.ts",
        ".runseal/support/prototype/traversal.ts",
        ".runseal/support/cooked-gltf-presentation.ts",
        ".runseal/support/temporal-presentation.ts",
        ".runseal/support/terrain/query.ts",
        ".runseal/support/actor/lifecycle.ts",
        ".runseal/support/actor/admission.ts",
        ".runseal/support/actor/gpu.ts",
        ".runseal/support/actor/simulation.ts",
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
