import {
    array,
    CANONICAL_MAPPING,
    CANONICAL_PAYLOAD,
    captureEvidence,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    object,
    same,
    stableLodProbe,
    stableProbe,
    TERRAIN_LOD_REVISION as LOD_REVISION,
    validateLodProbe,
    validateProbe,
} from "../support/terrain.ts";

const SAMPLE_COUNT = 32;
const WARMUP_MS = 250;
const output = "out/terrain/0014-gpu-terrain-lod-stitching/terrain.wlt";
const reportRelative = "out/captures/0014-gpu-terrain-lod-stitching/acceptance.json";

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
    return object(await sidecar(["inspect", "workbench", verb, JSON.stringify(payload)]), "data");
}

async function rejectedEvent(verb: string, payload: unknown): Promise<string> {
    const result = await new Deno.Command("sidecar", {
        args: [
            "inspect",
            "workbench",
            verb,
            JSON.stringify(payload),
            "--config",
            sidecarConfig,
            "--format",
            "json",
        ],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(result.stdout).trim();
    if (!stdout) fail(`rejected ${verb} returned no JSON`);
    const value = JSON.parse(stdout) as Record<string, unknown>;
    if (result.success || value.ok !== false) {
        fail(`${verb} unexpectedly accepted invalid settings`);
    }
    if (typeof value.error !== "string") fail(`${verb} rejection did not contain an error string`);
    return value.error;
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

async function waitPublished(
    expected: Record<string, number>,
    minimumGeneration = 0,
): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const renderer = object(status, "renderer");
        const published = renderer.published;
        if (published && typeof published === "object") {
            const value = published as Record<string, unknown>;
            const config = object(value, "config");
            if (
                object(status, "stream").pending === null &&
                object(renderer, "transfer").copyPending === null &&
                field<number>(value, "generation", "number") >= minimumGeneration &&
                config.worldRegionSide === expected.world_region_side &&
                config.activeCenterX === expected.active_center_x &&
                config.activeCenterZ === expected.active_center_z &&
                config.activeRadius === expected.active_radius
            ) return status;
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail("terrain LOD publication timed out");
}

async function publish(workload: Record<string, number>): Promise<Record<string, unknown>> {
    const scheduled = await event("terrain.schedule", workload);
    await event("workbench.resume");
    const status = await waitPublished(workload);
    await event("workbench.pause");
    return { scheduled, published: object(object(status, "renderer"), "published") };
}

async function prepare(): Promise<void> {
    await event("terrain.open", { path: output });
    await publish(loadConfig());
    await event("terrain.enable");
    await event("terrain.lod.disable");
    await event("camera.reset");
    await event("workbench.pause");
}

async function configure(
    nearPatchRadius: number,
    middlePatchRadius: number,
    forcedLod: number | null,
    enabled = true,
): Promise<void> {
    await event("terrain.lod.configure", {
        near_patch_radius: nearPatchRadius,
        middle_patch_radius: middlePatchRadius,
        forced_lod: forcedLod,
    });
    await event(enabled ? "terrain.lod.enable" : "terrain.lod.disable");
}

async function probe(enabled: boolean): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateLodProbe(value, enabled);
    return value;
}

async function capture(id: string): Promise<Record<string, unknown>> {
    return captureEvidence(
        await event("perception.capture", {
            id,
            collection: "0014-gpu-terrain-lod-stitching",
            samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
        }),
    );
}

function captureHashes(capture: Record<string, unknown>): Record<string, unknown> {
    return {
        color: capture.color,
        objectId: capture.objectId,
        diagnostic: capture.diagnostic,
    };
}

async function holdStage(
    kind: "io" | "copy",
    destination: Record<string, number>,
): Promise<Record<string, unknown>> {
    const before = stableLodProbe(await probe(true));
    await event(`terrain.${kind}_gate.arm`);
    await event("terrain.schedule", destination);
    await event("workbench.resume");
    const deadline = Date.now() + 5_000;
    let heldStatus: Record<string, unknown> | undefined;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const renderer = object(status, "renderer");
        const stream = object(status, "stream");
        const reached = kind === "io"
            ? stream.pending && object(stream, "pending").stage === "reading"
            : object(renderer, "transfer").copyPending !== null;
        if (reached) {
            heldStatus = status;
            break;
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    if (!heldStatus) fail(`${kind} hold did not reach the expected stage`);
    await event("workbench.pause");
    const held = stableLodProbe(await probe(true));
    same(held, before, `${kind} held LOD snapshot`);
    await event(`terrain.${kind}_gate.release`);
    await event("workbench.resume");
    const published = await waitPublished(destination);
    await event("workbench.pause");
    const after = stableLodProbe(await probe(true));
    return { before, held, heldStatus, published, after };
}

async function measuredWorkload(
    name: string,
    near: number,
    middle: number,
    forced: number | null,
    enabled: boolean,
): Promise<Record<string, unknown>> {
    await configure(near, middle, forced, enabled);
    await event("workbench.resume");
    await new Promise((resolve) => setTimeout(resolve, WARMUP_MS));
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) samples.push(await probe(enabled));
    const evidence = stableLodProbe(samples[0]);
    for (const sample of samples.slice(1)) {
        same(stableLodProbe(sample), evidence, `${name} LOD timing evidence`);
    }
    const timing = (fieldName: string) =>
        distribution(
            samples.map((sample) => field<number>(object(sample, "timing"), fieldName, "number")),
        );
    return {
        sampleCount: samples.length,
        warmupMs: WARMUP_MS,
        evidence,
        validationMs: timing("seamMs"),
        rasterMs: timing("rasterMs"),
        totalMs: timing("totalMs"),
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :terrain-lod");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);
const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const reportPath = `${root}/${reportRelative}`;
let sidecarConfig = "sidecar.toml";
let finalReport: Record<string, unknown> | undefined;

await lifecycle("stop");
const cooked = await cook();
const environment = await collectEnvironment(root);
try {
    await lifecycle("start");
    await prepare();
    const status = await event("workbench.status");
    const renderer = object(status, "renderer");
    const firstProcess = field<number>(status, "processId", "number");
    if (renderer.debugLayer !== true || renderer.deviceRemovedReason !== null) {
        fail("debug terrain LOD capability gate failed");
    }

    const disabledProbe = await probe(false);
    validateProbe(disabledProbe);
    if (
        disabledProbe.activeMappingSha256 !== CANONICAL_MAPPING ||
        disabledProbe.payloadSha256 !== CANONICAL_PAYLOAD
    ) fail("disabled terrain LOD mapping or payload hash mismatch");
    const disabledCapture = await capture("disabled");
    same(captureHashes(disabledCapture), {
        color: "1c044011973c4df5ffc2f0f92967f8595281ecb892073d991c0f9adaa6b7d1aa",
        objectId: "37e32108bf3aaba3b1e37e2488be5221c96a1a95b8359224e5b2dcb915c10aa8",
        diagnostic: "2dace648099e6eb234d6def4fc3c5274244c69eaf32c7fd6255a43eecd112127",
    }, "accepted Experiment 0013 attachments");
    const invalidSettings = [];
    for (
        const payload of [
            { near_patch_radius: 2, middle_patch_radius: 2, forced_lod: null },
            { near_patch_radius: 2, middle_patch_radius: 65, forced_lod: null },
            { near_patch_radius: 2, middle_patch_radius: 6, forced_lod: 3 },
        ]
    ) {
        const error = await rejectedEvent("terrain.lod.configure", payload);
        if (!error.includes("invalid_terrain_lod_config")) {
            fail("invalid terrain LOD settings returned the wrong protocol error");
        }
        invalidSettings.push({ payload, error });
    }

    await configure(2, 6, null);
    const automaticProbe = await probe(true);
    const automaticOracle = object(object(automaticProbe, "lod"), "oracle");
    same(array(automaticOracle, "lodCounts"), [25, 144, 231], "canonical automatic LOD counts");
    const automaticGeometry = object(automaticOracle, "geometry");
    if (
        field<number>(automaticGeometry, "vertexReductionPercent", "number") <= 0 ||
        field<number>(automaticGeometry, "triangleReductionPercent", "number") <= 0
    ) fail("automatic terrain LOD did not reduce work");
    const automaticCapture = await capture("automatic");

    const forced = [];
    for (const level of [0, 1, 2]) {
        await configure(2, 6, level);
        const value = await probe(true);
        const visual = await capture(`forced-${level}`);
        forced.push({ level, probe: value, capture: visual });
    }
    same(stableProbe(forced[0].probe), stableProbe(disabledProbe), "forced LOD 0 probe");
    same(
        captureHashes(forced[0].capture),
        captureHashes(disabledCapture),
        "forced LOD 0 attachments",
    );
    const expectedGeometry = [[32_400, 51_200], [10_000, 12_800], [3_600, 3_200]];
    for (const [index, entry] of forced.entries()) {
        const geometry = object(entry.probe, "geometry");
        if (
            geometry.vertices !== expectedGeometry[index][0] ||
            geometry.triangles !== expectedGeometry[index][1]
        ) fail(`forced LOD ${index} geometry mismatch`);
    }

    const bands = [];
    for (const [near, middle] of [[0, 2], [2, 6], [4, 8]]) {
        await configure(near, middle, null);
        bands.push({ near, middle, probe: await probe(true) });
    }
    const cameras = [];
    for (
        const camera of [
            { name: "interior", position: [1, 10, 1] },
            { name: "patch-edge", position: [4, 10, 1] },
            { name: "region-edge", position: [8, 10, 1] },
            { name: "four-patch-corner", position: [4, 10, 4] },
            { name: "grazing", position: [20, 3, 20] },
        ]
    ) {
        await event("camera.set_pose", {
            position: camera.position,
            target: [0, 0, 0],
            vertical_fov_degrees: 60,
        });
        cameras.push({ camera, probe: await probe(true), capture: await capture(camera.name) });
    }
    await event("camera.reset");
    await configure(2, 6, null);

    const ioHold = await holdStage("io", loadConfig(65, 64));
    const copyHold = await holdStage("copy", loadConfig(65, 65));
    const movement = [];
    for (const [x, z] of [[64, 64], [96, 96]]) {
        const transaction = await publish(loadConfig(x, z));
        movement.push({ x, z, transaction, probe: await probe(true) });
    }
    await publish(loadConfig());
    const revisitProbe = await probe(true);
    if (
        object(object(revisitProbe, "lod"), "oracle").lodSha256 !==
            automaticOracle.lodSha256
    ) fail("terrain LOD classification changed after physical slot movement");

    await lifecycle("restart");
    await prepare();
    await configure(2, 6, null);
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("terrain LOD process survived restart");
    const restartProbe = await probe(true);
    const restartCapture = await capture("restart");
    same(stableLodProbe(restartProbe), stableLodProbe(automaticProbe), "terrain LOD restart probe");
    same(restartCapture, automaticCapture, "terrain LOD restart attachments");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await prepare();
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release terrain LOD capability gate failed");
    }
    const benchmark = {
        disabled: await measuredWorkload("disabled", 2, 6, null, false),
        automatic: await measuredWorkload("automatic", 2, 6, null, true),
        forcedLod1: await measuredWorkload("forced LOD 1", 2, 6, 1, true),
        forcedLod2: await measuredWorkload("forced LOD 2", 2, 6, 2, true),
    };
    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: LOD_REVISION,
        environment,
        cooked,
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        baseline: { probe: disabledProbe, capture: disabledCapture },
        invalidSettings,
        automatic: { probe: automaticProbe, capture: automaticCapture },
        forced,
        bands,
        cameras,
        holds: { io: ioHold, copy: copyHold },
        movement,
        revisit: revisitProbe,
        restart: { probe: restartProbe, capture: restartCapture },
        benchmark,
    };
} finally {
    await lifecycle("stop");
}

for (const config of ["sidecar.toml", "sidecar.benchmark.toml"]) {
    sidecarConfig = config;
    const cleanup = await sidecar(["status"]);
    if (field<boolean>(object(cleanup, "runtime"), "running", "boolean")) {
        fail(`${config} broker remains running`);
    }
    for (const target of array(cleanup, "targets") as Record<string, unknown>[]) {
        if (field<boolean>(target, "running", "boolean")) fail(`${config} target remains running`);
    }
}
if (!finalReport) fail("no terrain LOD acceptance report was produced");
await Deno.mkdir(reportPath.replace(/[\\/][^\\/]+$/, ""), { recursive: true });
await Deno.writeTextFile(reportPath, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: reportRelative }));
