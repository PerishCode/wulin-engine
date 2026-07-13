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
    stableCompositionProbe,
    validateSamplingProbe,
} from "../support/composition.ts";

const SAMPLE_COUNT = 32;
const WARMUP_MS = 250;
const CELL_CENTER_HASH = "7e6779f8a69768b2c883aa339865c823d00dcaed63e3d6fa588e823a1e0e162c";
const ARBITRARY_GROUND_HASH = "c1f45c0af1eb28c2b02342e0feab3ff76e0ff54fb2b66fdbb53430a9c0a791db";
const ARBITRARY_POSITION_HASH = "509b4ffb49cdbdd29b40d9be2baf3b8c8030508060fcadc43932eb497eb03658";
const output = "out/terrain/0016-gpu-arbitrary-terrain-sampling/terrain.wlt";
const compatibilityReport = "out/captures/0015-atomic-terrain-object-composition/acceptance.json";
const reportPath = "out/captures/0016-gpu-arbitrary-terrain-sampling/acceptance.json";

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :terrain-sampling");
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
    fail("composition publication timed out");
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

async function prepare(workload = loadConfig()): Promise<void> {
    await event("terrain.open", { path: output });
    await event("composition.fixture", { fixture: "arbitrary-q8" });
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

function validateCanonical(probe: Record<string, unknown>): void {
    validateSamplingProbe(probe);
    const grounding = object(probe, "grounding");
    if (
        grounding.gpuSha256 !== ARBITRARY_GROUND_HASH ||
        grounding.cpuSha256 !== ARBITRARY_GROUND_HASH ||
        grounding.positionSha256 !== ARBITRARY_POSITION_HASH ||
        grounding.minimumNumerator !== -123_552 || grounding.maximumNumerator !== 164_608
    ) fail("canonical arbitrary sampling evidence changed");
    const published = object(object(probe, "pair"), "published");
    if (published.physicalSlotDivergenceCount !== 25) {
        fail("canonical pair did not diverge all physical cache mappings");
    }
}

async function probe(canonical = false): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    if (canonical) validateCanonical(value);
    else validateSamplingProbe(value);
    return value;
}

async function capture(id: string, requireBoth = true): Promise<Record<string, unknown>> {
    const value = await event("perception.capture", {
        id,
        collection: "0016-gpu-arbitrary-terrain-sampling",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const evidence = captureEvidence(value);
    const visible = object(object(object(value, "perception"), "evidence"), "fullFrame").objects;
    if (!Array.isArray(visible)) fail("sampling capture omitted semantic objects");
    const kinds = new Set(visible.map((entry) => object({ entry }, "entry").kind));
    if (requireBoth && (!kinds.has("terrain-region") || !kinds.has("region-proxy"))) {
        fail("sampling frame does not contain both semantic classes");
    }
    return evidence;
}

function cameraFor(centerX: number, centerZ: number): Record<string, unknown> {
    const x = (centerX - 64) * 16;
    const z = (centerZ - 64) * 16;
    return { position: [x + 9, 6, z + 12], target: [x, 1, z - 3], vertical_fov_degrees: 60 };
}

async function cameraSweep(): Promise<Record<string, unknown>[]> {
    const cameras = [
        { name: "default", position: [9, 6, 12], target: [0, 1, -3] },
        { name: "boundary", position: [44, 4, 0], target: [38, 0, 0] },
        { name: "corner", position: [44, 6, 44], target: [38, 0, 38] },
        { name: "high", position: [0, 36, 0], target: [0, 0, -1] },
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

async function measured(order: "terrain-first" | "object-first") {
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
        samples.push(await probe(true));
    }
    const stable = stableCompositionProbe(samples[0]);
    for (const sample of samples.slice(1)) {
        same(stableCompositionProbe(sample), stable, `${order} sampling evidence`);
    }
    return {
        order,
        warmupMs: WARMUP_MS,
        evidence: stable,
        timing: compositionTimings(samples),
        publicationMs: distribution(publicationMs),
    };
}

async function runCompatibility(): Promise<Record<string, unknown>> {
    await run("runseal", [":composition"], "Experiment 0015 compatibility workflow");
    const report = JSON.parse(await Deno.readTextFile(`${root}/${compatibilityReport}`));
    const canonical = object(object(report, "canonical"), "probe");
    const grounding = object(canonical, "grounding");
    if (
        grounding.fixture !== "cell-center" || grounding.groundDenominator !== 512 ||
        grounding.gpuSha256 !== CELL_CENTER_HASH || grounding.cpuSha256 !== CELL_CENTER_HASH
    ) fail("Experiment 0015 cell-center compatibility evidence changed");
    same(
        object(object(report, "canonical"), "terrainFirst"),
        object(object(report, "canonical"), "objectFirst"),
        "Experiment 0015 pass-order attachments",
    );
    return {
        report: compatibilityReport,
        grounding: stableCompositionProbe(canonical).grounding,
        benchmark: object(report, "benchmark"),
    };
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
        fail("debug sampling capability gate failed");
    }
    const canonicalProbe = await probe(true);
    const terrainFirst = await capture("canonical-terrain-first");
    await event("composition.order", { order: "object-first" });
    const objectFirst = await capture("canonical-object-first");
    same(objectFirst, terrainFirst, "arbitrary sampling pass-order attachments");
    await event("composition.order", { order: "terrain-first" });
    const cameras = await cameraSweep();

    const movement = [];
    for (const [x, z] of [[65, 64], [65, 65], [64, 64], [96, 96], [64, 64]]) {
        await event("camera.set_pose", cameraFor(x, z));
        const transaction = await publish(loadConfig(x, z));
        movement.push({
            x,
            z,
            transaction,
            probe: stableCompositionProbe(await probe(x === 64 && z === 64)),
        });
    }
    await event("camera.reset");
    same(
        object(movement[movement.length - 1], "probe"),
        stableCompositionProbe(canonicalProbe),
        "arbitrary sampling revisit",
    );

    await lifecycle("restart");
    await prepare();
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("sampling process survived restart");
    const restartProbe = await probe(true);
    const restartCapture = await capture("restart");
    same(
        stableCompositionProbe(restartProbe),
        stableCompositionProbe(canonicalProbe),
        "restart probe",
    );
    same(restartCapture, terrainFirst, "restart attachments");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await prepare();
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release sampling capability gate failed");
    }
    const benchmarkTerrainFirst = await measured("terrain-first");
    const benchmarkObjectFirst = await measured("object-first");
    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility,
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        canonical: { probe: canonicalProbe, terrainFirst, objectFirst },
        cameras,
        movement,
        restart: { probe: restartProbe, capture: restartCapture },
        benchmark: { terrainFirst: benchmarkTerrainFirst, objectFirst: benchmarkObjectFirst },
    };
} finally {
    await lifecycle("stop");
    sidecarConfig = "sidecar.toml";
    await lifecycle("stop");
}

if (!finalReport) fail("terrain sampling experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/0016-gpu-arbitrary-terrain-sampling`, { recursive: true });
await Deno.writeTextFile(`${root}/${reportPath}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: "pass", report: reportPath }, null, 2));
