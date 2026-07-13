import {
    captureEvidence,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    object,
    same,
} from "../support/terrain.ts";
import {
    compositionTimings,
    REVISION,
    sleep,
    stableLodCompositionProbe,
    validateLodCompositionProbe,
    validateSamplingProbe,
} from "../support/composition.ts";

const SAMPLE_COUNT = 32;
const WARMUP_MS = 250;
const GROUND_HASH = "c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db";
const POSITION_HASH = "509b4ffb49cdbdd29b40d9be2baf3b8c8030508060fcadc43932eb497eb03658";
const DISABLED_COLOR = "b345988cb1fcda9e8c6e09a50106a6a3efdf47391bb181ff2473d218890d7b72";
const DISABLED_OBJECT_ID = "b1f4a196de5f4d801d58395d3767e0b6edf8cbdfed75b0d5814eef558972f292";
const output = "out/terrain/0017-gpu-lod-terrain-composition/terrain.wlt";
const compatibilityReport = "out/captures/0016-gpu-arbitrary-terrain-sampling/acceptance.json";
const reportPath = "out/captures/0017-gpu-lod-terrain-composition/acceptance.json";

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :lod-composition");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);
const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
let sidecarConfig = "sidecar.toml";

async function run(command: string, args: string[], label: string): Promise<void> {
    const status = await new Deno.Command(command, {
        args,
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`${label} failed with exit code ${status.code}`);
}

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
    await run("sidecar", [verb, "--config", sidecarConfig], `sidecar ${verb}`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    return object(await sidecar(["inspect", "workbench", verb, JSON.stringify(payload)]), "data");
}

function configMatches(actual: Record<string, unknown>, expected: Record<string, number>): boolean {
    return actual.worldRegionSide === expected.world_region_side &&
        actual.activeCenterX === expected.active_center_x &&
        actual.activeCenterZ === expected.active_center_z &&
        actual.activeRadius === expected.active_radius;
}

async function waitPair(
    expected: Record<string, number>,
    minimumPublication: number,
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
    fail("LOD composition publication timed out");
}

async function publish(workload: Record<string, number>): Promise<Record<string, unknown>> {
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

async function configureLod(enabled: boolean, forcedLod: number | null): Promise<void> {
    await event("terrain.lod.configure", {
        near_patch_radius: 2,
        middle_patch_radius: 6,
        forced_lod: forcedLod,
    });
    await event(enabled ? "terrain.lod.enable" : "terrain.lod.disable");
}

async function prepare(): Promise<void> {
    await event("terrain.open", { path: output });
    await event("composition.fixture", { fixture: "arbitrary-q8" });
    await publish(loadConfig());
    await event("skeletal.configure", {
        animated_percent: 100,
        bone_count: 64,
        phase_count: 64,
        time_tick: 0,
        unique_poses: false,
        forced_lod: null,
    });
    await configureLod(false, null);
    await event("composition.enable");
    await event("composition.order", { order: "terrain-first" });
    await event("camera.reset");
    await event("workbench.pause");
}

function validateCanonical(probe: Record<string, unknown>): void {
    const grounding = object(probe, "grounding");
    if (grounding.gpuSha256 !== GROUND_HASH || grounding.positionSha256 !== POSITION_HASH) {
        fail("canonical exact grounding changed under terrain LOD");
    }
    if (object(object(probe, "pair"), "published").physicalSlotDivergenceCount !== 25) {
        fail("canonical LOD composition lost physical mapping divergence");
    }
}

async function probe(enabled: boolean, canonical = true): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    if (enabled) validateLodCompositionProbe(value);
    else validateSamplingProbe(value);
    if (canonical) validateCanonical(value);
    return value;
}

async function capture(id: string, requireBoth = true): Promise<Record<string, unknown>> {
    const value = await event("perception.capture", {
        id,
        collection: "0017-gpu-lod-terrain-composition",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const evidence = captureEvidence(value);
    const visible = object(object(object(value, "perception"), "evidence"), "fullFrame").objects;
    if (!Array.isArray(visible)) fail("LOD composition capture omitted semantic objects");
    const kinds = new Set(visible.map((entry) => object({ entry }, "entry").kind));
    if (requireBoth && (!kinds.has("terrain-region") || !kinds.has("region-proxy"))) {
        fail("LOD composition frame does not contain both semantic classes");
    }
    return evidence;
}

async function captureOrders(name: string): Promise<Record<string, unknown>> {
    await event("composition.order", { order: "terrain-first" });
    const terrainFirst = await capture(`${name}-terrain-first`);
    await event("composition.order", { order: "object-first" });
    const objectFirst = await capture(`${name}-object-first`);
    same(objectFirst, terrainFirst, `${name} pass-order attachments`);
    await event("composition.order", { order: "terrain-first" });
    return { terrainFirst, objectFirst };
}

function assertContact(
    probeValue: Record<string, unknown>,
    zeroCount: number,
    maximumAbsolute: number,
): void {
    const contact = object(probeValue, "contact");
    if (
        contact.zeroCount !== zeroCount || contact.maximumAbsoluteNumerator !== maximumAbsolute ||
        contact.exceedanceCount !== 0 || contact.firstExceedance !== null
    ) fail("LOD composition contact baseline changed");
}

function logicalEvidence(probeValue: Record<string, unknown>): Record<string, unknown> {
    const stable = stableLodCompositionProbe(probeValue);
    const terrain = object(stable, "terrain");
    const { activeMapping: _mapping, activeMappingSha256: _mappingHash, ...logicalTerrain } =
        terrain;
    return { ...stable, terrain: logicalTerrain };
}

function cameraFor(centerX: number, centerZ: number): Record<string, unknown> {
    const x = (centerX - 64) * 16;
    const z = (centerZ - 64) * 16;
    return { position: [x + 9, 6, z + 12], target: [x, 1, z - 3], vertical_fov_degrees: 60 };
}

async function cameraSweep(): Promise<Record<string, unknown>[]> {
    const cameras = [
        { name: "interior", position: [1, 10, 1], target: [0, 0, 0] },
        { name: "patch-edge", position: [4, 10, 1], target: [0, 0, 0] },
        { name: "region-edge", position: [8, 10, 1], target: [0, 0, 0] },
        { name: "corner", position: [4, 10, 4], target: [0, 0, 0] },
        { name: "high", position: [0, 36, 0], target: [0, 0, -1] },
        { name: "grazing", position: [20, 3, 20], target: [0, 0, 0] },
    ];
    const evidence = [];
    for (const camera of cameras) {
        await event("camera.set_pose", {
            position: camera.position,
            target: camera.target,
            vertical_fov_degrees: 60,
        });
        evidence.push({
            camera,
            probe: stableLodCompositionProbe(await probe(true)),
            capture: await capture(camera.name, false),
        });
    }
    await event("camera.reset");
    return evidence;
}

async function measured(
    name: string,
    enabled: boolean,
    forcedLod: number | null,
    order: "terrain-first" | "object-first",
): Promise<Record<string, unknown>> {
    await configureLod(enabled, forcedLod);
    await event("composition.order", { order });
    await event("workbench.resume");
    await sleep(WARMUP_MS);
    await event("workbench.pause");
    const samples = [];
    const publicationMs = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const transaction = await publish(loadConfig());
        publicationMs.push(
            field<number>(object(transaction, "published"), "publicationMs", "number"),
        );
        samples.push(await probe(enabled));
    }
    const stable = stableLodCompositionProbe(samples[0]);
    for (const sample of samples.slice(1)) {
        same(stableLodCompositionProbe(sample), stable, `${name} ${order} evidence`);
    }
    return {
        name,
        order,
        warmupMs: WARMUP_MS,
        evidence: stable,
        timing: compositionTimings(samples),
        publicationMs: distribution(publicationMs),
    };
}

async function runCompatibility(): Promise<Record<string, unknown>> {
    await run("runseal", [":terrain-sampling"], "Experiment 0016 compatibility workflow");
    const report = JSON.parse(await Deno.readTextFile(`${root}/${compatibilityReport}`));
    if (report.outcome !== "pass") fail("Experiment 0016 compatibility report did not pass");
    const canonical = object(object(report, "canonical"), "probe");
    validateSamplingProbe(canonical);
    validateCanonical(canonical);
    return { report: compatibilityReport, canonical: stableLodCompositionProbe(canonical) };
}

await lifecycle("stop");
sidecarConfig = "sidecar.benchmark.toml";
await lifecycle("stop");
sidecarConfig = "sidecar.toml";
const compatibility = await runCompatibility();
await run(
    "cargo",
    ["run", "--locked", "--release", "-p", "terrain-cooker", "--", output],
    "terrain cooker",
);
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;
try {
    await lifecycle("start");
    await prepare();
    const status = await event("workbench.status");
    const renderer = object(status, "renderer");
    const firstProcess = field<number>(status, "processId", "number");
    if (renderer.debugLayer !== true || renderer.deviceRemovedReason !== null) {
        fail("debug LOD composition capability gate failed");
    }

    const disabledProbe = await probe(false);
    assertContact(disabledProbe, 25_600, 0);
    const disabledAttachments = await captureOrders("disabled");
    const disabled = object(disabledAttachments, "terrainFirst");
    if (disabled.color !== DISABLED_COLOR || disabled.objectId !== DISABLED_OBJECT_ID) {
        fail("accepted disabled composition attachments changed");
    }

    await configureLod(true, null);
    const automaticProbe = await probe(true);
    same(
        object(object(object(automaticProbe, "terrain"), "lod"), "oracle").lodCounts,
        [25, 144, 231],
        "automatic terrain LOD counts",
    );
    assertContact(automaticProbe, 2_640, 23_488);
    const automaticAttachments = await captureOrders("automatic");

    const forced = [];
    for (
        const [level, zeroCount, maximumAbsolute] of [[0, 25_600, 0], [1, 2_470, 5_376], [
            2,
            363,
            23_488,
        ]]
    ) {
        await configureLod(true, level);
        const value = await probe(true);
        assertContact(value, zeroCount, maximumAbsolute);
        const attachments = await captureOrders(`forced-${level}`);
        if (level === 0) {
            same(object(attachments, "terrainFirst"), disabled, "forced LOD0 attachments");
        }
        forced.push({ level, probe: value, attachments });
    }

    await configureLod(true, null);
    const cameras = await cameraSweep();
    const movement = [];
    for (const [x, z] of [[65, 64], [65, 65], [96, 96], [64, 64]]) {
        await event("camera.set_pose", cameraFor(x, z));
        const transaction = await publish(loadConfig(x, z));
        movement.push({
            x,
            z,
            transaction,
            probe: stableLodCompositionProbe(await probe(true, x === 64 && z === 64)),
        });
    }
    await event("camera.reset");
    const revisitProbe = await probe(true);
    same(
        logicalEvidence(revisitProbe),
        logicalEvidence(automaticProbe),
        "automatic LOD composition revisit",
    );

    await lifecycle("restart");
    await prepare();
    await configureLod(true, null);
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("LOD composition process survived restart");
    const restartProbe = await probe(true);
    const restartCapture = await capture("restart");
    same(
        stableLodCompositionProbe(restartProbe),
        stableLodCompositionProbe(automaticProbe),
        "restart probe",
    );
    same(restartCapture, object(automaticAttachments, "terrainFirst"), "restart attachments");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await prepare();
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release LOD composition capability gate failed");
    }
    const benchmark: Record<string, unknown> = {};
    for (
        const [name, enabled, forcedLod] of [
            ["disabled", false, null],
            ["automatic", true, null],
            ["forced-lod1", true, 1],
            ["forced-lod2", true, 2],
        ] as const
    ) {
        benchmark[name] = {
            terrainFirst: await measured(name, enabled, forcedLod, "terrain-first"),
            objectFirst: await measured(name, enabled, forcedLod, "object-first"),
        };
    }
    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility,
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        disabled: { probe: disabledProbe, attachments: disabledAttachments },
        automatic: { probe: automaticProbe, attachments: automaticAttachments },
        forced,
        cameras,
        movement,
        revisit: revisitProbe,
        restart: { probe: restartProbe, capture: restartCapture },
        benchmark,
    };
} finally {
    await lifecycle("stop");
    sidecarConfig = "sidecar.toml";
    await lifecycle("stop");
}

if (!finalReport) fail("LOD composition experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/0017-gpu-lod-terrain-composition`, { recursive: true });
await Deno.writeTextFile(`${root}/${reportPath}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: "pass", report: reportPath }, null, 2));
