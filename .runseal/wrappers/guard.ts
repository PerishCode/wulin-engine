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
]);
await run("flavor", "flavor", ["check", "--root", ".", "--config", "flavor.toml"]);
await run("sidecar doctor", "sidecar", ["doctor", "--config", "sidecar.toml"]);
await run("sidecar plan", "sidecar", ["plan", "--config", "sidecar.toml", "--format", "json"]);
