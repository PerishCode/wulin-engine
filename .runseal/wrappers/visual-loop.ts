function fail(message: string): never {
    throw new Error(message);
}

async function sidecar(args: string[]): Promise<unknown> {
    const output = await new Deno.Command("sidecar", {
        args: [...args, "--config", "sidecar.toml", "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    const stderr = decoder.decode(output.stderr).trim();
    if (!output.success) {
        fail(`visual-loop: sidecar ${args[0]} failed${stderr ? `: ${stderr}` : ""}`);
    }
    if (!stdout) return null;
    try {
        return JSON.parse(stdout);
    } catch {
        fail(`visual-loop: sidecar returned non-JSON output: ${stdout}`);
    }
}

async function lifecycle(command: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [command, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`visual-loop: sidecar ${command} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]) as Record<string, unknown>;
    if (response.ok !== true || typeof response.data !== "object" || response.data === null) {
        fail(`visual-loop: ${verb} returned an invalid response`);
    }
    return response.data as Record<string, unknown>;
}

function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) fail(`visual-loop: expected ${name} to be ${type}`);
    return value as T;
}

function object(owner: Record<string, unknown>, name: string): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`visual-loop: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

async function capture(id: string): Promise<Record<string, unknown>> {
    const manifest = await event("workbench.capture", { id });
    const image = object(manifest, "image");
    const artifacts = object(manifest, "artifacts");
    if (field<number>(image, "width", "number") !== 1280) fail(`${id}: width mismatch`);
    if (field<number>(image, "height", "number") !== 720) fail(`${id}: height mismatch`);
    if (field<boolean>(manifest, "launchedBySidecar", "boolean") !== true) {
        fail(`${id}: capture was not launched by Sidecar`);
    }
    const renderer = object(manifest, "renderer");
    if (field<string>(renderer, "format", "string") !== "R8G8B8A8_UNORM") {
        fail(`${id}: format mismatch`);
    }
    if (renderer.deviceRemovedReason !== null) fail(`${id}: device removal was reported`);
    for (const key of ["png", "manifest"]) {
        const relative = field<string>(artifacts, key, "string");
        const info = await Deno.stat(`${root}/${relative}`);
        if (!info.isFile || info.size === 0) fail(`${id}: missing ${key} artifact`);
    }
    return manifest;
}

function pixelHash(manifest: Record<string, unknown>): string {
    return field<string>(object(manifest, "image"), "pixelSha256", "string");
}

function pngHash(manifest: Record<string, unknown>): string {
    return field<string>(object(manifest, "image"), "pngSha256", "string");
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :visual-loop");
    console.log("");
    console.log("Run the canonical Experiment 0002 deterministic capture workload.");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`visual-loop: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("visual-loop: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const colorA = { rgba: [0.08, 0.42, 0.24, 1.0] };
const colorB = { rgba: [0.55, 0.08, 0.16, 1.0] };
const reportPath = `${root}/out/experiments/0002-deterministic-visual-loop/acceptance.json`;
let report: Record<string, unknown> | undefined;

await lifecycle("stop");
try {
    await Deno.remove(reportPath);
} catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) throw error;
}
try {
    await lifecycle("start");
    await event("workbench.pause");
    await event("workbench.set_clear_color", colorA);
    const colorA1 = await capture("color-a-1");
    const colorA2 = await capture("color-a-2");
    await event("workbench.set_clear_color", colorB);
    const colorB1 = await capture("color-b-1");

    const firstProcess = field<number>(colorA1, "processId", "number");
    if (field<number>(colorA2, "processId", "number") !== firstProcess) {
        fail("visual-loop: same-process captures used different process IDs");
    }
    if (pixelHash(colorA1) !== pixelHash(colorA2)) {
        fail("visual-loop: repeated Color A captures produced different pixel hashes");
    }
    if (pngHash(colorA1) !== pngHash(colorA2)) {
        fail("visual-loop: repeated Color A captures produced different PNG hashes");
    }
    if (pixelHash(colorA1) === pixelHash(colorB1)) {
        fail("visual-loop: Color A and Color B produced the same pixel hash");
    }

    await lifecycle("restart");
    await event("workbench.pause");
    await event("workbench.set_clear_color", colorA);
    const colorARestart = await capture("color-a-restart");
    const restartedProcess = field<number>(colorARestart, "processId", "number");
    if (restartedProcess === firstProcess) fail("visual-loop: restart preserved the process ID");
    if (pixelHash(colorA1) !== pixelHash(colorARestart)) {
        fail("visual-loop: Color A changed after Sidecar restart");
    }
    if (pngHash(colorA1) !== pngHash(colorARestart)) {
        fail("visual-loop: Color A PNG encoding changed after Sidecar restart");
    }

    report = {
        schemaVersion: 1,
        outcome: "pass",
        firstProcess,
        restartedProcess,
        colorA: { pixelSha256: pixelHash(colorA1), pngSha256: pngHash(colorA1) },
        colorB: { pixelSha256: pixelHash(colorB1), pngSha256: pngHash(colorB1) },
        captures: ["color-a-1", "color-a-2", "color-b-1", "color-a-restart"],
    };
} finally {
    await lifecycle("stop");
}

const status = await sidecar(["status"] as string[]) as Record<string, unknown>;
const runtime = object(status, "runtime");
if (field<boolean>(runtime, "running", "boolean")) fail("visual-loop: broker remains running");
const targets = status.targets;
if (!Array.isArray(targets)) fail("visual-loop: status targets are invalid");
for (const target of targets) {
    if (typeof target !== "object" || target === null || Array.isArray(target)) {
        fail("visual-loop: status target is invalid");
    }
    if (field<boolean>(target as Record<string, unknown>, "running", "boolean")) {
        fail("visual-loop: a target remains running");
    }
}
if (!report) fail("visual-loop: acceptance report was not produced");
await Deno.writeTextFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
