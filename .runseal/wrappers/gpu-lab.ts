function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

async function run(command: string, args: string[]): Promise<void> {
    const status = await new Deno.Command(command, {
        args,
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) {
        fail(`gpu-lab: ${command} failed with exit code ${status.code}`);
    }
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :gpu-lab [correctness|benchmark] [gpu-lab options]");
    console.log("");
    console.log("Bootstrap and run Experiment 0001 with a canonical report path.");
    Deno.exit(0);
}

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) {
    fail("gpu-lab: RUNSEAL_PROFILE_PATH is not set");
}
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const args = [...Deno.args];
const mode = args[0] === "correctness" || args[0] === "benchmark" ? args.shift()! : "correctness";
if (args.includes("--mode")) {
    fail("gpu-lab: pass correctness or benchmark as the first positional argument");
}
if (!args.includes("--report")) {
    args.push("--report", `out/experiments/0001-gpu-lab/${mode}.json`);
}

await run("pwsh", [
    "-NoProfile",
    "-File",
    "experiments/0001-gpu-lab/scripts/bootstrap.ps1",
]);
await run("cargo", [
    "run",
    "--locked",
    "-p",
    "gpu-lab",
    "--release",
    "--",
    "--mode",
    mode,
    ...args,
]);
