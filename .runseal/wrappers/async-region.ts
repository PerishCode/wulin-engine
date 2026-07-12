function fail(message: string): never {
    throw new Error(message);
}

async function command(
    args: string[],
): Promise<{ success: boolean; value: Record<string, unknown> }> {
    const output = await new Deno.Command("sidecar", {
        args: [...args, "--config", "sidecar.toml", "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    if (!stdout) fail(`async-region: sidecar ${args[0]} returned no JSON`);
    try {
        return { success: output.success, value: JSON.parse(stdout) };
    } catch {
        fail(`async-region: sidecar returned invalid JSON: ${stdout}`);
    }
}

async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [verb, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`async-region: sidecar ${verb} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await command([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]);
    if (!response.success || response.value.ok !== true) {
        fail(`async-region: ${verb} failed: ${String(response.value.error)}`);
    }
    return object(response.value, "data");
}

async function expectBusy(x: number, z: number): Promise<string> {
    const response = await command([
        "inspect",
        "workbench",
        "async.schedule",
        JSON.stringify(config(x, z)),
    ]);
    const error = String(response.value.error);
    if (response.success || response.value.ok !== false || !error.startsWith("stream_busy:")) {
        fail("async-region: concurrent request was not rejected as stream_busy");
    }
    return error;
}

function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) fail(`async-region: expected ${name} to be ${type}`);
    return value as T;
}

function object(owner: Record<string, unknown>, name: string): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`async-region: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) fail(`async-region: expected ${name} to be an array`);
    return value;
}

function config(x: number, z: number): Record<string, number> {
    return {
        world_region_side: 128,
        active_center_x: x,
        active_center_z: z,
        active_radius: 2,
    };
}

type Expected = {
    retained: number;
    uploaded: number;
    evicted: number;
    resident: number;
    protectedCount: number;
    bytes: number;
};

async function schedule(
    x: number,
    z: number,
    expected: Expected,
): Promise<Record<string, unknown>> {
    const report = await event("async.schedule", config(x, z));
    for (
        const [name, value] of [
            ["retainedRegionCount", expected.retained],
            ["uploadedRegionCount", expected.uploaded],
            ["evictedRegionCount", expected.evicted],
            ["residentRegionCount", expected.resident],
            ["protectedRegionCount", expected.protectedCount],
            ["instanceBytes", expected.bytes],
        ] as const
    ) {
        if (field<number>(report, name, "number") !== value) {
            fail(`async-region: [${x},${z}] ${name} mismatch`);
        }
    }
    if (field<number>(report, "scheduleMs", "number") <= 0) {
        fail(`async-region: [${x},${z}] schedule timing is invalid`);
    }
    return report;
}

async function waitPublished(x: number, z: number): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        const status = await event("async.status");
        const published = status.published;
        if (typeof published === "object" && published !== null && !Array.isArray(published)) {
            const value = published as Record<string, unknown>;
            if (value.activeCenterX === x && value.activeCenterZ === z && status.pending === null) {
                return status;
            }
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail(`async-region: [${x},${z}] publication timed out`);
}

async function publish(x: number, z: number, expected: Expected): Promise<Record<string, unknown>> {
    const report = await schedule(x, z, expected);
    await event("workbench.resume");
    const status = await waitPublished(x, z);
    await event("workbench.pause");
    report.publication = object(status, "lastCompleted");
    return report;
}

async function followCamera(x: number, z: number): Promise<void> {
    const worldX = (x - 64) * 16;
    const worldZ = (z - 64) * 16;
    await event("camera.set_pose", {
        position: [worldX, 30, worldZ + 30],
        target: [worldX, 0, worldZ],
        vertical_fov_degrees: 60,
    });
}

function distribution(values: number[]): Record<string, number> {
    if (values.some((value) => !Number.isFinite(value) || value <= 0)) {
        fail("async-region: invalid GPU timing sample");
    }
    const sorted = [...values].sort((left, right) => left - right);
    const at = (fraction: number) => sorted[Math.ceil(fraction * sorted.length) - 1];
    return {
        minimum: sorted[0],
        median: at(0.5),
        p95: at(0.95),
        p99: at(0.99),
        maximum: sorted.at(-1)!,
    };
}

async function measureProbes(): Promise<Record<string, unknown>> {
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < 16; index += 1) samples.push(await event("load.probe"));
    const visible = field<number>(samples[0], "visibleInstanceCount", "number");
    for (const sample of samples) {
        if (
            field<number>(sample, "activeRegionCount", "number") !== 25 ||
            field<number>(sample, "candidateInstanceCount", "number") !== 25_600 ||
            field<number>(sample, "visibleInstanceCount", "number") !== visible ||
            field<number>(sample, "indirectDrawCount", "number") !== 1 ||
            field<number>(sample, "probeIterations", "number") !== 64 ||
            JSON.stringify(array(sample, "dispatch")) !== JSON.stringify([25, 4, 1])
        ) fail("async-region: GPU probe shape or visibility changed");
    }
    if (visible <= 0 || visible >= 25_600) fail("async-region: visibility was not compacted");
    return {
        sampleCount: samples.length,
        probeIterations: 64,
        visible,
        gpuCompactionMs: distribution(
            samples.map((sample) => field(sample, "gpuCompactionMs", "number")),
        ),
        gpuDrawMs: distribution(samples.map((sample) => field(sample, "gpuDrawMs", "number"))),
        gpuTotalMs: distribution(samples.map((sample) => field(sample, "gpuTotalMs", "number"))),
    };
}

async function captureEvidence(id: string): Promise<Record<string, unknown>> {
    const color = await event("workbench.capture", {
        id: `${id}-color`,
        collection: "0007-async-region-publication",
    });
    const ids = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0007-async-region-publication",
        samples: [{ x: 0, y: 0 }, { x: 600, y: 600 }],
    });
    for (const manifest of [color, ids]) {
        if (
            manifest.lastError !== null || object(manifest, "renderer").deviceRemovedReason !== null
        ) {
            fail(`async-region: ${id} reported a renderer failure`);
        }
        if (
            field<string>(object(manifest, "workload"), "mode", "string") !== "async-resident-load"
        ) {
            fail(`async-region: ${id} workload mode mismatch`);
        }
    }
    const perception = object(ids, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`async-region: ${id} has unknown IDs`);
    const samples = array(evidence, "samples") as Record<string, unknown>[];
    if (
        samples[0].id !== 0 || typeof samples[1].name !== "string" ||
        !samples[1].name.startsWith("load.region.")
    ) fail(`async-region: ${id} sample mismatch`);
    const colorPixels = field<number>(object(color, "image"), "differentPixelCount", "number");
    const background = field<number>(
        object(evidence, "fullFrame"),
        "backgroundPixelCount",
        "number",
    );
    if (colorPixels !== 921_600 - background) fail(`async-region: ${id} coverage mismatch`);
    return {
        colorSha256: field<string>(object(color, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(perception, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(perception, "diagnosticPngSha256", "string"),
        sampleId: samples[1].id,
        sampleName: samples[1].name,
    };
}

function same(actual: unknown, expected: unknown, label: string): void {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        fail(`async-region: ${label} mismatch`);
    }
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :async-region");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`async-region: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("async-region: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const reportPath = `${root}/out/captures/0007-async-region-publication/acceptance.json`;
let finalReport: Record<string, unknown> | undefined;

await lifecycle("stop");
try {
    await Deno.remove(reportPath);
} catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) throw error;
}
try {
    await lifecycle("start");
    const initial = await publish(64, 64, {
        retained: 0,
        uploaded: 25,
        evicted: 0,
        resident: 25,
        protectedCount: 0,
        bytes: 512_000,
    });
    const initialStatus = await event("async.status");
    const resources = {
        capacity: field<number>(initialStatus, "capacity", "number"),
        descriptorCount: field<number>(initialStatus, "descriptorCount", "number"),
        inFlightCapacity: field<number>(initialStatus, "inFlightCapacity", "number"),
        regionPayloadBytes: field<number>(initialStatus, "regionPayloadBytes", "number"),
        defaultHeapAllocationBytes: field<number>(
            initialStatus,
            "defaultHeapAllocationBytes",
            "number",
        ),
        uploadArenaBytes: field<number>(initialStatus, "uploadArenaBytes", "number"),
    };
    if (
        resources.capacity !== 50 || resources.descriptorCount !== 50 ||
        resources.inFlightCapacity !== 1 || resources.regionPayloadBytes !== 1_024_000 ||
        resources.uploadArenaBytes !== 1_024_000 ||
        resources.defaultHeapAllocationBytes < resources.regionPayloadBytes
    ) fail("async-region: resource bounds mismatch");
    await followCamera(64, 64);
    const initialProbes = await measureProbes();
    const initialEvidence = await captureEvidence("initial");
    if (
        initialEvidence.colorSha256 !==
            "9bd075106177ec022395275cd534869c18abbdfd60dd7e8aa040ba7e8c9dbfac" ||
        initialEvidence.rawIdSha256 !==
            "8431f1c795ec759e777acd7214611ab511e519b0143e2671ff755b0bf6427c6f"
    ) {
        fail("async-region: initial evidence differs from Experiment 0006");
    }
    const firstProcess = field<number>(await event("workbench.status"), "processId", "number");

    const gate = await event("async.gate.arm");
    const beforeFrame = field<number>(await event("workbench.status"), "frameIndex", "number");
    const adjacentX = await schedule(65, 64, {
        retained: 20,
        uploaded: 5,
        evicted: 0,
        resident: 30,
        protectedCount: 25,
        bytes: 102_400,
    });
    const heldStatus = await event("async.status");
    if (field<string>(object(heldStatus, "pending"), "stage", "string") !== "gated") {
        fail("async-region: held transaction is not gated");
    }
    const busyError = await expectBusy(65, 65);
    await event("workbench.resume");
    let afterFrame = beforeFrame;
    const frameDeadline = Date.now() + 3_000;
    while (afterFrame - beforeFrame < 30 && Date.now() < frameDeadline) {
        await new Promise((resolve) => setTimeout(resolve, 20));
        afterFrame = field<number>(await event("workbench.status"), "frameIndex", "number");
    }
    await event("workbench.pause");
    if (afterFrame - beforeFrame < 30) {
        fail("async-region: direct frames did not advance while gated");
    }
    const heldEvidence = await captureEvidence("held");
    same(heldEvidence, initialEvidence, "held snapshot evidence");
    const stillHeld = await event("async.status");
    const heldPending = object(stillHeld, "pending");
    const heldReport = object(heldPending, "report");
    if (
        field<number>(object(stillHeld, "copy"), "completedFence", "number") >=
            field<number>(heldReport, "copyFence", "number")
    ) fail("async-region: held copy completed");

    await event("async.gate.release");
    await event("workbench.resume");
    const releasedStatus = await waitPublished(65, 64);
    await event("workbench.pause");
    adjacentX.publication = object(releasedStatus, "lastCompleted");
    await followCamera(65, 64);
    const adjacentXProbes = await measureProbes();

    const adjacentZ = await publish(65, 65, {
        retained: 20,
        uploaded: 5,
        evicted: 0,
        resident: 35,
        protectedCount: 25,
        bytes: 102_400,
    });
    await followCamera(65, 65);
    const adjacentZProbes = await measureProbes();

    const revisit = await publish(64, 64, {
        retained: 25,
        uploaded: 0,
        evicted: 0,
        resident: 35,
        protectedCount: 25,
        bytes: 0,
    });
    await followCamera(64, 64);
    const revisitProbes = await measureProbes();
    const revisitEvidence = await captureEvidence("revisit");
    same(revisitEvidence, initialEvidence, "cached revisit evidence");

    const teleport = await publish(96, 96, {
        retained: 0,
        uploaded: 25,
        evicted: 10,
        resident: 50,
        protectedCount: 25,
        bytes: 512_000,
    });
    await followCamera(96, 96);
    const teleportProbes = await measureProbes();

    await lifecycle("restart");
    const restarted = await publish(64, 64, {
        retained: 0,
        uploaded: 25,
        evicted: 0,
        resident: 25,
        protectedCount: 0,
        bytes: 512_000,
    });
    if (restarted.uploadedSha256 !== initial.uploadedSha256) {
        fail("async-region: initial upload checksum changed after restart");
    }
    await followCamera(64, 64);
    const restartProbes = await measureProbes();
    const restartEvidence = await captureEvidence("restart");
    same(restartEvidence, initialEvidence, "restart evidence");
    const restartedProcess = field<number>(await event("workbench.status"), "processId", "number");
    if (restartedProcess === firstProcess) fail("async-region: process identity survived restart");

    await event("load.disable");
    await event("camera.reset");
    const calibrationColor = await event("workbench.capture", {
        id: "calibration-color",
        collection: "0007-async-region-publication",
    });
    const calibrationIds = await event("perception.capture", {
        id: "calibration-ids",
        collection: "0007-async-region-publication",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const calibrationColorHash = field<string>(
        object(calibrationColor, "image"),
        "pixelSha256",
        "string",
    );
    const calibrationIdHash = field<string>(
        object(calibrationIds, "perception"),
        "rawSha256",
        "string",
    );
    if (
        calibrationColorHash !==
            "8f0fc6e9a49b95330921ff4f6b30c18bb4d16f2bd86f1b452a00f2ff00b68f6d" ||
        calibrationIdHash !== "b132c850f0295119f9beba14d9287881a8bf1e8d51487800726de942f10e1da4"
    ) {
        fail("async-region: calibration regression changed");
    }

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        asyncRevision: "async-resident-v1",
        resources,
        firstProcess,
        restartedProcess,
        held: {
            gateFence: gate.gateFence,
            beforeFrame,
            afterFrame,
            advancedFrames: afterFrame - beforeFrame,
            stage: object(heldStatus, "pending").stage,
            busyError,
            evidence: heldEvidence,
        },
        transactions: { initial, adjacentX, adjacentZ, revisit, teleport, restarted },
        probes: {
            initial: initialProbes,
            adjacentX: adjacentXProbes,
            adjacentZ: adjacentZProbes,
            revisit: revisitProbes,
            teleport: teleportProbes,
            restart: restartProbes,
        },
        residentEvidence: initialEvidence,
        calibration: { colorSha256: calibrationColorHash, rawIdSha256: calibrationIdHash },
    };
} finally {
    await lifecycle("stop");
}

const cleanup = await command(["status"]);
if (field<boolean>(object(cleanup.value, "runtime"), "running", "boolean")) {
    fail("async-region: broker remains running");
}
for (const target of array(cleanup.value, "targets") as Record<string, unknown>[]) {
    if (field<boolean>(target, "running", "boolean")) fail("async-region: target remains running");
}
if (!finalReport) fail("async-region: no acceptance report was produced");
await Deno.writeTextFile(reportPath, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));
