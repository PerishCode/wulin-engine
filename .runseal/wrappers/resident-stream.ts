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
        fail(`resident-stream: sidecar ${args[0]} failed${stderr ? `: ${stderr}` : ""}`);
    }
    if (!stdout) return null;
    try {
        return JSON.parse(stdout);
    } catch {
        fail(`resident-stream: sidecar returned non-JSON output: ${stdout}`);
    }
}

async function lifecycle(command: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [command, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`resident-stream: sidecar ${command} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]) as Record<string, unknown>;
    if (response.ok !== true || typeof response.data !== "object" || response.data === null) {
        fail(`resident-stream: ${verb} returned an invalid response`);
    }
    return response.data as Record<string, unknown>;
}

function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) fail(`resident-stream: expected ${name} to be ${type}`);
    return value as T;
}

function object(owner: Record<string, unknown>, name: string): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`resident-stream: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) fail(`resident-stream: expected ${name} to be an array`);
    return value;
}

type StreamExpected = {
    retained: number;
    uploaded: number;
    evicted: number;
    resident: number;
    instanceBytes: number;
    totalBytes: number;
};

async function stream(
    x: number,
    z: number,
    expected: StreamExpected,
): Promise<Record<string, unknown>> {
    const report = await event("resident.stream", {
        world_region_side: 128,
        active_center_x: x,
        active_center_z: z,
        active_radius: 2,
    });
    for (
        const [fieldName, value] of [
            ["retainedRegionCount", expected.retained],
            ["uploadedRegionCount", expected.uploaded],
            ["evictedRegionCount", expected.evicted],
            ["residentRegionCount", expected.resident],
            ["instanceBytes", expected.instanceBytes],
            ["mappingBytes", 200],
            ["totalBytes", expected.totalBytes],
        ] as const
    ) {
        if (field<number>(report, fieldName, "number") !== value) {
            fail(`resident-stream: [${x},${z}] ${fieldName} mismatch`);
        }
    }
    if (field<number>(report, "generationMs", "number") < 0) {
        fail(`resident-stream: [${x},${z}] generation timing mismatch`);
    }
    if (field<number>(report, "transactionMs", "number") <= 0) {
        fail(`resident-stream: [${x},${z}] transaction timing mismatch`);
    }
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

type Distribution = {
    minimum: number;
    median: number;
    p95: number;
    p99: number;
    maximum: number;
};

type ProbeSample = {
    visible: number;
    compaction: number;
    draw: number;
    total: number;
};

function distribution(values: number[]): Distribution {
    if (values.length === 0 || values.some((value) => !Number.isFinite(value) || value <= 0)) {
        fail("resident-stream: timing distribution contains invalid values");
    }
    const sorted = [...values].sort((left, right) => left - right);
    const percentile = (value: number) => sorted[Math.ceil(value * sorted.length) - 1];
    return {
        minimum: sorted[0],
        median: percentile(0.5),
        p95: percentile(0.95),
        p99: percentile(0.99),
        maximum: sorted.at(-1)!,
    };
}

async function probe(): Promise<ProbeSample> {
    const report = await event("load.probe");
    if (field<number>(report, "activeRegionCount", "number") !== 25) {
        fail("resident-stream: active region count mismatch");
    }
    if (field<number>(report, "candidateInstanceCount", "number") !== 25_600) {
        fail("resident-stream: candidate count mismatch");
    }
    if (JSON.stringify(array(report, "dispatch")) !== JSON.stringify([25, 4, 1])) {
        fail("resident-stream: dispatch mismatch");
    }
    if (field<number>(report, "indirectDrawCount", "number") !== 1) {
        fail("resident-stream: indirect draw count mismatch");
    }
    if (field<number>(report, "probeIterations", "number") !== 64) {
        fail("resident-stream: probe iteration count mismatch");
    }
    const visible = field<number>(report, "visibleInstanceCount", "number");
    if (visible <= 0 || visible >= 25_600) fail("resident-stream: visibility is not compacted");
    const compaction = field<number>(report, "gpuCompactionMs", "number");
    const draw = field<number>(report, "gpuDrawMs", "number");
    const total = field<number>(report, "gpuTotalMs", "number");
    if (compaction <= 0 || draw <= 0 || total < compaction + draw) {
        fail("resident-stream: GPU timestamps are invalid");
    }
    return { visible, compaction, draw, total };
}

async function measureProbes(): Promise<Record<string, unknown>> {
    const samples: ProbeSample[] = [];
    for (let index = 0; index < 16; index += 1) samples.push(await probe());
    const visible = samples[0].visible;
    if (samples.some((sample) => sample.visible !== visible)) {
        fail("resident-stream: visible count changed within one resident state");
    }
    return {
        sampleCount: samples.length,
        probeIterations: 64,
        visible,
        gpuCompactionMs: distribution(samples.map((sample) => sample.compaction)),
        gpuDrawMs: distribution(samples.map((sample) => sample.draw)),
        gpuTotalMs: distribution(samples.map((sample) => sample.total)),
    };
}

async function captureEvidence(id: string): Promise<Record<string, unknown>> {
    const color = await event("workbench.capture", {
        id: `${id}-color`,
        collection: "0006-resident-region-streaming",
    });
    const ids = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0006-resident-region-streaming",
        samples: [{ x: 0, y: 0 }, { x: 600, y: 600 }],
    });
    for (const manifest of [color, ids]) {
        if (manifest.lastError !== null) fail(`${id}: renderer error was reported`);
        if (object(manifest, "renderer").deviceRemovedReason !== null) {
            fail(`${id}: device removal was reported`);
        }
        if (field<string>(object(manifest, "workload"), "mode", "string") !== "resident-load") {
            fail(`${id}: capture workload mismatch`);
        }
    }
    const perception = object(ids, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`${id}: unknown IDs were reported`);
    const samples = array(evidence, "samples") as Record<string, unknown>[];
    if (samples[0].id !== 0 || typeof samples[1].name !== "string") {
        fail(`${id}: semantic sample mismatch`);
    }
    if (!samples[1].name.startsWith("load.region.")) fail(`${id}: region semantic mismatch`);
    const colorPixels = field<number>(object(color, "image"), "differentPixelCount", "number");
    const background = field<number>(
        object(evidence, "fullFrame"),
        "backgroundPixelCount",
        "number",
    );
    if (colorPixels !== 921_600 - background) fail(`${id}: color and ID coverage differ`);
    return {
        colorSha256: field<string>(object(color, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(perception, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(perception, "diagnosticPngSha256", "string"),
        sampleId: samples[1].id,
        sampleName: samples[1].name,
    };
}

function sameEvidence(
    actual: Record<string, unknown>,
    expected: Record<string, unknown>,
    label: string,
): void {
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        fail(`resident-stream: ${label} evidence mismatch`);
    }
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :resident-stream");
    console.log("");
    console.log("Run the canonical Experiment 0006 resident region streaming workload.");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`resident-stream: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("resident-stream: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const reportPath = `${root}/out/captures/0006-resident-region-streaming/acceptance.json`;
const initialExpected = {
    retained: 0,
    uploaded: 25,
    evicted: 0,
    resident: 25,
    instanceBytes: 512_000,
    totalBytes: 512_200,
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
    await followCamera(64, 64);
    const initial = await stream(64, 64, initialExpected);
    const initialProbes = await measureProbes();
    const initialVisible = field<number>(initialProbes, "visible", "number");
    const initialEvidence = await captureEvidence("initial");
    if (
        initialEvidence.colorSha256 !==
            "9bd075106177ec022395275cd534869c18abbdfd60dd7e8aa040ba7e8c9dbfac"
    ) {
        fail("resident-stream: resident color differs from Experiment 0005");
    }
    if (
        initialEvidence.rawIdSha256 !==
            "8431f1c795ec759e777acd7214611ab511e519b0143e2671ff755b0bf6427c6f"
    ) {
        fail("resident-stream: resident IDs differ from Experiment 0005");
    }
    const firstProcess = field<number>(await event("workbench.status"), "processId", "number");

    await followCamera(65, 64);
    const adjacentX = await stream(65, 64, {
        retained: 20,
        uploaded: 5,
        evicted: 0,
        resident: 30,
        instanceBytes: 102_400,
        totalBytes: 102_600,
    });
    const adjacentXProbes = await measureProbes();
    const adjacentXVisible = field<number>(adjacentXProbes, "visible", "number");

    await followCamera(65, 65);
    const adjacentZ = await stream(65, 65, {
        retained: 20,
        uploaded: 5,
        evicted: 0,
        resident: 35,
        instanceBytes: 102_400,
        totalBytes: 102_600,
    });
    const adjacentZProbes = await measureProbes();
    const adjacentZVisible = field<number>(adjacentZProbes, "visible", "number");

    await followCamera(64, 64);
    const revisit = await stream(64, 64, {
        retained: 25,
        uploaded: 0,
        evicted: 0,
        resident: 35,
        instanceBytes: 0,
        totalBytes: 200,
    });
    const revisitProbes = await measureProbes();
    const revisitVisible = field<number>(revisitProbes, "visible", "number");
    if (revisitVisible !== initialVisible) {
        fail("resident-stream: cached revisit visible count mismatch");
    }
    const revisitEvidence = await captureEvidence("revisit");
    sameEvidence(revisitEvidence, initialEvidence, "cached revisit");
    const unchanged = await stream(64, 64, {
        retained: 25,
        uploaded: 0,
        evicted: 0,
        resident: 35,
        instanceBytes: 0,
        totalBytes: 200,
    });

    await followCamera(96, 96);
    const teleport = await stream(96, 96, {
        retained: 0,
        uploaded: 25,
        evicted: 11,
        resident: 49,
        instanceBytes: 512_000,
        totalBytes: 512_200,
    });
    const teleportProbes = await measureProbes();
    const teleportVisible = field<number>(teleportProbes, "visible", "number");

    await lifecycle("restart");
    await event("workbench.pause");
    await followCamera(64, 64);
    const restarted = await stream(64, 64, initialExpected);
    if (restarted.uploadedSha256 !== initial.uploadedSha256) {
        fail("resident-stream: generated data changed after restart");
    }
    const restartProbes = await measureProbes();
    const restartVisible = field<number>(restartProbes, "visible", "number");
    if (restartVisible !== initialVisible) {
        fail("resident-stream: restart visible count mismatch");
    }
    const restartEvidence = await captureEvidence("restart");
    sameEvidence(restartEvidence, initialEvidence, "restart");
    const restartedProcess = field<number>(
        await event("workbench.status"),
        "processId",
        "number",
    );
    if (restartedProcess === firstProcess) {
        fail("resident-stream: restart preserved the workbench process ID");
    }

    await event("load.disable");
    await event("camera.reset");
    const calibrationColor = await event("workbench.capture", {
        id: "calibration-color",
        collection: "0006-resident-region-streaming",
    });
    const calibrationIds = await event("perception.capture", {
        id: "calibration-ids",
        collection: "0006-resident-region-streaming",
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
        calibrationColorHash !== "8f0fc6e9a49b95330921ff4f6b30c18bb4d16f2bd86f1b452a00f2ff00b68f6d"
    ) {
        fail("resident-stream: calibration color baseline changed");
    }
    if (calibrationIdHash !== "b132c850f0295119f9beba14d9287881a8bf1e8d51487800726de942f10e1da4") {
        fail("resident-stream: calibration object-ID baseline changed");
    }

    report = {
        schemaVersion: 1,
        outcome: "pass",
        residentRevision: "resident-region-v1",
        firstProcess,
        restartedProcess,
        visibility: {
            initial: initialVisible,
            adjacentX: adjacentXVisible,
            adjacentZ: adjacentZVisible,
            revisit: revisitVisible,
            teleport: teleportVisible,
            restart: restartVisible,
        },
        probes: {
            initial: initialProbes,
            adjacentX: adjacentXProbes,
            adjacentZ: adjacentZProbes,
            revisit: revisitProbes,
            teleport: teleportProbes,
            restart: restartProbes,
        },
        initial,
        adjacentX,
        adjacentZ,
        revisit,
        unchanged,
        teleport,
        restarted,
        residentEvidence: initialEvidence,
        calibration: { colorSha256: calibrationColorHash, rawIdSha256: calibrationIdHash },
    };
} finally {
    await lifecycle("stop");
}

const status = await sidecar(["status"]) as Record<string, unknown>;
if (field<boolean>(object(status, "runtime"), "running", "boolean")) {
    fail("resident-stream: broker remains running");
}
for (const target of array(status, "targets")) {
    if (typeof target !== "object" || target === null || Array.isArray(target)) {
        fail("resident-stream: invalid status target");
    }
    if (field<boolean>(target as Record<string, unknown>, "running", "boolean")) {
        fail("resident-stream: a target remains running");
    }
}
if (!report) fail("resident-stream: acceptance report was not produced");
await Deno.writeTextFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
