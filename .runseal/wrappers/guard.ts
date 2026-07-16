import { requireContactHistoryRemoved } from "../support/guard/contact-removal.ts";
import { requireCanonicalOperatorIdentity } from "../support/guard/canonical-operator.ts";
import { requireLiveOperatorSurface } from "../support/guard/live-operator-surface.ts";
import { requireInputJournalRemoved } from "../support/guard/input-journal-removal.ts";
import { requirePresentationStatusRemoved } from "../support/guard/presentation-status-removal.ts";
import { requireSimulationHistoryRemoved } from "../support/guard/simulation-control-removal.ts";
import { requireTerrainHistoryRemoved } from "../support/guard/terrain-transaction-removal.ts";

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
    if (!status.success) fail(`guard: ${label} failed with exit code ${status.code}`);
}

async function requireSingleOwner(
    label: string,
    symbol: string,
    paths: string[],
    expectedPrefix: string,
): Promise<void> {
    const output = await new Deno.Command("git", {
        args: ["grep", "--no-index", "-n", "--fixed-strings", symbol, "--", ...paths],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    const lines = new TextDecoder().decode(output.stdout).trim().split(/\r?\n/)
        .filter((line) => line.length > 0);
    if (output.code !== 0 || lines.length !== 1 || !lines[0].startsWith(expectedPrefix)) {
        fail(`guard: ${label} ownership diverged: ${JSON.stringify(lines)}`);
    }
}

async function requireRuntimeBoundary(): Promise<void> {
    console.log("==> canonical runtime ownership");
    for (
        const path of [
            "apps/workbench/src/rendering",
            "apps/workbench/src/streaming",
            "apps/workbench/src/scene",
            "apps/workbench/src/load.rs",
            "apps/workbench/src/resident.rs",
            "apps/workbench/src/world.rs",
            "apps/workbench/shaders",
            "apps/workbench/build.rs",
            "apps/prototype/src/rendering",
            "apps/prototype/src/streaming",
            "apps/prototype/src/scene",
            "apps/prototype/src/bootstrap.rs",
            "apps/prototype/src/input.rs",
            "apps/prototype/src/window.rs",
            "apps/prototype/shaders",
            "apps/prototype/build.rs",
        ]
    ) {
        try {
            await Deno.stat(`${root}/${path}`);
            fail(`guard: runtime ownership remains under the workbench host: ${path}`);
        } catch (error) {
            if (!(error instanceof Deno.errors.NotFound)) throw error;
        }
    }

    const forbidden = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            "crate::(capture|inspect|perception|window)|reference[_-]host|apps/(workbench|prototype)|mods/|experiments/",
            "--",
            "crates/engine-runtime",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (forbidden.code === 0) {
        fail(
            `guard: engine runtime depends on host-owned source\n${
                new TextDecoder().decode(forbidden.stdout)
            }`,
        );
    }
    if (forbidden.code !== 1) {
        fail(`guard: runtime ownership scan failed with exit code ${forbidden.code}`);
    }

    await requireSingleOwner(
        "canonical renderer",
        "pub struct Renderer",
        ["apps", "crates"],
        "crates/engine-runtime/src/rendering/renderer/mod.rs:",
    );

    const tree = await new Deno.Command("cargo", {
        args: ["tree", "-p", "engine-runtime", "--edges", "normal", "--prefix", "none"],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!tree.success) fail(`guard: engine-runtime dependency tree failed with ${tree.code}`);
    if (/^(workbench|prototype|reference-host)\s/m.test(new TextDecoder().decode(tree.stdout))) {
        fail("guard: engine-runtime dependency tree points back to a host");
    }

    const rendererTimeAuthority = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            [
                "time_running",
                "automatic_time_advance_count",
                "manual_time_step_count",
                "time_wrap_count",
                "advance_presentation_frame",
                "pause_presentation_time",
                "resume_presentation_time",
                "set_presentation_time",
                "step_presentation_time",
                "presentation_time_json",
            ].join("|"),
            "--",
            "crates/engine-runtime/src/rendering",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (rendererTimeAuthority.code === 0) {
        fail(
            `guard: renderer retains presentation timeline authority\n${
                new TextDecoder().decode(rendererTimeAuthority.stdout)
            }`,
        );
    }
    if (rendererTimeAuthority.code !== 1) {
        fail(`guard: renderer time-authority scan failed with ${rendererTimeAuthority.code}`);
    }

    await requireSingleOwner(
        "presentation timeline",
        "pub(crate) struct PresentationTimeline",
        ["crates/engine-runtime/src"],
        "crates/engine-runtime/src/timeline/presentation.rs:",
    );
    await requireSingleOwner(
        "simulation schedule",
        "pub(crate) struct SimulationSchedule",
        ["crates/engine-runtime/src"],
        "crates/engine-runtime/src/timeline/simulation.rs:",
    );
    await requireSingleOwner(
        "host input",
        "pub struct HostInput",
        ["apps", "crates"],
        "crates/reference-host/src/input.rs:",
    );

    const hostTreeChecks = [
        ["workbench", false],
        ["prototype", true],
    ] as const;
    for (const [app, forbidWorkbench] of hostTreeChecks) {
        const appTree = await new Deno.Command("cargo", {
            args: ["tree", "-p", app, "--edges", "normal", "--prefix", "none"],
            cwd: root,
            stdout: "piped",
            stderr: "inherit",
        }).output();
        if (!appTree.success) fail(`guard: ${app} dependency tree failed with ${appTree.code}`);
        const dependencies = new TextDecoder().decode(appTree.stdout);
        if (!/^reference-host\s/m.test(dependencies) || !/^engine-runtime\s/m.test(dependencies)) {
            fail(`guard: ${app} does not consume the shared reference host and runtime`);
        }
        if (forbidWorkbench && /^workbench\s/m.test(dependencies)) {
            fail("guard: prototype depends on the diagnostic workbench");
        }
    }

    const prototypeDiagnostics = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            "InspectServer|inspect_socket|workbench::|arm_.*_gate|perception|CapturedFrame",
            "--",
            "apps/prototype",
            "sidecar.prototype.toml",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (prototypeDiagnostics.code === 0) {
        fail(
            `guard: prototype depends on diagnostic behavior\n${
                new TextDecoder().decode(prototypeDiagnostics.stdout)
            }`,
        );
    }
    if (prototypeDiagnostics.code !== 1) {
        fail(`guard: prototype diagnostic scan failed with ${prototypeDiagnostics.code}`);
    }

    const engineHostInput = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            "HostInput|NativeMessage|WM_(SYS)?KEY|WM_KILLFOCUS",
            "--",
            "crates/engine-runtime",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (engineHostInput.code === 0) {
        fail(
            `guard: engine runtime depends on host-native input\n${
                new TextDecoder().decode(engineHostInput.stdout)
            }`,
        );
    }
    if (engineHostInput.code !== 1) {
        fail(`guard: host input dependency scan failed with ${engineHostInput.code}`);
    }

    const engineBootstrap = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            "declarative-runtime-bootstrap|--bootstrap",
            "--",
            "crates/engine-runtime",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (engineBootstrap.code === 0) {
        fail(
            `guard: engine runtime depends on host bootstrap policy\n${
                new TextDecoder().decode(engineBootstrap.stdout)
            }`,
        );
    }
    if (engineBootstrap.code !== 1) {
        fail(`guard: host bootstrap dependency scan failed with ${engineBootstrap.code}`);
    }
}

async function requireCalibrationSurfaceRemoved(): Promise<void> {
    console.log("==> removed calibration compatibility surface");
    for (
        const path of [
            "apps/workbench/src/inspect/world_control.rs",
            "crates/engine-runtime/shaders/calibration.hlsl",
            "crates/engine-runtime/src/rendering/calibration",
            "crates/engine-runtime/src/rendering/renderer/modes.rs",
            "crates/engine-runtime/src/world.rs",
            "crates/engine-runtime/tests/private/scene.rs",
        ]
    ) {
        try {
            await Deno.stat(`${root}/${path}`);
            fail(`guard: removed calibration path returned: ${path}`);
        } catch (error) {
            if (!(error instanceof Deno.errors.NotFound)) throw error;
        }
    }

    const pattern = [
        "calibration-v1",
        "calibration_mode_active",
        "(^|[^A-Za-z0-9_])SplitPosition([^A-Za-z0-9_]|$)",
        "(^|[^A-Za-z0-9_])WorldSpace([^A-Za-z0-9_]|$)",
        "(^|[^A-Za-z0-9_])SceneRenderer([^A-Za-z0-9_]|$)",
        "(^|[^A-Za-z0-9_])ObjectIdTarget([^A-Za-z0-9_]|$)",
        "scene\\.list_objects",
        "world\\.(status|relocate|rebase|reset|probe)",
    ].join("|");
    const output = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            pattern,
            "--",
            "apps",
            "crates",
            "tools",
            ".runseal/wrappers/canonical-runtime.ts",
            ".runseal/wrappers/gpu-lab.ts",
            ".runseal/wrappers/init.ts",
            ".runseal/wrappers/workbench.ts",
            "flavor.toml",
            "runseal.toml",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (output.code === 0) {
        fail(
            `guard: removed calibration compatibility symbol found\n${
                new TextDecoder().decode(output.stdout)
            }`,
        );
    }
    if (output.code !== 1) {
        fail(`guard: removed calibration scan failed with exit code ${output.code}`);
    }
}

async function forbiddenScan(): Promise<void> {
    console.log("==> forbidden compatibility scan");
    const pattern = [
        '"objects\\.disable"',
        '"composition\\.(order|fixture|enable|disable)"',
        '"terrain\\.(global\\.schedule|schedule|enable|disable|lod\\.[^"]*)"',
        '"load\\.(configure|enable|disable|probe|status)"',
        '"resident\\.(stream|status)"',
        '"async\\.(schedule|status)"',
        '"cooked\\.(open|schedule|status)"',
        '"meshlet\\.(configure|enable|disable|status)"',
        '"skeletal\\.(configure|enable|disable|status)"',
        '"surface\\.(configure|enable|disable|status)"',
        '"occlusion\\.(enable|disable|reset)"',
        "PayloadPreparation",
        "TerrainLodSettings",
        "forced_lod",
        "format-V1",
        "schema-1",
        "schema-2",
        "WLRGN002",
        "--identity-order",
        "ordinal restoration",
    ].join("|");
    const output = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            pattern,
            "--",
            "apps",
            "crates",
            "tools",
            ".runseal/support",
            ".runseal/wrappers/workbench.ts",
            "flavor.toml",
            "runseal.toml",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (output.code === 0) {
        fail(
            `guard: forbidden compatibility symbol found\n${
                new TextDecoder().decode(output.stdout)
            }`,
        );
    }
    if (output.code !== 1) fail(`guard: forbidden scan failed with exit code ${output.code}`);
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :guard");
    console.log("\nRun the canonical repository validation path.");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`guard: unexpected argument: ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("guard: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");

await requireCanonicalOperatorIdentity(root, fail);
await requireLiveOperatorSurface(root, fail);
await requireInputJournalRemoved(root, fail);
await requireRuntimeBoundary();
await requireCalibrationSurfaceRemoved();
await requireContactHistoryRemoved(root, fail);
await requireTerrainHistoryRemoved(root, fail);
await requireSimulationHistoryRemoved(root, fail);
await requirePresentationStatusRemoved(root, fail);
await run("git diff check", "git", ["diff", "--check"]);
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
    ".runseal/wrappers/prototype.ts",
    ".runseal/wrappers/workbench.ts",
    ".runseal/wrappers/canonical-actor.ts",
    ".runseal/wrappers/canonical-prototype.ts",
    ".runseal/wrappers/canonical-runtime.ts",
    ".runseal/support/canonical-runtime.ts",
    ".runseal/support/object/nearest.ts",
    ".runseal/support/runtime-bootstrap.ts",
    ".runseal/support/prototype/host.ts",
    ".runseal/support/prototype/input.ts",
    ".runseal/support/prototype/object/observation.ts",
    ".runseal/support/prototype/object/gates.ts",
    ".runseal/support/prototype/presentation.ts",
    ".runseal/support/prototype/traversal.ts",
    ".runseal/support/actor/lifecycle.ts",
    ".runseal/support/actor/gpu.ts",
    ".runseal/support/actor/simulation.ts",
    ".runseal/support/terrain/query.ts",
    ".runseal/support/cooked-gltf-presentation.ts",
    ".runseal/support/temporal-presentation.ts",
]);
await run("resource acceptance tests", "deno", [
    "test",
    ".runseal/support/resource-acceptance_test.ts",
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
await run("sidecar bootstrap doctor", "sidecar", ["doctor", "--config", "sidecar.bootstrap.toml"]);
await run("sidecar bootstrap plan", "sidecar", [
    "plan",
    "--config",
    "sidecar.bootstrap.toml",
    "--format",
    "json",
]);
await run("sidecar prototype doctor", "sidecar", ["doctor", "--config", "sidecar.prototype.toml"]);
await run("sidecar prototype plan", "sidecar", [
    "plan",
    "--config",
    "sidecar.prototype.toml",
    "--format",
    "json",
]);
await forbiddenScan();
