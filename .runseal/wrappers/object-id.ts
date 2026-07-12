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
        fail(`object-id: sidecar ${args[0]} failed${stderr ? `: ${stderr}` : ""}`);
    }
    if (!stdout) return null;
    try {
        return JSON.parse(stdout);
    } catch {
        fail(`object-id: sidecar returned non-JSON output: ${stdout}`);
    }
}

async function lifecycle(command: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [command, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`object-id: sidecar ${command} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]) as Record<string, unknown>;
    if (response.ok !== true || typeof response.data !== "object" || response.data === null) {
        fail(`object-id: ${verb} returned an invalid response`);
    }
    return response.data as Record<string, unknown>;
}

function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) fail(`object-id: expected ${name} to be ${type}`);
    return value as T;
}

function object(owner: Record<string, unknown>, name: string): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`object-id: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) fail(`object-id: expected ${name} to be an array`);
    return value;
}

function same(left: unknown, right: unknown, label: string): void {
    if (JSON.stringify(left) !== JSON.stringify(right)) fail(`object-id: ${label} mismatch`);
}

function sameRegion(
    actual: Record<string, unknown>,
    expected: { x: number; y: number; width: number; height: number },
): void {
    for (const key of ["x", "y", "width", "height"] as const) {
        if (field<number>(actual, key, "number") !== expected[key]) {
            fail(`object-id: bounded region ${key} mismatch`);
        }
    }
}

const samples = [{ x: 0, y: 0 }, { x: 640, y: 360 }];

async function capture(
    id: string,
    region?: { x: number; y: number; width: number; height: number },
): Promise<Record<string, unknown>> {
    const manifest = await event("perception.capture", {
        id,
        collection: "0004-object-id-perception",
        samples,
        ...(region ? { region } : {}),
    });
    if (field<number>(manifest, "schemaVersion", "number") !== 2) {
        fail(`${id}: manifest schema mismatch`);
    }
    if (manifest.lastError !== null) fail(`${id}: renderer error was reported`);
    const renderer = object(manifest, "renderer");
    if (renderer.deviceRemovedReason !== null) fail(`${id}: device removal was reported`);
    const perception = object(manifest, "perception");
    if (field<string>(perception, "format", "string") !== "R32_UINT") {
        fail(`${id}: object-ID format mismatch`);
    }
    if (field<number>(perception, "rawValueCount", "number") !== 921_600) {
        fail(`${id}: object-ID value count mismatch`);
    }
    if (field<number>(perception, "rawByteCount", "number") !== 3_686_400) {
        fail(`${id}: object-ID byte count mismatch`);
    }
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`${id}: unknown IDs were reported`);
    const fullFrame = object(evidence, "fullFrame");
    if (array(fullFrame, "objects").length !== 8) fail(`${id}: visible object count mismatch`);
    const captureSamples = array(evidence, "samples") as Record<string, unknown>[];
    if (captureSamples.length !== 2 || captureSamples[0].id !== 0) {
        fail(`${id}: background sample mismatch`);
    }
    if (captureSamples[1].id !== 110 || captureSamples[1].name !== "block.occluder") {
        fail(`${id}: center occlusion sample mismatch`);
    }
    const artifacts = object(manifest, "artifacts");
    for (const key of ["png", "manifest", "objectIds", "objectIdPng"]) {
        const relative = field<string>(artifacts, key, "string");
        const info = await Deno.stat(`${root}/${relative}`);
        if (!info.isFile || info.size === 0) fail(`${id}: missing ${key} artifact`);
    }
    return manifest;
}

function perception(manifest: Record<string, unknown>): Record<string, unknown> {
    return object(manifest, "perception");
}

function evidence(manifest: Record<string, unknown>): Record<string, unknown> {
    return object(perception(manifest), "evidence");
}

function rawHash(manifest: Record<string, unknown>): string {
    return field<string>(perception(manifest), "rawSha256", "string");
}

function diagnosticHash(manifest: Record<string, unknown>): string {
    return field<string>(perception(manifest), "diagnosticPngSha256", "string");
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :object-id");
    console.log("");
    console.log("Run the canonical Experiment 0004 object-ID perception workload.");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`object-id: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("object-id: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const reportPath = `${root}/out/captures/0004-object-id-perception/acceptance.json`;
const boundedRegion = { x: 560, y: 240, width: 160, height: 200 };
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
    const registry = await event("scene.list_objects");
    const registryObjects = array(registry, "objects") as Record<string, unknown>[];
    if (registryObjects.length !== 8) fail("object-id: scene registry count mismatch");
    const registryNames = new Map(registryObjects.map((entry) => [entry.id, entry.name]));

    await event("camera.reset");
    const default1 = await capture("default-1");
    const default2 = await capture("default-2");
    if (rawHash(default1) !== rawHash(default2)) fail("object-id: default raw IDs drifted");
    if (diagnosticHash(default1) !== diagnosticHash(default2)) {
        fail("object-id: default diagnostic PNG drifted");
    }
    same(
        object(evidence(default1), "fullFrame"),
        object(evidence(default2), "fullFrame"),
        "default full-frame histogram",
    );

    const visible = array(object(evidence(default1), "fullFrame"), "objects") as Record<
        string,
        unknown
    >[];
    for (const entry of visible) {
        if (registryNames.get(entry.id) !== entry.name) {
            fail(`object-id: semantic join mismatch for ID ${entry.id}`);
        }
    }

    const bounded = await capture("bounded", boundedRegion);
    if (rawHash(default1) !== rawHash(bounded)) {
        fail("object-id: analysis region changed the rendered ID buffer");
    }
    const boundedEvidence = object(evidence(bounded), "region");
    sameRegion(object(boundedEvidence, "bounds"), boundedRegion);
    if (field<number>(boundedEvidence, "pixelCount", "number") !== 32_000) {
        fail("object-id: bounded region pixel count mismatch");
    }
    const boundedObjects = array(boundedEvidence, "objects") as Record<string, unknown>[];
    if (!boundedObjects.some((entry) => entry.id === 110)) {
        fail("object-id: bounded region does not contain the occluder");
    }

    await event("camera.set_pose", alternatePose);
    const alternate = await capture("alternate");
    if (rawHash(default1) === rawHash(alternate)) {
        fail("object-id: alternate camera did not change object IDs");
    }

    const firstProcess = field<number>(default1, "processId", "number");
    await lifecycle("restart");
    await event("workbench.pause");
    await event("camera.reset");
    const defaultRestart = await capture("default-restart");
    const restartedProcess = field<number>(defaultRestart, "processId", "number");
    if (firstProcess === restartedProcess) fail("object-id: restart preserved the process ID");
    if (rawHash(default1) !== rawHash(defaultRestart)) {
        fail("object-id: default raw IDs changed after restart");
    }
    if (diagnosticHash(default1) !== diagnosticHash(defaultRestart)) {
        fail("object-id: diagnostic PNG changed after restart");
    }
    same(evidence(default1), evidence(defaultRestart), "restart perception evidence");

    report = {
        schemaVersion: 1,
        outcome: "pass",
        sceneRevision: "calibration-v1",
        objectCount: registryObjects.length,
        firstProcess,
        restartedProcess,
        default: {
            rawSha256: rawHash(default1),
            diagnosticPngSha256: diagnosticHash(default1),
        },
        alternate: {
            rawSha256: rawHash(alternate),
            diagnosticPngSha256: diagnosticHash(alternate),
        },
        boundedRegion,
        boundedObjectIds: boundedObjects.map((entry) => entry.id),
        captures: ["default-1", "default-2", "bounded", "alternate", "default-restart"],
    };
} finally {
    await lifecycle("stop");
}

const status = await sidecar(["status"]) as Record<string, unknown>;
if (field<boolean>(object(status, "runtime"), "running", "boolean")) {
    fail("object-id: broker remains running");
}
for (const target of array(status, "targets")) {
    if (typeof target !== "object" || target === null || Array.isArray(target)) {
        fail("object-id: invalid status target");
    }
    if (field<boolean>(target as Record<string, unknown>, "running", "boolean")) {
        fail("object-id: a target remains running");
    }
}
if (!report) fail("object-id: acceptance report was not produced");
await Deno.writeTextFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
