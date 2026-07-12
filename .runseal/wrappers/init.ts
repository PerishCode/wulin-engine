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
        "docs/adr/0003-native-workbench-control-plane.md",
        "docs/adr/0004-frame-artifact-contract.md",
        "experiments/0002-deterministic-visual-loop/README.md",
        "flavor.toml",
        "runseal.toml",
        "sidecar.toml",
        ".runseal/deno.json",
        ".runseal/deno.lock",
        ".runseal/hooks/pre-commit",
        ".runseal/wrappers/guard.ts",
        ".runseal/wrappers/gpu-lab.ts",
        ".runseal/wrappers/visual-loop.ts",
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
