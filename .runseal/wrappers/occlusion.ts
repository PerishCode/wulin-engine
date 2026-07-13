import {
    array,
    assertStableQueried,
    baselineSkeletalSettings,
    collectEnvironment,
    distribution,
    fail,
    field,
    HIGH_OCCLUSION_CAMERA,
    loadConfig,
    object,
    OCCLUSION_REVISION,
    same,
    stableOcclusionEvidence,
    stableProbeEvidence,
    surfaceSettings,
    validateProbe,
} from "../support/occlusion.ts";

async function sidecar(
    args: string[],
): Promise<{ success: boolean; value: Record<string, unknown> }> {
    for (let attempt = 0; attempt < 3; attempt += 1) {
        const output = await new Deno.Command("sidecar", {
            args: [...args, "--config", sidecarConfig, "--format", "json"],
            cwd: root,
            stdout: "piped",
            stderr: "piped",
        }).output();
        const stdout = decoder.decode(output.stdout).trim();
        if (stdout) {
            try {
                return { success: output.success, value: JSON.parse(stdout) };
            } catch {
                if (attempt === 2) fail(`sidecar returned invalid JSON: ${stdout}`);
            }
        } else if (attempt === 2) {
            fail(`sidecar ${args[0]} returned no JSON after three attempts`);
        }
        await new Promise((resolve) => setTimeout(resolve, 50));
    }
    fail("sidecar retry loop exhausted");
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
    const response = await sidecar(["inspect", "workbench", verb, JSON.stringify(payload)]);
    if (!response.success || response.value.ok !== true) {
        fail(`${verb} failed: ${JSON.stringify(response.value.error)}`);
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
        const published = status.published;
        if (status.pending === null && typeof published === "object" && published) {
            const value = published as Record<string, unknown>;
            if (
                value.worldRegionSide === expected.world_region_side &&
                value.activeCenterX === expected.active_center_x &&
                value.activeCenterZ === expected.active_center_z &&
                value.activeRadius === expected.active_radius
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

async function setCamera(camera: Record<string, unknown>): Promise<void> {
    await event("camera.set_pose", camera);
}

async function followCamera(x = 64, z = 64): Promise<void> {
    const worldX = (x - 64) * 16;
    const worldZ = (z - 64) * 16;
    await setCamera({
        position: [worldX, 30, worldZ + 30],
        target: [worldX, 0, worldZ],
        vertical_fov_degrees: 60,
    });
}

async function configureSkeletal(value = baselineSkeletalSettings()): Promise<void> {
    await event("skeletal.configure", value);
}

async function prepare(path: string, workload = loadConfig()): Promise<void> {
    await event("cooked.open", { path });
    await publish(workload);
    await followCamera(workload.active_center_x, workload.active_center_z);
    await configureSkeletal();
    await event("surface.configure", surfaceSettings());
    await event("surface.enable");
    await event("workbench.pause");
}

async function probe(requireFullMaterialCoverage = true): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateProbe(value, requireFullMaterialCoverage);
    return value;
}

function assertBypass(probeValue: Record<string, unknown>, reason?: string): void {
    const occlusion = object(probeValue, "occlusion");
    if (
        field<boolean>(occlusion, "historyQueried", "boolean") ||
        field<number>(occlusion, "occluded", "number") !== 0 ||
        field<number>(occlusion, "bypassed", "number") !==
            field<number>(occlusion, "sourceVisible", "number")
    ) fail("occlusion invalidation frame did not bypass the complete source list");
    if (reason && field<string>(occlusion, "bypassReason", "string") !== reason) {
        fail(`occlusion bypass reason differs from ${reason}`);
    }
}

function assertQueried(probeValue: Record<string, unknown>, minimumMeshletPercent = 0): void {
    const occlusion = object(probeValue, "occlusion");
    if (
        !field<boolean>(occlusion, "historyQueried", "boolean") ||
        field<number>(occlusion, "tested", "number") !==
            field<number>(occlusion, "sourceVisible", "number")
    ) fail("occlusion compatible frame did not query every source object");
    const source = field<number>(occlusion, "sourceMeshlets", "number");
    const submitted = field<number>(occlusion, "submittedMeshlets", "number");
    const eliminatedPercent = (source - submitted) * 100 / source;
    if (eliminatedPercent < minimumMeshletPercent) {
        fail(
            `occlusion eliminated ${eliminatedPercent}% meshlets, expected ${minimumMeshletPercent}%`,
        );
    }
}

async function captureEvidence(
    id: string,
    probeValue: Record<string, unknown>,
): Promise<Record<string, unknown>> {
    const capture = await event("workbench.capture", {
        id: `${id}-color`,
        collection: "0012-gpu-conservative-occlusion",
    });
    if (capture.lastError !== null || object(capture, "renderer").deviceRemovedReason !== null) {
        fail(`${id} color capture reported a renderer failure`);
    }
    const perception = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0012-gpu-conservative-occlusion",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }, { x: 600, y: 600 }],
    });
    if (
        perception.lastError !== null || object(perception, "renderer").deviceRemovedReason !== null
    ) {
        fail(`${id} semantic capture reported a renderer failure`);
    }
    const semantic = object(perception, "perception");
    if (array(object(semantic, "evidence"), "unknownIds").length !== 0) {
        fail(`${id} has unknown semantic IDs`);
    }
    return {
        colorSha256: field<string>(object(capture, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(semantic, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(semantic, "diagnosticPngSha256", "string"),
        surface: stableProbeEvidence(probeValue),
    };
}

async function pairedProbe(
    label: string,
    requireFullMaterialCoverage = true,
    minimumMeshletPercent = 0,
    capture = false,
): Promise<Record<string, unknown>> {
    await event("surface.occlusion.disable");
    await event("surface.occlusion.reset");
    const baseline = await probe(requireFullMaterialCoverage);
    assertBypass(baseline, "disabled");
    const baselineCapture = capture ? await captureEvidence(`${label}-baseline`, baseline) : null;
    await event("surface.occlusion.enable");
    const queried = await probe(requireFullMaterialCoverage);
    assertQueried(queried, minimumMeshletPercent);
    same(stableProbeEvidence(queried), stableProbeEvidence(baseline), `${label} surface output`);
    const queriedCapture = capture ? await captureEvidence(`${label}-queried`, queried) : null;
    if (capture) same(queriedCapture, baselineCapture, `${label} full attachment output`);
    return { baseline, queried, baselineCapture, queriedCapture };
}

async function invalidatedPair(
    label: string,
    mutate: () => Promise<void>,
    requireFullMaterialCoverage = true,
): Promise<Record<string, unknown>> {
    await mutate();
    const bypass = await probe(requireFullMaterialCoverage);
    assertBypass(bypass, "execution-signature-changed");
    const queried = await probe(requireFullMaterialCoverage);
    assertQueried(queried);
    same(stableProbeEvidence(queried), stableProbeEvidence(bypass), `${label} invalidated output`);
    return { bypass, queried };
}

async function measuredProbes(enabled: boolean): Promise<Record<string, unknown>> {
    if (enabled) {
        await event("surface.occlusion.enable");
        await probe(false);
    } else {
        await event("surface.occlusion.disable");
    }
    await event("workbench.resume");
    await new Promise((resolve) => setTimeout(resolve, WARMUP_MS));
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) samples.push(await probe(false));
    const evidence = stableOcclusionEvidence(samples[0]);
    for (const sample of samples.slice(1)) {
        same(stableOcclusionEvidence(sample), evidence, "occlusion timing evidence");
    }
    const timing = (name: string) =>
        distribution(samples.map((sample) => field<number>(sample, name, "number")));
    const queryTiming = distribution(
        samples.map((sample) => field<number>(object(sample, "occlusion"), "gpuQueryMs", "number")),
    );
    return {
        enabled,
        sampleCount: samples.length,
        warmupMs: WARMUP_MS,
        evidence,
        gpuQueryMs: queryTiming,
        gpuVisibilityMs: timing("gpuVisibilityMs"),
        gpuResolveMs: timing("gpuResolveMs"),
        gpuHierarchyMs: timing("gpuHierarchyMs"),
        gpuTotalMs: timing("gpuTotalMs"),
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :occlusion");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);
const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const output = "out/cooked/0012-gpu-conservative-occlusion/regions.wlr";
const reportPath = `${root}/out/captures/0012-gpu-conservative-occlusion/acceptance.json`;
const SAMPLE_COUNT = 32;
const WARMUP_MS = 250;
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
    await prepare(output);
    const status = await event("workbench.status");
    const firstProcess = field<number>(status, "processId", "number");
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== true || renderer.deviceRemovedReason !== null) {
        fail("debug occlusion capability gate failed");
    }
    const surfaceStatus = await event("surface.status");
    const occlusionStatus = object(surfaceStatus, "occlusion");
    same(object(occlusionStatus, "resources"), {
        hierarchyFormat: "R32_UINT",
        hierarchyMipCount: 11,
        hierarchyBytes: 4_915_052,
        executionBytes: 5_632_332,
        totalSurfaceExecutionBytes: 25_655_340,
    }, "occlusion resource contract");
    same(object(occlusionStatus, "submission"), {
        queryDispatchCount: 1,
        queryGroups: 100,
        prefixDispatchCount: 1,
        prefixGroups: 1,
        scatterDispatchCount: 1,
        scatterGroups: 100,
        compactionDispatchCount: 3,
        hierarchyDispatchCount: 11,
    }, "occlusion submission contract");

    const canonical = await pairedProbe("canonical", true, 0, true);
    if (
        field<number>(
            object(canonical.queried as Record<string, unknown>, "occlusion"),
            "occluded",
            "number",
        ) === 0
    ) {
        fail("canonical camera eliminated no objects");
    }
    const explicitResetStatus = await event("surface.occlusion.reset");
    if (object(explicitResetStatus, "occlusion").historyAvailable !== false) {
        fail("explicit reset retained history");
    }
    const explicitBypass = await probe();
    assertBypass(explicitBypass, "explicit-reset");
    const explicitQueried = await probe();
    same(
        stableProbeEvidence(explicitQueried),
        stableProbeEvidence(explicitBypass),
        "explicit reset output",
    );

    const highInvalidation = await invalidatedPair(
        "high-camera",
        () => setCamera(HIGH_OCCLUSION_CAMERA),
        false,
    );
    assertQueried(highInvalidation.queried as Record<string, unknown>, 25);
    const highFullPair = await pairedProbe("high-camera", false, 25, true);

    await followCamera();
    const cameraReturn = await probe();
    assertBypass(cameraReturn, "execution-signature-changed");
    const cameraReturnQuery = await probe();
    same(
        stableProbeEvidence(cameraReturnQuery),
        stableProbeEvidence(cameraReturn),
        "camera return output",
    );

    const time11 = await invalidatedPair(
        "time-11",
        () => configureSkeletal(baselineSkeletalSettings({ time_tick: 11 })),
    );
    const time0 = await invalidatedPair("time-0-return", () => configureSkeletal());
    same(
        stableProbeEvidence(time0.queried as Record<string, unknown>),
        stableProbeEvidence(canonical.queried as Record<string, unknown>),
        "returned time-0 output",
    );

    const lods = [];
    for (const forced_lod of [0, 1, 2]) {
        await configureSkeletal(baselineSkeletalSettings({ forced_lod }));
        lods.push({ forcedLod: forced_lod, pair: await pairedProbe(`lod-${forced_lod}`) });
    }
    await configureSkeletal();
    await configureSkeletal(baselineSkeletalSettings({ bone_count: 128, unique_poses: true }));
    const unique128 = await pairedProbe("unique-128", true, 0, false);
    await configureSkeletal();

    const radii = [];
    for (const radius of [0, 1, 2]) {
        await event("surface.occlusion.disable");
        const workload = loadConfig(64, 64, radius);
        const transaction = await publish(workload);
        await followCamera();
        radii.push({ radius, transaction, pair: await pairedProbe(`radius-${radius}`) });
    }
    await event("surface.occlusion.disable");
    const adjacentTransaction = await publish(loadConfig(65, 64, 2));
    await followCamera(65, 64);
    const adjacent = await pairedProbe("adjacent", true, 0, false);
    await event("surface.occlusion.disable");
    const revisitTransaction = await publish(loadConfig());
    await followCamera();
    const revisit = await pairedProbe("revisit", true, 0, false);
    same(
        stableProbeEvidence(revisit.queried as Record<string, unknown>),
        stableProbeEvidence(canonical.queried as Record<string, unknown>),
        "cached revisit output",
    );

    await lifecycle("restart");
    await prepare(output);
    await event("surface.occlusion.enable");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("workbench process survived restart");
    const restartBypass = await probe();
    assertBypass(restartBypass, "surface-enable");
    const restartQueried = await probe();
    same(
        stableProbeEvidence(restartQueried),
        stableProbeEvidence(canonical.queried as Record<string, unknown>),
        "restart queried output",
    );

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await prepare(output);
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release occlusion capability gate failed");
    }
    const benchmarkCanonicalDisabled = await measuredProbes(false);
    const benchmarkCanonicalQueried = await measuredProbes(true);
    await setCamera(HIGH_OCCLUSION_CAMERA);
    const benchmarkHighDisabled = await measuredProbes(false);
    const benchmarkHighQueried = await measuredProbes(true);
    assertStableQueried(object(benchmarkHighQueried, "evidence"), 25);

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: OCCLUSION_REVISION,
        environment,
        cooked,
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        status: { surface: surfaceStatus.resources, occlusion: occlusionStatus },
        processes: { first: firstProcess, restarted: restartedProcess },
        canonical,
        invalidation: {
            explicitBypass,
            explicitQueried,
            highInvalidation,
            cameraReturn,
            time11,
            time0,
        },
        highCamera: { camera: HIGH_OCCLUSION_CAMERA, pair: highFullPair },
        sweeps: { lods, unique128, radii },
        movement: { adjacentTransaction, adjacent, revisitTransaction, revisit },
        restart: { bypass: restartBypass, queried: restartQueried },
        benchmark: {
            sampleCount: SAMPLE_COUNT,
            warmupMs: WARMUP_MS,
            canonical: { disabled: benchmarkCanonicalDisabled, queried: benchmarkCanonicalQueried },
            high: { disabled: benchmarkHighDisabled, queried: benchmarkHighQueried },
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
if (!finalReport) fail("no occlusion acceptance report was produced");
await Deno.mkdir(reportPath.replace(/[\\/][^\\/]+$/, ""), { recursive: true });
await Deno.writeTextFile(reportPath, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({
    outcome: finalReport.outcome,
    report: "out/captures/0012-gpu-conservative-occlusion/acceptance.json",
}));
