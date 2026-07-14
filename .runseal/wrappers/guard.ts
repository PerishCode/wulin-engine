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
