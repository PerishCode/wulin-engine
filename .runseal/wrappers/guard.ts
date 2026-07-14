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

async function requireWrapperSet(): Promise<void> {
    const names: string[] = [];
    for await (const entry of Deno.readDir(`${root}/.runseal/wrappers`)) {
        if (entry.isFile) names.push(entry.name);
    }
    names.sort();
    const expected = [
        "canonical-runtime.ts",
        "gpu-lab.ts",
        "guard.ts",
        "init.ts",
        "workbench.ts",
    ];
    if (JSON.stringify(names) !== JSON.stringify(expected)) {
        fail(`guard: Runseal wrapper set diverged: ${JSON.stringify(names)}`);
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
            "crate::(capture|inspect|perception|window)|apps/workbench|mods/|experiments/",
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

    const renderers = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "--fixed-strings",
            "pub struct Renderer",
            "--",
            "apps",
            "crates",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    const rendererLines = new TextDecoder().decode(renderers.stdout).trim().split(/\r?\n/)
        .filter((line) => line.length > 0);
    if (
        renderers.code !== 0 || rendererLines.length !== 1 ||
        !rendererLines[0].startsWith("crates/engine-runtime/src/rendering/renderer/mod.rs:")
    ) fail(`guard: canonical renderer ownership diverged: ${JSON.stringify(rendererLines)}`);

    const tree = await new Deno.Command("cargo", {
        args: ["tree", "-p", "engine-runtime", "--edges", "normal", "--prefix", "none"],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!tree.success) fail(`guard: engine-runtime dependency tree failed with ${tree.code}`);
    if (/^workbench\s/m.test(new TextDecoder().decode(tree.stdout))) {
        fail("guard: engine-runtime dependency tree points back to workbench");
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

    const timelineOwner = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "--fixed-strings",
            "pub(crate) struct PresentationTimeline",
            "--",
            "crates/engine-runtime/src",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    const timelineLines = new TextDecoder().decode(timelineOwner.stdout).trim().split(/\r?\n/)
        .filter((line) => line.length > 0);
    if (
        timelineOwner.code !== 0 || timelineLines.length !== 1 ||
        !timelineLines[0].startsWith("crates/engine-runtime/src/timeline.rs:")
    ) fail(`guard: presentation timeline ownership diverged: ${JSON.stringify(timelineLines)}`);

    const inputOwner = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "--fixed-strings",
            "pub(crate) struct HostInput",
            "--",
            "apps",
            "crates",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    const inputOwnerLines = new TextDecoder().decode(inputOwner.stdout).trim().split(/\r?\n/)
        .filter((line) => line.length > 0);
    if (
        inputOwner.code !== 0 || inputOwnerLines.length !== 1 ||
        !inputOwnerLines[0].startsWith("apps/workbench/src/input.rs:")
    ) fail(`guard: host input ownership diverged: ${JSON.stringify(inputOwnerLines)}`);

    const engineHostInput = await new Deno.Command("git", {
        args: [
            "grep",
            "--no-index",
            "-n",
            "-E",
            "HostInput|NativeMessage|PostedMessage|WM_(SYS)?KEY|WM_KILLFOCUS",
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

await requireWrapperSet();
await requireRuntimeBoundary();
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
    ".runseal/wrappers/workbench.ts",
    ".runseal/wrappers/canonical-runtime.ts",
    ".runseal/support/canonical-runtime.ts",
    ".runseal/support/host-input-replay.ts",
    ".runseal/support/cooked-gltf-presentation.ts",
    ".runseal/support/temporal-presentation.ts",
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
await forbiddenScan();
