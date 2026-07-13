import {
    array,
    assertSame as same,
    config,
    corruptRegion,
    distribution,
    type Expected,
    field,
    fileSha256,
    object,
} from "../support/cooked-region.ts";

function fail(message: string): never {
    throw new Error(message);
}

async function sidecar(
    args: string[],
): Promise<{ success: boolean; value: Record<string, unknown> }> {
    const output = await new Deno.Command("sidecar", {
        args: [...args, "--config", "sidecar.toml", "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    if (!stdout) fail(`cooked-region: sidecar ${args[0]} returned no JSON`);
    try {
        return { success: output.success, value: JSON.parse(stdout) };
    } catch {
        fail(`cooked-region: sidecar returned invalid JSON: ${stdout}`);
    }
}

async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [verb, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`cooked-region: sidecar ${verb} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]);
    if (!response.success || response.value.ok !== true) {
        fail(`cooked-region: ${verb} failed: ${String(response.value.error)}`);
    }
    return object(response.value, "data");
}

async function cook(path: string): Promise<Record<string, unknown>> {
    const output = await new Deno.Command("cargo", {
        args: ["run", "--locked", "--release", "-p", "region-cooker", "--", path],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail(`cooked-region: cooker failed with exit code ${output.code}`);
    const stdout = decoder.decode(output.stdout).trim();
    try {
        return JSON.parse(stdout);
    } catch {
        fail(`cooked-region: cooker returned invalid JSON: ${stdout}`);
    }
}

async function schedule(
    x: number,
    z: number,
    expected: Expected,
): Promise<Record<string, unknown>> {
    const start = performance.now();
    const report = await event("cooked.schedule", config(x, z));
    report.scheduleReturnMs = performance.now() - start;
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
            fail(`cooked-region: [${x},${z}] ${name} mismatch`);
        }
    }
    if (array(report, "requestedRegionIds").length !== expected.chunks) {
        fail(`cooked-region: [${x},${z}] requested chunk count mismatch`);
    }
    if (field<number>(report, "scheduleReturnMs", "number") > 500) {
        fail(`cooked-region: [${x},${z}] schedule call blocked`);
    }
    return report;
}

async function expectBusy(): Promise<string> {
    const response = await sidecar([
        "inspect",
        "workbench",
        "cooked.schedule",
        JSON.stringify(config(65, 64)),
    ]);
    const error = String(response.value.error);
    if (response.success || response.value.ok !== false || !error.startsWith("stream_busy:")) {
        fail("cooked-region: concurrent request was not rejected as stream_busy");
    }
    return error;
}

async function waitPublished(
    x: number,
    z: number,
    expected: Expected,
): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        const status = await event("cooked.status");
        const completed = status.lastCompleted;
        if (typeof completed === "object" && completed !== null && !Array.isArray(completed)) {
            const report = completed as Record<string, unknown>;
            const publishedConfig = object(report, "config");
            if (
                publishedConfig.activeCenterX === x && publishedConfig.activeCenterZ === z &&
                status.pending === null
            ) {
                const io = object(report, "io");
                if (
                    field<number>(io, "chunkCount", "number") !== expected.chunks ||
                    field<number>(io, "seekCount", "number") !== expected.chunks ||
                    field<number>(io, "payloadBytes", "number") !== expected.bytes
                ) fail(`cooked-region: [${x},${z}] I/O shape mismatch`);
                const gpu = object(report, "gpu");
                if (field<string>(gpu, "uploadedSha256", "string") !== expected.uploadSha256) {
                    fail(`cooked-region: [${x},${z}] upload checksum mismatch`);
                }
                return report;
            }
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail(`cooked-region: [${x},${z}] publication timed out`);
}

async function publish(
    x: number,
    z: number,
    expected: Expected,
): Promise<Record<string, unknown>> {
    const report = await schedule(x, z, expected);
    await event("workbench.resume");
    report.publication = await waitPublished(x, z, expected);
    await event("workbench.pause");
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

async function measureProbes(): Promise<Record<string, unknown>> {
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < 8; index += 1) samples.push(await event("load.probe"));
    const visible = field<number>(samples[0], "visibleInstanceCount", "number");
    for (const sample of samples) {
        if (
            field<number>(sample, "activeRegionCount", "number") !== 25 ||
            field<number>(sample, "candidateInstanceCount", "number") !== 25_600 ||
            field<number>(sample, "visibleInstanceCount", "number") !== visible ||
            field<number>(sample, "indirectDrawCount", "number") !== 1 ||
            field<number>(sample, "probeIterations", "number") !== 64 ||
            JSON.stringify(array(sample, "dispatch")) !== JSON.stringify([25, 4, 1])
        ) fail("cooked-region: GPU probe shape or visibility changed");
    }
    if (visible <= 0 || visible >= 25_600) fail("cooked-region: visibility was not compacted");
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

async function captureResident(id: string): Promise<Record<string, unknown>> {
    const color = await event("workbench.capture", {
        id: `${id}-color`,
        collection: "0008-cooked-region-io",
    });
    const ids = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0008-cooked-region-io",
        samples: [{ x: 0, y: 0 }, { x: 600, y: 600 }],
    });
    const perception = object(ids, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`${id}: unknown object IDs`);
    return {
        colorSha256: field<string>(object(color, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(perception, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(perception, "diagnosticPngSha256", "string"),
    };
}

async function captureCalibration(id: string): Promise<Record<string, unknown>> {
    const color = await event("workbench.capture", {
        id: `${id}-color`,
        collection: "0008-cooked-region-io",
    });
    const ids = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0008-cooked-region-io",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    return {
        colorSha256: field<string>(object(color, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(object(ids, "perception"), "rawSha256", "string"),
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :cooked-region");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`cooked-region: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("cooked-region: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const output = "out/cooked/0008-cooked-region-io";
const packA = `${output}/regions-a.wlr`;
const packB = `${output}/regions-b.wlr`;
const corruptPack = `${output}/regions-corrupt.wlr`;
const reportPath = `${root}/out/captures/0008-cooked-region-io/acceptance.json`;
let finalReport: Record<string, unknown> | undefined;

await lifecycle("stop");
for (const path of [packA, packB, corruptPack]) {
    try {
        await Deno.remove(`${root}/${path}`);
    } catch (error) {
        if (!(error instanceof Deno.errors.NotFound)) throw error;
    }
}
try {
    await Deno.remove(reportPath);
} catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) throw error;
}

const cookA = await cook(packA);
const cookB = await cook(packB);
const metadata = object(cookA, "metadata");
if (
    field<number>(metadata, "regionCount", "number") !== 60 ||
    field<number>(metadata, "payloadOffset", "number") !== 4_096 ||
    field<number>(metadata, "payloadBytes", "number") !== 1_228_800 ||
    field<number>(metadata, "fileBytes", "number") !== 1_232_896 ||
    field<number>(metadata, "payloadAlignment", "number") !== 4_096
) fail("cooked-region: cooked pack shape mismatch");
const packSha256 = field<string>(cookA, "fileSha256", "string");
if (
    packSha256 !== field<string>(cookB, "fileSha256", "string") ||
    packSha256 !== await fileSha256(`${root}/${packA}`) ||
    packSha256 !== await fileSha256(`${root}/${packB}`)
) fail("cooked-region: independent cooker outputs differ");
await corruptRegion(`${root}/${packA}`, `${root}/${corruptPack}`, 12_126);
const corruptSha256 = await fileSha256(`${root}/${corruptPack}`);
if (corruptSha256 === packSha256) fail("cooked-region: corrupt pack hash did not change");

const INITIAL: Expected = {
    retained: 0,
    uploaded: 25,
    evicted: 0,
    resident: 25,
    protectedCount: 0,
    chunks: 25,
    bytes: 512_000,
    uploadSha256: "280cb2eea7fc3e23743c6bd74f9b986ceaf00cb742a3b8214f130c6f9ea501f2",
};
const ADJACENT_X: Expected = {
    retained: 20,
    uploaded: 5,
    evicted: 0,
    resident: 30,
    protectedCount: 25,
    chunks: 5,
    bytes: 102_400,
    uploadSha256: "965b15b051eb21f77d6553ac748bcf2c0c92e38f465339612b649b172cbcfdd1",
};
const ADJACENT_Z: Expected = {
    retained: 20,
    uploaded: 5,
    evicted: 0,
    resident: 35,
    protectedCount: 25,
    chunks: 5,
    bytes: 102_400,
    uploadSha256: "2bf29f3d03389f658c38330d7dbdba736761b39bb74dc52ba0d0796153c6751f",
};
const REVISIT: Expected = {
    retained: 25,
    uploaded: 0,
    evicted: 0,
    resident: 35,
    protectedCount: 25,
    chunks: 0,
    bytes: 0,
    uploadSha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
};
const TELEPORT: Expected = {
    retained: 0,
    uploaded: 25,
    evicted: 10,
    resident: 50,
    protectedCount: 25,
    chunks: 25,
    bytes: 512_000,
    uploadSha256: "9df08f2cdf0897ef516ba9e39f80d42e296a28b7ad24203e0b9d8991af6f889e",
};

try {
    await lifecycle("start");
    const opened = await event("cooked.open", { path: packA });
    same(opened, metadata, "runtime pack metadata");
    const openStatus = await event("cooked.status");
    const runtimePack = object(openStatus, "pack");
    if (
        field<number>(runtimePack, "indexReadBytes", "number") !== 3_424 ||
        field<number>(runtimePack, "payloadBytesReadAtOpen", "number") !== 0 ||
        field<number>(openStatus, "workerCapacity", "number") !== 1 ||
        field<number>(openStatus, "requestCapacity", "number") !== 1 ||
        field<number>(openStatus, "completionCapacity", "number") !== 1
    ) fail("cooked-region: runtime I/O bounds mismatch");

    await event("workbench.pause");
    const gate = await event("cooked.gate.arm");
    const beforeFrame = field<number>(await event("workbench.status"), "frameIndex", "number");
    const initial = await schedule(64, 64, INITIAL);
    const busyError = await expectBusy();
    const gated = await event("cooked.status");
    const gatedPending = object(gated, "pending");
    if (
        field<string>(gatedPending, "stage", "string") !== "io-gated" ||
        field<number>(gatedPending, "chunkCount", "number") !== 0 ||
        field<number>(gatedPending, "payloadBytesRead", "number") !== 0
    ) fail("cooked-region: gated request performed payload I/O");
    await event("workbench.resume");
    let afterFrame = beforeFrame;
    const frameDeadline = Date.now() + 3_000;
    while (afterFrame - beforeFrame < 30 && Date.now() < frameDeadline) {
        await new Promise((resolve) => setTimeout(resolve, 20));
        afterFrame = field<number>(await event("workbench.status"), "frameIndex", "number");
    }
    await event("workbench.pause");
    if (afterFrame - beforeFrame < 30) fail("cooked-region: direct frames stalled behind I/O");
    const heldCalibration = await captureCalibration("held-calibration");
    same(heldCalibration, {
        colorSha256: "8f0fc6e9a49b95330921ff4f6b30c18bb4d16f2bd86f1b452a00f2ff00b68f6d",
        rawIdSha256: "b132c850f0295119f9beba14d9287881a8bf1e8d51487800726de942f10e1da4",
    }, "gated calibration evidence");

    await event("cooked.gate.release");
    await event("workbench.resume");
    initial.publication = await waitPublished(64, 64, INITIAL);
    await event("workbench.pause");
    await followCamera(64, 64);
    const initialProbes = await measureProbes();
    const initialEvidence = await captureResident("initial");
    same(initialEvidence, {
        colorSha256: "9bd075106177ec022395275cd534869c18abbdfd60dd7e8aa040ba7e8c9dbfac",
        rawIdSha256: "8431f1c795ec759e777acd7214611ab511e519b0143e2671ff755b0bf6427c6f",
        diagnosticPngSha256: "91c9fad2b269e4ed5e6689015fec862a1934b43db23199ba536373d891a919e8",
    }, "initial resident evidence");
    const firstProcess = field<number>(await event("workbench.status"), "processId", "number");

    const adjacentX = await publish(65, 64, ADJACENT_X);
    const adjacentZ = await publish(65, 65, ADJACENT_Z);
    const revisit = await publish(64, 64, REVISIT);
    await followCamera(64, 64);
    const revisitEvidence = await captureResident("revisit");
    same(revisitEvidence, initialEvidence, "cached revisit evidence");
    const teleport = await publish(96, 96, TELEPORT);
    await followCamera(96, 96);
    const teleportProbes = await measureProbes();

    await lifecycle("restart");
    await event("cooked.open", { path: packA });
    const restarted = await publish(64, 64, INITIAL);
    await followCamera(64, 64);
    const restartEvidence = await captureResident("restart");
    same(restartEvidence, initialEvidence, "restart resident evidence");
    const restartedProcess = field<number>(await event("workbench.status"), "processId", "number");
    if (restartedProcess === firstProcess) fail("cooked-region: process identity survived restart");

    await event("cooked.open", { path: corruptPack });
    const copyBeforeFailure = object(await event("async.status"), "copy");
    await schedule(96, 96, {
        ...TELEPORT,
        evicted: 0,
        resident: 50,
    });
    await event("workbench.resume");
    let failedStatus: Record<string, unknown> | undefined;
    const failureDeadline = Date.now() + 5_000;
    while (Date.now() < failureDeadline) {
        const status = await event("cooked.status");
        if (status.pending === null && status.lastError !== null) {
            failedStatus = status;
            break;
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    await event("workbench.pause");
    if (!failedStatus) fail("cooked-region: corrupt payload failure was not observed");
    const failure = object(failedStatus, "lastError");
    const failureIo = object(failure, "io");
    if (
        !field<string>(failure, "message", "string").includes("checksum mismatch") ||
        field<number>(failureIo, "chunkCount", "number") !== 1 ||
        field<number>(failureIo, "payloadBytes", "number") !== 20_480
    ) fail("cooked-region: corrupt payload failure evidence mismatch");
    const asyncAfterFailure = await event("async.status");
    const publishedAfterFailure = object(asyncAfterFailure, "published");
    if (
        publishedAfterFailure.activeCenterX !== 64 || publishedAfterFailure.activeCenterZ !== 64 ||
        asyncAfterFailure.reservation !== null || asyncAfterFailure.pending !== null ||
        JSON.stringify(object(asyncAfterFailure, "copy")) !== JSON.stringify(copyBeforeFailure)
    ) fail("cooked-region: failed I/O mutated GPU publication state");
    await followCamera(64, 64);
    const failureEvidence = await captureResident("failure-preserved");
    same(failureEvidence, initialEvidence, "failure-preserved resident evidence");

    await event("load.disable");
    await event("camera.reset");
    const calibration = await captureCalibration("calibration");
    same(calibration, heldCalibration, "final calibration evidence");

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        cookedRevision: "cooked-region-v1",
        cooker: {
            packSha256,
            corruptSha256,
            metadata,
            indexSha256: field<string>(metadata, "indexSha256", "string"),
        },
        processes: { first: firstProcess, restarted: restartedProcess },
        held: {
            gateFence: gate.gateFence,
            beforeFrame,
            afterFrame,
            advancedFrames: afterFrame - beforeFrame,
            busyError,
            calibration: heldCalibration,
        },
        transactions: { initial, adjacentX, adjacentZ, revisit, teleport, restarted },
        probes: { initial: initialProbes, teleport: teleportProbes },
        residentEvidence: initialEvidence,
        failure: { status: failure, evidence: failureEvidence, async: asyncAfterFailure },
        calibration,
    };
} finally {
    await lifecycle("stop");
}

const cleanup = await sidecar(["status"]);
if (field<boolean>(object(cleanup.value, "runtime"), "running", "boolean")) {
    fail("cooked-region: broker remains running");
}
for (const target of array(cleanup.value, "targets") as Record<string, unknown>[]) {
    if (field<boolean>(target, "running", "boolean")) fail("cooked-region: target remains running");
}
if (!finalReport) fail("cooked-region: no acceptance report was produced");
await Deno.mkdir(reportPath.replace(/[\\/][^\\/]+$/, ""), { recursive: true });
await Deno.writeTextFile(reportPath, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));
