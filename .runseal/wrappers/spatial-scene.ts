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
        fail(`spatial-scene: sidecar ${args[0]} failed${stderr ? `: ${stderr}` : ""}`);
    }
    if (!stdout) return null;
    try {
        return JSON.parse(stdout);
    } catch {
        fail(`spatial-scene: sidecar returned non-JSON output: ${stdout}`);
    }
}

async function lifecycle(command: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [command, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`spatial-scene: sidecar ${command} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]) as Record<string, unknown>;
    if (response.ok !== true || typeof response.data !== "object" || response.data === null) {
        fail(`spatial-scene: ${verb} returned an invalid response`);
    }
    return response.data as Record<string, unknown>;
}

function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) fail(`spatial-scene: expected ${name} to be ${type}`);
    return value as T;
}

function object(owner: Record<string, unknown>, name: string): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`spatial-scene: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) fail(`spatial-scene: expected ${name} to be an array`);
    return value;
}

async function capture(id: string): Promise<Record<string, unknown>> {
    const manifest = await event("workbench.capture", {
        id,
        collection: "0003-spatial-calibration-scene",
    });
    const image = object(manifest, "image");
    if (field<number>(image, "width", "number") !== 1280) fail(`${id}: width mismatch`);
    if (field<number>(image, "height", "number") !== 720) fail(`${id}: height mismatch`);
    if (field<number>(image, "differentPixelCount", "number") <= 100_000) {
        fail(`${id}: calibration scene covers too few non-reference pixels`);
    }
    const renderer = object(manifest, "renderer");
    if (renderer.deviceRemovedReason !== null) fail(`${id}: device removal was reported`);
    const spatial = object(manifest, "spatial");
    if (field<string>(spatial, "sceneRevision", "string") !== "calibration-v1") {
        fail(`${id}: scene revision mismatch`);
    }
    const depth = object(spatial, "depth");
    if (field<boolean>(depth, "reverseZ", "boolean") !== true) {
        fail(`${id}: reverse-Z is not declared`);
    }
    if (field<string>(depth, "comparison", "string") !== "GREATER") {
        fail(`${id}: depth comparison mismatch`);
    }
    const artifacts = object(manifest, "artifacts");
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

function camera(manifest: Record<string, unknown>): Record<string, unknown> {
    return object(object(manifest, "spatial"), "camera");
}

function expectVector(actual: unknown[], expected: number[], label: string): void {
    if (
        actual.length !== expected.length ||
        actual.some((value, index) => value !== expected[index])
    ) {
        fail(`spatial-scene: ${label} mismatch`);
    }
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :spatial-scene");
    console.log("");
    console.log("Run the canonical Experiment 0003 spatial calibration workload.");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`spatial-scene: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("spatial-scene: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const reportPath = `${root}/out/captures/0003-spatial-calibration-scene/acceptance.json`;
const alternatePose = {
    position: [-9, 5, 10],
    target: [0, 1, -3],
    vertical_fov_degrees: 60,
};
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
    const listed = await event("scene.list_objects");
    const objects = array(listed, "objects") as Record<string, unknown>[];
    const expectedObjects = [
        [1, "ground.reference"],
        [10, "axis.positive_x"],
        [11, "axis.positive_y"],
        [12, "axis.positive_z"],
        [100, "marker.near"],
        [101, "marker.center"],
        [102, "marker.far"],
        [110, "block.occluder"],
    ];
    if (objects.length !== expectedObjects.length) fail("spatial-scene: object count mismatch");
    expectedObjects.forEach(([expectedId, expectedName], index) => {
        if (objects[index].id !== expectedId || objects[index].name !== expectedName) {
            fail(`spatial-scene: object ${index} identity mismatch`);
        }
    });

    await event("camera.reset");
    const default1 = await capture("default-1");
    const default2 = await capture("default-2");
    if (pixelHash(default1) !== pixelHash(default2) || pngHash(default1) !== pngHash(default2)) {
        fail("spatial-scene: repeated default captures are not deterministic");
    }
    expectVector(array(camera(default1), "position"), [9, 6, 12], "default camera position");
    expectVector(array(camera(default1), "target"), [0, 1, -3], "default camera target");

    await event("camera.set_pose", alternatePose);
    const alternate = await capture("alternate");
    if (pixelHash(default1) === pixelHash(alternate)) {
        fail("spatial-scene: alternate camera did not change the rendered pixels");
    }
    expectVector(
        array(camera(alternate), "position"),
        alternatePose.position,
        "alternate camera position",
    );

    const firstProcess = field<number>(default1, "processId", "number");
    await lifecycle("restart");
    await event("workbench.pause");
    await event("camera.reset");
    const defaultRestart = await capture("default-restart");
    const restartedProcess = field<number>(defaultRestart, "processId", "number");
    if (firstProcess === restartedProcess) fail("spatial-scene: restart preserved the process ID");
    if (
        pixelHash(default1) !== pixelHash(defaultRestart) ||
        pngHash(default1) !== pngHash(defaultRestart)
    ) {
        fail("spatial-scene: default camera changed after restart");
    }

    report = {
        schemaVersion: 1,
        outcome: "pass",
        sceneRevision: "calibration-v1",
        objectCount: objects.length,
        firstProcess,
        restartedProcess,
        default: { pixelSha256: pixelHash(default1), pngSha256: pngHash(default1) },
        alternate: { pixelSha256: pixelHash(alternate), pngSha256: pngHash(alternate) },
        captures: ["default-1", "default-2", "alternate", "default-restart"],
    };
} finally {
    await lifecycle("stop");
}

const status = await sidecar(["status"]) as Record<string, unknown>;
if (field<boolean>(object(status, "runtime"), "running", "boolean")) {
    fail("spatial-scene: broker remains running");
}
for (const target of array(status, "targets")) {
    if (typeof target !== "object" || target === null || Array.isArray(target)) {
        fail("spatial-scene: invalid status target");
    }
    if (field<boolean>(target as Record<string, unknown>, "running", "boolean")) {
        fail("spatial-scene: a target remains running");
    }
}
if (!report) fail("spatial-scene: acceptance report was not produced");
await Deno.writeTextFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
