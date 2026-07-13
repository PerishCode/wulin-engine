import {
    ANIMATION_SHA256,
    array,
    baselineSkeletalSettings,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    MESH_SHA256,
    object,
    REVISION,
    same,
    stableProbeEvidence,
    SURFACE_SHA256,
    type SurfaceSettings,
    surfaceSettings,
    validateCatalogs,
    validateProbe,
} from "../support/surface-resolve.ts";

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

async function configureSurface(value: SurfaceSettings): Promise<void> {
    await event("surface.configure", value);
}

async function configureSkeletal(value = baselineSkeletalSettings()): Promise<void> {
    await event("skeletal.configure", value);
}

async function probe(): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateProbe(value);
    validateCatalogs(value);
    return value;
}

async function measuredProbes(): Promise<Record<string, unknown>> {
    await event("workbench.resume");
    await new Promise((resolve) => setTimeout(resolve, WARMUP_MS));
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) samples.push(await probe());
    const evidence = stableProbeEvidence(samples[0]);
    for (const sample of samples.slice(1)) {
        same(stableProbeEvidence(sample), evidence, "timing probe evidence");
    }
    const timing = (name: string) =>
        distribution(samples.map((sample) => field<number>(sample, name, "number")));
    const skeletalTiming = (name: string) =>
        distribution(
            samples.map((sample) => field<number>(object(sample, "skeletal"), name, "number")),
        );
    return {
        warmupMs: WARMUP_MS,
        sampleCount: samples.length,
        settings: samples[0].settings,
        evidence,
        gpuCullClassifyMs: skeletalTiming("gpuCullClassifyMs"),
        gpuPoseEvaluateMs: skeletalTiming("gpuPoseEvaluateMs"),
        gpuVisibilityMs: timing("gpuVisibilityMs"),
        gpuResolveMs: timing("gpuResolveMs"),
        gpuTotalMs: timing("gpuTotalMs"),
    };
}

async function colorHash(id: string): Promise<string> {
    const capture = await event("workbench.capture", {
        id,
        collection: "0011-gpu-surface-resolve",
    });
    if (capture.lastError !== null || object(capture, "renderer").deviceRemovedReason !== null) {
        fail(`${id} reported a renderer failure`);
    }
    return field<string>(object(capture, "image"), "pixelSha256", "string");
}

async function captureEvidence(
    id: string,
    surfaceProbe: Record<string, unknown>,
): Promise<Record<string, unknown>> {
    const colorSha256 = await colorHash(`${id}-color`);
    const ids = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0011-gpu-surface-resolve",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }, { x: 600, y: 600 }],
    });
    const workload = object(ids, "workload");
    if (
        field<boolean>(object(workload, "surface"), "enabled", "boolean") !== true ||
        ids.lastError !== null || object(ids, "renderer").deviceRemovedReason !== null
    ) fail(`${id} surface capture state is invalid`);
    const perception = object(ids, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`${id} has unknown semantic IDs`);
    const fullFrame = object(evidence, "fullFrame");
    const visible = array(fullFrame, "objects") as Record<string, unknown>[];
    if (
        visible.length === 0 ||
        visible.some((entry) =>
            entry.kind !== "region-proxy" || !String(entry.name).startsWith("load.region.")
        )
    ) fail(`${id} semantic joins are invalid`);
    return {
        colorSha256,
        rawIdSha256: field<string>(perception, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(perception, "diagnosticPngSha256", "string"),
        visibleRegionCount: visible.length,
        surface: stableProbeEvidence(surfaceProbe),
    };
}

async function prepare(path: string, workload: Record<string, number>): Promise<void> {
    await event("cooked.open", { path });
    await publish(workload);
    await followCamera(workload.active_center_x, workload.active_center_z);
    await configureSkeletal();
    await configureSurface(surfaceSettings());
    await event("surface.enable");
}

async function surfaceSweep(
    values: SurfaceSettings[],
): Promise<{ value: SurfaceSettings; probe: Record<string, unknown> }[]> {
    const results: { value: SurfaceSettings; probe: Record<string, unknown> }[] = [];
    for (const value of values) {
        await configureSurface(value);
        results.push({ value, probe: await probe() });
    }
    return results;
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :surface-resolve");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);
const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const output = "out/cooked/0011-gpu-surface-resolve/regions.wlr";
const reportPath = `${root}/out/captures/0011-gpu-surface-resolve/acceptance.json`;
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
    await prepare(output, loadConfig());
    const status = await event("workbench.status");
    const renderer = object(status, "renderer");
    if (
        renderer.barycentrics !== true || renderer.rasterizerOrderedViews !== true ||
        renderer.visibilityFormat !== true ||
        renderer.colorUavFormat !== true || renderer.debugLayer !== true ||
        renderer.deviceRemovedReason !== null
    ) fail("debug surface capability gate failed");
    const surfaceStatus = await event("surface.status");
    same(object(surfaceStatus, "catalog"), {
        sha256: SURFACE_SHA256,
        vertexCount: 1_872,
        primitiveCount: 3_648,
        materialCount: 64,
        textureMipCount: 7,
        gpuBytes: 1_488_384,
    }, "surface catalog contract");
    same(object(surfaceStatus, "resources"), {
        width: 1_280,
        height: 720,
        visibilityFormat: "R32G32_UINT",
        colorFormat: "R8G8B8A8_UNORM",
        executionBytes: 20_023_008,
    }, "surface resource contract");
    const initialProbe = await probe();
    if (
        (array(initialProbe, "samples") as Record<string, unknown>[]).filter((sample) =>
            sample.candidateIndex !== null
        ).length < 3
    ) fail("canonical surface probe has fewer than three visible samples");
    same(object(object(initialProbe, "skeletal"), "gpu"), {
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
    }, "canonical skeletal counters");
    same(object(initialProbe, "stats"), {
        resolvedPixels: 921_600,
        visiblePixels: 720_813,
        backgroundPixels: 200_787,
        observedMaterialMask: [4_294_967_295, 4_294_967_295],
        observedMaterialCount: 64,
    }, "canonical surface counters");
    const initialEvidence = await captureEvidence("initial", initialProbe);
    const firstProcess = field<number>(status, "processId", "number");

    const materials = await surfaceSweep(
        [1, 8, 64].map((material_count) => surfaceSettings({ material_count })),
    );
    for (const entry of materials) {
        if (entry.value.material_count !== 64) {
            same(
                field<string>(entry.probe, "visibilitySha256", "string"),
                initialProbe.visibilitySha256,
                "material visibility",
            );
        }
    }
    const materialColors = [];
    for (const material_count of [1, 8, 64]) {
        await configureSurface(surfaceSettings({ material_count }));
        materialColors.push(await colorHash(`material-${material_count}`));
    }
    if (new Set(materialColors).size !== 3) fail("material sweep did not change color output");

    const mips = await surfaceSweep(
        [0, 3, 6].map((mip_level) => surfaceSettings({ mip_level })),
    );
    for (const entry of mips) {
        same(
            field<string>(entry.probe, "visibilitySha256", "string"),
            initialProbe.visibilitySha256,
            "mip visibility",
        );
    }
    const mipColors = [];
    for (const mip_level of [0, 3, 6]) {
        await configureSurface(surfaceSettings({ mip_level }));
        mipColors.push(await colorHash(`mip-${mip_level}`));
    }
    if (new Set(mipColors).size !== 3) fail("mip sweep did not change color output");

    await configureSurface(surfaceSettings());
    const lods = [];
    for (const forced_lod of [0, 1, 2]) {
        await configureSkeletal(baselineSkeletalSettings({ forced_lod }));
        lods.push({ forcedLod: forced_lod, probe: await probe() });
    }
    if (new Set(lods.map((entry) => entry.probe.visibilitySha256)).size !== 3) {
        fail("LOD sweep did not change visibility");
    }
    await configureSkeletal();
    const uniqueProbe = await (async () => {
        await configureSkeletal(baselineSkeletalSettings({ bone_count: 128, unique_poses: true }));
        return await probe();
    })();
    await configureSkeletal();

    await configureSkeletal(baselineSkeletalSettings({ time_tick: 11 }));
    const time11Probe = await probe();
    const time11Evidence = await captureEvidence("time-11", time11Probe);
    if (
        time11Evidence.colorSha256 === initialEvidence.colorSha256 ||
        field<string>(time11Probe, "visibilitySha256", "string") ===
            field<string>(initialProbe, "visibilitySha256", "string")
    ) fail("animation time did not change color and visibility");
    await configureSkeletal();
    const returnedProbe = await probe();
    const returnedEvidence = await captureEvidence("returned-time-0", returnedProbe);
    same(returnedEvidence, initialEvidence, "returned time evidence");

    const radii = [];
    for (const radius of [0, 1, 2]) {
        const workload = loadConfig(64, 64, radius);
        const transaction = await publish(workload);
        radii.push({ radius, transaction, probe: await probe() });
    }
    const adjacentTransaction = await publish(loadConfig(65, 64));
    await followCamera(65, 64);
    const adjacentProbe = await probe();
    const revisitTransaction = await publish(loadConfig());
    await followCamera();
    const revisitProbe = await probe();
    const revisitEvidence = await captureEvidence("revisit", revisitProbe);
    same(revisitEvidence, initialEvidence, "revisit evidence");

    await lifecycle("restart");
    await prepare(output, loadConfig());
    const restartProbe = await probe();
    const restartEvidence = await captureEvidence("restart", restartProbe);
    same(restartEvidence, initialEvidence, "restart evidence");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("process survived Sidecar restart");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await prepare(output, loadConfig());
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release benchmark capability gate failed");
    }
    await configureSkeletal(baselineSkeletalSettings({ bone_count: 128, unique_poses: true }));
    await event("workbench.resume");
    await new Promise((resolve) => setTimeout(resolve, PREHEAT_MS));
    await event("workbench.pause");
    await configureSkeletal();
    const benchmarkInitial = await measuredProbes();
    const benchmarkMaterials = [];
    for (const material_count of [1, 8, 64]) {
        await configureSurface(surfaceSettings({ material_count }));
        benchmarkMaterials.push({ materialCount: material_count, timing: await measuredProbes() });
    }
    const benchmarkMips = [];
    for (const mip_level of [0, 3, 6]) {
        await configureSurface(surfaceSettings({ mip_level }));
        benchmarkMips.push({ mipLevel: mip_level, timing: await measuredProbes() });
    }
    await configureSurface(surfaceSettings());
    const benchmarkLods = [];
    for (const forced_lod of [0, 1, 2]) {
        await configureSkeletal(baselineSkeletalSettings({ forced_lod }));
        benchmarkLods.push({ forcedLod: forced_lod, timing: await measuredProbes() });
    }
    const benchmarkPoses = [];
    for (
        const value of [
            baselineSkeletalSettings(),
            baselineSkeletalSettings({ bone_count: 128, unique_poses: true }),
        ]
    ) {
        await configureSkeletal(value);
        benchmarkPoses.push({ value, timing: await measuredProbes() });
    }
    await configureSkeletal();
    const benchmarkRadii = [];
    for (const radius of [0, 1, 2]) {
        const workload = loadConfig(64, 64, radius);
        await publish(workload);
        benchmarkRadii.push({ radius, timing: await measuredProbes() });
    }
    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        hashes: { meshlet: MESH_SHA256, animation: ANIMATION_SHA256, surface: SURFACE_SHA256 },
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        catalog: surfaceStatus.catalog,
        resources: surfaceStatus.resources,
        submission: surfaceStatus.submission,
        cooked,
        processes: { first: firstProcess, restarted: restartedProcess },
        initial: { probe: initialProbe, evidence: initialEvidence },
        sweeps: { materials, materialColors, mips, mipColors, lods, uniqueProbe, radii },
        deterministicTime: { time11: time11Evidence, returnedTime0: returnedEvidence },
        movement: { adjacentTransaction, adjacentProbe, revisitTransaction, revisitEvidence },
        restart: { probe: restartProbe, evidence: restartEvidence },
        benchmark: {
            preheatMs: PREHEAT_MS,
            sampleCount: SAMPLE_COUNT,
            initial: benchmarkInitial,
            materials: benchmarkMaterials,
            mips: benchmarkMips,
            lods: benchmarkLods,
            poses: benchmarkPoses,
            radii: benchmarkRadii,
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
console.log(JSON.stringify({
    outcome: finalReport.outcome,
    report: "out/captures/0011-gpu-surface-resolve/acceptance.json",
}));
