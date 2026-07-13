import {
    captureEvidence,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    object,
    same,
    stableProbe as stableTerrainProbe,
    validateProbe as validateTerrainProbe,
} from "../support/terrain.ts";
import { validateProbe as validateSkeletalProbe } from "../support/skeletal-crowds.ts";
import {
    compositionTimings,
    REVISION,
    sleep,
    stableCompositionProbe,
    validateCompositionProbe,
} from "../support/composition.ts";

const SAMPLE_COUNT = 32;
const WARMUP_MS = 250;
const output = "out/terrain/0015-atomic-terrain-object-composition/terrain.wlt";
const corruptPath = "out/terrain/0015-atomic-terrain-object-composition/terrain-corrupt.wlt";
const reportPath = "out/captures/0015-atomic-terrain-object-composition/acceptance.json";

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :composition");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);
const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
let sidecarConfig = "sidecar.toml";

async function sidecar(args: string[]): Promise<Record<string, unknown>> {
    for (let attempt = 0; attempt < 3; attempt += 1) {
        const result = await new Deno.Command("sidecar", {
            args: [...args, "--config", sidecarConfig, "--format", "json"],
            cwd: root,
            stdout: "piped",
            stderr: "piped",
        }).output();
        const stdout = decoder.decode(result.stdout).trim();
        if (stdout) {
            try {
                const value = JSON.parse(stdout) as Record<string, unknown>;
                if (!result.success || value.ok === false) {
                    fail(`sidecar ${args[0]} failed: ${JSON.stringify(value.error)}`);
                }
                return value;
            } catch (error) {
                if (attempt === 2) fail(`sidecar returned invalid JSON: ${error}: ${stdout}`);
            }
        }
        await sleep(50);
    }
    fail("sidecar retry loop exhausted");
}

async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    const result = await new Deno.Command("sidecar", {
        args: [verb, "--config", sidecarConfig],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!result.success) fail(`sidecar ${verb} failed for ${sidecarConfig}`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    return object(await sidecar(["inspect", "workbench", verb, JSON.stringify(payload)]), "data");
}

async function cook(): Promise<Record<string, unknown>> {
    const result = await new Deno.Command("cargo", {
        args: ["run", "--locked", "--release", "-p", "terrain-cooker", "--", output],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!result.success) fail("terrain cooker failed");
    return JSON.parse(decoder.decode(result.stdout).trim());
}

function configMatches(
    actual: Record<string, unknown>,
    expected: Record<string, number>,
): boolean {
    return actual.worldRegionSide === expected.world_region_side &&
        actual.activeCenterX === expected.active_center_x &&
        actual.activeCenterZ === expected.active_center_z &&
        actual.activeRadius === expected.active_radius;
}

async function waitPair(
    expected: Record<string, number>,
    minimumPublication = 0,
): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        if (status.pending === null && status.published) {
            const published = object(status, "published");
            if (
                configMatches(object(published, "config"), expected) &&
                field<number>(published, "publicationCount", "number") >= minimumPublication
            ) return status;
        }
        await sleep(10);
    }
    fail("composition publication timed out");
}

async function waitTerrainPublished(expected: Record<string, number>): Promise<void> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const renderer = object(status, "renderer");
        if (renderer.published && object(status, "stream").pending === null) {
            if (configMatches(object(object(renderer, "published"), "config"), expected)) return;
        }
        await sleep(10);
    }
    fail("standalone terrain publication timed out");
}

async function waitAsyncPublished(expected: Record<string, number>): Promise<void> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("async.status");
        if (status.pending === null && status.published) {
            if (configMatches(status.published as Record<string, unknown>, expected)) return;
        }
        await sleep(10);
    }
    fail("standalone instance publication timed out");
}

async function publish(
    workload: Record<string, number>,
): Promise<Record<string, unknown>> {
    const before = await event("composition.status");
    const previous = before.published
        ? field<number>(object(before, "published"), "publicationCount", "number")
        : 0;
    const scheduled = await event("composition.schedule", workload);
    await event("workbench.resume");
    const status = await waitPair(workload, previous + 1);
    await event("workbench.pause");
    return { scheduled, published: object(status, "published") };
}

async function prepare(workload = loadConfig()): Promise<void> {
    await event("terrain.open", { path: output });
    await publish(workload);
    await event("skeletal.configure", {
        animated_percent: 100,
        bone_count: 64,
        phase_count: 64,
        time_tick: 0,
        unique_poses: false,
        forced_lod: null,
    });
    await event("composition.enable");
    await event("composition.order", { order: "terrain-first" });
    await event("camera.reset");
    await event("workbench.pause");
}

async function probe(requireVisible = true): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateCompositionProbe(value, requireVisible);
    return value;
}

async function capture(id: string, requireBoth = true): Promise<Record<string, unknown>> {
    const value = await event("perception.capture", {
        id,
        collection: "0015-atomic-terrain-object-composition",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const evidence = captureEvidence(value);
    const visible = object(object(object(value, "perception"), "evidence"), "fullFrame")
        .objects;
    if (!Array.isArray(visible)) fail("composition capture omitted semantic objects");
    const kinds = new Set(
        visible.map((entry) => object({ entry }, "entry").kind),
    );
    if (requireBoth && (!kinds.has("terrain-region") || !kinds.has("region-proxy"))) {
        fail("composition frame does not contain both semantic classes");
    }
    return evidence;
}

async function hold(
    kind: "terrain-io" | "terrain-copy" | "instance-copy",
    destination: Record<string, number>,
): Promise<Record<string, unknown>> {
    const beforeProbe = stableCompositionProbe(await probe());
    const beforeCapture = await capture(`${kind}-before`);
    const gate = kind === "instance-copy"
        ? "async.gate"
        : kind === "terrain-io"
        ? "terrain.io_gate"
        : "terrain.copy_gate";
    await event(`${gate}.arm`);
    await event("composition.schedule", destination);
    await event("workbench.resume");
    const deadline = Date.now() + 10_000;
    let heldStatus: Record<string, unknown> | undefined;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        if (status.pending) {
            const pending = object(status, "pending");
            const reached = kind === "instance-copy"
                ? pending.terrainStage === "staged" && pending.instanceStage === "in-flight"
                : pending.instanceStage === "staged" && pending.terrainStage === "in-flight";
            if (reached) {
                heldStatus = status;
                break;
            }
        }
        await sleep(10);
    }
    if (!heldStatus) fail(`${kind} did not reach its one-half-ready state`);
    await event("workbench.pause");
    const heldProbe = stableCompositionProbe(await probe());
    const heldCapture = await capture(`${kind}-held`);
    same(heldProbe, beforeProbe, `${kind} old pair probe`);
    same(heldCapture, beforeCapture, `${kind} old pair attachments`);
    await event(`${gate}.release`);
    await event("workbench.resume");
    const published = await waitPair(destination);
    await event("workbench.pause");
    return { beforeProbe, beforeCapture, heldStatus, heldProbe, heldCapture, published };
}

async function waitRollback(): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        if (status.pending === null && status.lastFailure) return status;
        await sleep(10);
    }
    fail("composition rollback timed out");
}

async function captureCameras(): Promise<Record<string, unknown>[]> {
    const cameras = [
        { name: "default", position: [9, 6, 12], target: [0, 1, -3] },
        { name: "near-contact", position: [0, 2.5, 44], target: [0, 0, 38] },
        { name: "ridge", position: [-20, 5, 44], target: [-20, 0, 35] },
        { name: "valley", position: [20, 4, 44], target: [20, -1, 35] },
        { name: "region-edge", position: [44, 4, 0], target: [38, 0, 0] },
        { name: "four-region-corner", position: [44, 6, 44], target: [38, 0, 38] },
        { name: "grazing", position: [0, 1, 46], target: [0, 0, 34] },
    ];
    const evidence = [];
    for (const camera of cameras) {
        await event("camera.set_pose", {
            position: camera.position,
            target: camera.target,
            vertical_fov_degrees: 60,
        });
        evidence.push({ camera, capture: await capture(camera.name, false) });
    }
    await event("camera.reset");
    return evidence;
}

async function measuredComposition(order: "terrain-first" | "object-first") {
    await event("composition.order", { order });
    await event("workbench.resume");
    await sleep(WARMUP_MS);
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    const publicationMs = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const result = await publish(loadConfig());
        publicationMs.push(
            field<number>(object(result, "published"), "publicationMs", "number"),
        );
        samples.push(await probe());
    }
    const stable = stableCompositionProbe(samples[0]);
    for (const sample of samples.slice(1)) {
        same(stableCompositionProbe(sample), stable, `${order} composition evidence`);
    }
    return {
        order,
        warmupMs: WARMUP_MS,
        evidence: stable,
        timing: compositionTimings(samples),
        publicationMs: distribution(publicationMs),
    };
}

async function measuredTerrainOnly(): Promise<Record<string, unknown>> {
    await lifecycle("restart");
    await event("terrain.open", { path: output });
    await event("terrain.schedule", loadConfig());
    await event("workbench.resume");
    await waitTerrainPublished(loadConfig());
    await event("terrain.enable");
    await sleep(WARMUP_MS);
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const sample = await event("load.probe");
        validateTerrainProbe(sample);
        samples.push(sample);
    }
    const timing = (name: string) =>
        distribution(
            samples.map((sample) => field<number>(object(sample, "timing"), name, "number")),
        );
    return {
        sampleCount: samples.length,
        evidence: stableTerrainProbe(samples[0]),
        seamMs: timing("seamMs"),
        rasterMs: timing("rasterMs"),
        totalMs: timing("totalMs"),
    };
}

async function measuredSkeletalOnly(): Promise<Record<string, unknown>> {
    await lifecycle("restart");
    await event("async.schedule", loadConfig());
    await event("workbench.resume");
    await waitAsyncPublished(loadConfig());
    await event("skeletal.configure", {
        animated_percent: 100,
        bone_count: 64,
        phase_count: 64,
        time_tick: 0,
        unique_poses: false,
        forced_lod: null,
    });
    await event("skeletal.enable");
    await sleep(WARMUP_MS);
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const sample = await event("load.probe");
        validateSkeletalProbe(sample);
        samples.push(sample);
    }
    const timing = (name: string) =>
        distribution(samples.map((sample) => field<number>(sample, name, "number")));
    return {
        sampleCount: samples.length,
        evidence: {
            gpu: object(samples[0], "gpu"),
            cpuOracle: object(samples[0], "cpuOracle"),
        },
        cullClassifyMs: timing("gpuCullClassifyMs"),
        poseCompactMs: timing("gpuPoseCompactMs"),
        poseEvaluateMs: timing("gpuPoseEvaluateMs"),
        meshSkinMs: timing("gpuMeshSkinMs"),
        totalMs: timing("gpuTotalMs"),
    };
}

await lifecycle("stop");
sidecarConfig = "sidecar.benchmark.toml";
await lifecycle("stop");
sidecarConfig = "sidecar.toml";
const cooked = await cook();
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;
try {
    await lifecycle("start");
    await prepare();
    const status = await event("workbench.status");
    const renderer = object(status, "renderer");
    const firstProcess = field<number>(status, "processId", "number");
    if (renderer.debugLayer !== true || renderer.deviceRemovedReason !== null) {
        fail("debug composition capability gate failed");
    }
    const canonicalProbe = await probe();
    const canonicalCapture = await capture("canonical-terrain-first");
    await event("composition.order", { order: "object-first" });
    const objectFirstCapture = await capture("canonical-object-first");
    same(objectFirstCapture, canonicalCapture, "composition render-order attachments");
    await event("composition.order", { order: "terrain-first" });
    const cameras = await captureCameras();

    const movement = [];
    for (const [x, z] of [[65, 64], [65, 65], [64, 64], [96, 96]]) {
        const transaction = await publish(loadConfig(x, z));
        movement.push({
            x,
            z,
            transaction,
            probe: stableCompositionProbe(await probe(x !== 96 || z !== 96)),
        });
    }
    await publish(loadConfig());
    same(stableCompositionProbe(await probe()), stableCompositionProbe(canonicalProbe), "revisit");

    const instanceHold = await hold("instance-copy", loadConfig(65, 64));
    const terrainIoHold = await hold("terrain-io", loadConfig(65, 65));
    const terrainCopyHold = await hold("terrain-copy", loadConfig(64, 64));

    await Deno.copyFile(output, corruptPath);
    await event("terrain.open", { path: corruptPath });
    const beforeFailureProbe = stableCompositionProbe(await probe());
    const beforeFailureCapture = await capture("corruption-before");
    const bytes = await Deno.readFile(corruptPath);
    bytes[bytes.length - 1] ^= 1;
    await Deno.writeFile(corruptPath, bytes);
    await event("composition.schedule", loadConfig(96, 96));
    await event("workbench.resume");
    const failure = await waitRollback();
    await event("workbench.pause");
    const failedProbe = stableCompositionProbe(await probe());
    const failedCapture = await capture("corruption-held");
    same(failedProbe, beforeFailureProbe, "corrupt pair probe rollback");
    same(failedCapture, beforeFailureCapture, "corrupt pair attachment rollback");
    bytes[bytes.length - 1] ^= 1;
    await Deno.writeFile(corruptPath, bytes);
    const retry = await publish(loadConfig(96, 96));
    const retryProbe = await probe(false);

    await lifecycle("restart");
    await prepare();
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("composition process survived restart");
    const restartProbe = await probe();
    const restartCapture = await capture("restart");
    same(
        stableCompositionProbe(restartProbe),
        stableCompositionProbe(canonicalProbe),
        "restart probe",
    );
    same(restartCapture, canonicalCapture, "restart attachments");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    const terrainOnly = await measuredTerrainOnly();
    const skeletalOnly = await measuredSkeletalOnly();
    await lifecycle("restart");
    await prepare();
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release composition capability gate failed");
    }
    const terrainFirst = await measuredComposition("terrain-first");
    const objectFirst = await measuredComposition("object-first");
    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        cooked,
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        canonical: {
            probe: canonicalProbe,
            terrainFirst: canonicalCapture,
            objectFirst: objectFirstCapture,
        },
        cameras,
        movement,
        holds: {
            instanceCopy: instanceHold,
            terrainIo: terrainIoHold,
            terrainCopy: terrainCopyHold,
        },
        corruption: {
            failure,
            beforeFailureProbe,
            beforeFailureCapture,
            failedProbe,
            failedCapture,
            retry,
            retryProbe,
        },
        restart: { probe: restartProbe, capture: restartCapture },
        benchmark: { terrainOnly, skeletalOnly, terrainFirst, objectFirst },
    };
} finally {
    await lifecycle("stop");
    sidecarConfig = "sidecar.toml";
    await lifecycle("stop");
}

if (!finalReport) fail("composition experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/0015-atomic-terrain-object-composition`, {
    recursive: true,
});
await Deno.writeTextFile(`${root}/${reportPath}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: "pass", report: reportPath }, null, 2));
