import {
    ANIMATION_SHA256,
    array,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    MESH_SHA256,
    object,
    REVISION,
    same,
    type Settings,
    settings,
    validateProbe,
} from "../support/skeletal-crowds.ts";

async function sidecar(
    args: string[],
): Promise<{ success: boolean; value: Record<string, unknown> }> {
    const output = await new Deno.Command("sidecar", {
        args: [...args, "--config", sidecarConfig, "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    if (!stdout) fail(`sidecar ${args[0]} returned no JSON`);
    try {
        return { success: output.success, value: JSON.parse(stdout) };
    } catch {
        fail(`sidecar returned invalid JSON: ${stdout}`);
    }
}

async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [verb, "--config", sidecarConfig],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`sidecar ${verb} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]);
    if (!response.success || response.value.ok !== true) {
        fail(`${verb} failed: ${String(response.value.error)}`);
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
    if (!output.success) fail("region cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim());
}

async function waitPublished(expected: Record<string, number>): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        const status = await event("async.status");
        if (status.pending === null && typeof status.published === "object" && status.published) {
            const published = status.published as Record<string, unknown>;
            if (
                published.worldRegionSide === expected.world_region_side &&
                published.activeCenterX === expected.active_center_x &&
                published.activeCenterZ === expected.active_center_z &&
                published.activeRadius === expected.active_radius
            ) return status;
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail("cooked publication timed out");
}

async function publish(workload: Record<string, number>): Promise<Record<string, unknown>> {
    const report = await event("cooked.schedule", workload);
    await event("workbench.resume");
    const status = await waitPublished(workload);
    await event("workbench.pause");
    report.publication = object(status, "lastCompleted");
    return report;
}

async function followCamera(x = 64, z = 64): Promise<void> {
    const worldX = (x - 64) * 16;
    const worldZ = (z - 64) * 16;
    await event("camera.set_pose", {
        position: [worldX, 30, worldZ + 30],
        target: [worldX, 0, worldZ],
        vertical_fov_degrees: 60,
    });
}

async function configure(value: Settings): Promise<void> {
    await event("skeletal.configure", value);
}

async function probe(): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateProbe(value);
    return value;
}

async function measuredProbes(): Promise<Record<string, unknown>> {
    await event("workbench.resume");
    await new Promise((resolve) => setTimeout(resolve, WARMUP_MS));
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) samples.push(await probe());
    const counts = object(samples[0], "gpu");
    for (const sample of samples.slice(1)) same(object(sample, "gpu"), counts, "probe counts");
    const timing = (name: string) =>
        distribution(
            samples.map((sample) => field<number>(sample, name, "number")),
            name,
            name === "gpuPoseEvaluateMs" && counts.activePoses === 0,
        );
    return {
        warmupMs: WARMUP_MS,
        sampleCount: samples.length,
        settings: samples[0].settings,
        counts,
        gpuCullClassifyMs: timing("gpuCullClassifyMs"),
        gpuPoseCompactMs: timing("gpuPoseCompactMs"),
        gpuPoseEvaluateMs: timing("gpuPoseEvaluateMs"),
        gpuMeshSkinMs: timing("gpuMeshSkinMs"),
        gpuTotalMs: timing("gpuTotalMs"),
    };
}

async function correctnessSweep(
    name: string,
    values: Settings[],
): Promise<Record<string, unknown>[]> {
    const results: Record<string, unknown>[] = [];
    for (const value of values) {
        await configure(value);
        results.push({ name, value, probe: await probe() });
    }
    return results;
}

async function timingSweep(name: string, values: Settings[]): Promise<Record<string, unknown>[]> {
    const results: Record<string, unknown>[] = [];
    for (const value of values) {
        await configure(value);
        results.push({ name, value, measurements: await measuredProbes() });
    }
    return results;
}

async function captureEvidence(id: string): Promise<Record<string, unknown>> {
    const color = await event("workbench.capture", {
        id: `${id}-color`,
        collection: "0010-gpu-skeletal-crowds",
    });
    const ids = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0010-gpu-skeletal-crowds",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }, { x: 600, y: 600 }],
    });
    for (const manifest of [color, ids]) {
        if (
            manifest.lastError !== null || object(manifest, "renderer").deviceRemovedReason !== null
        ) {
            fail(`${id} reported a renderer failure`);
        }
        const workload = object(manifest, "workload");
        if (
            field<string>(workload, "mode", "string") !== "async-resident-load" ||
            field<boolean>(object(workload, "skeletal"), "enabled", "boolean") !== true
        ) fail(`${id} workload mode mismatch`);
    }
    const perception = object(ids, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`${id} has unknown IDs`);
    const fullFrame = object(evidence, "fullFrame");
    const visible = array(fullFrame, "objects") as Record<string, unknown>[];
    if (
        visible.length === 0 ||
        visible.some((entry) =>
            entry.kind !== "region-proxy" || !String(entry.name).startsWith("load.region.")
        )
    ) fail(`${id} semantic joins are invalid`);
    return {
        colorSha256: field<string>(object(color, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(perception, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(perception, "diagnosticPngSha256", "string"),
        visibleRegionCount: visible.length,
        backgroundPixelCount: field<number>(fullFrame, "backgroundPixelCount", "number"),
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :skeletal-crowds");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const output = "out/cooked/0010-gpu-skeletal-crowds/regions.wlr";
const reportPath = `${root}/out/captures/0010-gpu-skeletal-crowds/acceptance.json`;
const WARMUP_MS = 250;
const PREHEAT_MS = 2_000;
const SAMPLE_COUNT = 32;
let sidecarConfig = "sidecar.toml";
let finalReport: Record<string, unknown> | undefined;

await lifecycle("stop");
try {
    await Deno.remove(reportPath);
} catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) throw error;
}
const cooked = await cook(output);
const environment = await collectEnvironment(root);

try {
    await lifecycle("start");
    await event("cooked.open", { path: output });
    const initialTransaction = await publish(loadConfig());
    await followCamera();
    const baselineSettings = settings();
    await configure(baselineSettings);
    await event("skeletal.enable");

    const status = await event("workbench.status");
    const renderer = object(status, "renderer");
    if (
        field<number>(renderer, "meshShaderTier", "number") < 1 ||
        Number(field<string>(renderer, "shaderModel", "string")) < 6.6 ||
        renderer.deviceRemovedReason !== null
    ) fail("reference capability gate failed");
    const skeletalStatus = await event("skeletal.status");
    const catalog = object(skeletalStatus, "catalog");
    same(catalog, {
        meshletSha256: MESH_SHA256,
        animationSha256: ANIMATION_SHA256,
        boneCount: 128,
        clipCount: 8,
        sampleCountPerClip: 64,
        skinBindingCount: 1_872,
        meshletGpuBytes: 57_152,
        animationGpuBytes: 3_169_920,
    }, "catalog contract");
    same(object(skeletalStatus, "resources"), {
        visibleCapacity: 25_600,
        sharedPoseCapacity: 512,
        uniquePoseCapacity: 25_600,
        paletteBytes: 157_286_400,
        executionBytes: 158_005_616,
    }, "resource contract");

    const initialProbe = await probe();
    same(object(initialProbe, "gpu"), {
        visible: 18_928,
        rejected: 6_672,
        animated: 18_928,
        staticCount: 0,
        activePoses: 512,
        reusedPoses: 18_416,
        evaluatedBones: 32_768,
        lodCounts: [6_243, 12_640, 45],
        meshlets: 69_270,
        emittedVertices: 2_399_960,
        emittedTriangles: 3_316_944,
        skinInfluences: 9_599_840,
        observedArchetypeMask: 255,
    }, "canonical counters");
    const initialEvidence = await captureEvidence("initial-time-0");
    const firstProcess = field<number>(status, "processId", "number");

    await configure(settings({ time_tick: 11 }));
    const time11Probe = await probe();
    const time11Evidence = await captureEvidence("time-11");
    if (time11Evidence.colorSha256 === initialEvidence.colorSha256) {
        fail("time 11 did not change the color capture");
    }
    await configure(baselineSettings);
    const returnedProbe = await probe();
    same(object(returnedProbe, "gpu"), object(initialProbe, "gpu"), "returned time counters");
    const returnedEvidence = await captureEvidence("returned-time-0");
    same(returnedEvidence, initialEvidence, "returned time evidence");

    const animatedSweep = await correctnessSweep(
        "animatedPercent",
        [0, 25, 50, 100].map((value) => settings({ animated_percent: value })),
    );
    const boneSweep = await correctnessSweep(
        "boneCount",
        [16, 32, 64, 128].map((value) => settings({ bone_count: value })),
    );
    const phaseSweep = await correctnessSweep(
        "phaseCount",
        [1, 8, 64].map((value) => settings({ phase_count: value })),
    );
    const uniqueSweep = await correctnessSweep("unique128", [
        settings({ bone_count: 128, unique_poses: true }),
    ]);
    const lodSweep = await correctnessSweep(
        "forcedLod",
        [0, 1, 2].map((value) => settings({ forced_lod: value })),
    );

    await configure(baselineSettings);
    const adjacentTransaction = await publish(loadConfig(65, 64));
    await followCamera(65, 64);
    const adjacentProbe = await probe();
    const revisitTransaction = await publish(loadConfig());
    await followCamera();
    const revisitProbe = await probe();
    same(object(revisitProbe, "gpu"), object(initialProbe, "gpu"), "revisit counters");
    const revisitEvidence = await captureEvidence("revisit");
    same(revisitEvidence, initialEvidence, "revisit evidence");

    await lifecycle("restart");
    await event("cooked.open", { path: output });
    const restartTransaction = await publish(loadConfig());
    await followCamera();
    await configure(baselineSettings);
    await event("skeletal.enable");
    const restartProbe = await probe();
    same(object(restartProbe, "gpu"), object(initialProbe, "gpu"), "restart counters");
    const restartEvidence = await captureEvidence("restart");
    same(restartEvidence, initialEvidence, "restart evidence");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("process survived restart");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await event("cooked.open", { path: output });
    const benchmarkTransaction = await publish(loadConfig());
    await followCamera();
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (
        benchmarkRenderer.debugLayer !== false ||
        benchmarkRenderer.deviceRemovedReason !== null
    ) fail("release benchmark capability gate failed");
    const benchmarkSkeletalStatus = await event("skeletal.status");
    same(object(benchmarkSkeletalStatus, "catalog"), catalog, "benchmark catalog");
    same(
        object(benchmarkSkeletalStatus, "resources"),
        object(skeletalStatus, "resources"),
        "benchmark resources",
    );
    await configure(settings({ bone_count: 128, unique_poses: true }));
    await event("skeletal.enable");
    await event("workbench.resume");
    await new Promise((resolve) => setTimeout(resolve, PREHEAT_MS));
    await event("workbench.pause");
    await configure(baselineSettings);
    const benchmarkProbe = await probe();
    const benchmarkInitial = await measuredProbes();
    const benchmarkAnimated = await timingSweep(
        "animatedPercent",
        [0, 25, 50, 100].map((value) => settings({ animated_percent: value })),
    );
    const benchmarkBones = await timingSweep(
        "boneCount",
        [16, 32, 64, 128].map((value) => settings({ bone_count: value })),
    );
    const benchmarkPhases = await timingSweep(
        "phaseCount",
        [1, 8, 64].map((value) => settings({ phase_count: value })),
    );
    const benchmarkUnique = await timingSweep("unique128", [
        settings({ bone_count: 128, unique_poses: true }),
    ]);
    const benchmarkLod = await timingSweep(
        "forcedLod",
        [0, 1, 2].map((value) => settings({ forced_lod: value })),
    );
    const benchmarkProcess = field<number>(benchmarkStatus, "processId", "number");

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        capability: {
            correctness: {
                adapter: renderer.adapter,
                featureLevel: renderer.featureLevel,
                meshShaderTier: renderer.meshShaderTier,
                shaderModel: renderer.shaderModel,
                debugLayer: renderer.debugLayer,
            },
            benchmark: {
                adapter: benchmarkRenderer.adapter,
                featureLevel: benchmarkRenderer.featureLevel,
                meshShaderTier: benchmarkRenderer.meshShaderTier,
                shaderModel: benchmarkRenderer.shaderModel,
                debugLayer: benchmarkRenderer.debugLayer,
            },
        },
        catalog,
        resources: skeletalStatus.resources,
        submission: skeletalStatus.submission,
        cooked,
        processes: {
            first: firstProcess,
            restarted: restartedProcess,
            benchmark: benchmarkProcess,
        },
        initial: {
            transaction: initialTransaction,
            probe: initialProbe,
            evidence: initialEvidence,
        },
        deterministicTime: {
            time11: { probe: time11Probe, evidence: time11Evidence },
            returnedTime0: { probe: returnedProbe, evidence: returnedEvidence },
        },
        sweeps: {
            animated: animatedSweep,
            bones: boneSweep,
            phases: phaseSweep,
            unique: uniqueSweep,
            lod: lodSweep,
        },
        movement: {
            adjacent: { transaction: adjacentTransaction, probe: adjacentProbe },
            revisit: {
                transaction: revisitTransaction,
                probe: revisitProbe,
                evidence: revisitEvidence,
            },
        },
        restart: {
            transaction: restartTransaction,
            probe: restartProbe,
            evidence: restartEvidence,
        },
        benchmark: {
            preheat: {
                milliseconds: PREHEAT_MS,
                settings: settings({ bone_count: 128, unique_poses: true }),
            },
            transaction: benchmarkTransaction,
            probe: benchmarkProbe,
            initial: benchmarkInitial,
            sweeps: {
                animated: benchmarkAnimated,
                bones: benchmarkBones,
                phases: benchmarkPhases,
                unique: benchmarkUnique,
                lod: benchmarkLod,
            },
        },
    };
} finally {
    await lifecycle("stop");
}

for (const config of ["sidecar.toml", "sidecar.benchmark.toml"]) {
    sidecarConfig = config;
    const cleanup = await sidecar(["status"]);
    if (field<boolean>(object(cleanup.value, "runtime"), "running", "boolean")) {
        fail(`${config} broker remains running`);
    }
    for (const target of array(cleanup.value, "targets") as Record<string, unknown>[]) {
        if (field<boolean>(target, "running", "boolean")) fail(`${config} target remains running`);
    }
}
if (!finalReport) fail("no acceptance report was produced");
await Deno.mkdir(reportPath.replace(/[\\/][^\\/]+$/, ""), { recursive: true });
await Deno.writeTextFile(reportPath, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));
