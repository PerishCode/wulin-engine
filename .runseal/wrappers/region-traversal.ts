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
import { compositionTimings, sleep, validateLodCompositionProbe } from "../support/composition.ts";
import {
    configMatches,
    logicalEvidence,
    traversal,
    TRAVERSAL_REVISION,
    worldCenter,
} from "../support/traversal.ts";

const SAMPLE_COUNT = 32;
const output = "out/terrain/0018-camera-driven-region-traversal/terrain.wlt";
const compatibilityReport = "out/captures/0017-gpu-lod-terrain-composition/acceptance.json";
const reportPath = "out/captures/0018-camera-driven-region-traversal/acceptance.json";

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :region-traversal");
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

async function waitStatus(
    label: string,
    predicate: (status: Record<string, unknown>) => boolean,
): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        traversal(status);
        if (predicate(status)) return status;
        await sleep(10);
    }
    fail(`${label} timed out`);
}

async function waitPublished(
    x: number,
    z: number,
    minimumAutomaticPublications?: number,
): Promise<Record<string, unknown>> {
    return await waitStatus(`publication ${x},${z}`, (status) => {
        if (status.pending !== null || !configMatches(object(status, "published").config, x, z)) {
            return false;
        }
        return minimumAutomaticPublications === undefined ||
            field<number>(traversal(status), "automaticPublicationCount", "number") >=
                minimumAutomaticPublications;
    });
}

async function waitDesired(x: number, z: number): Promise<Record<string, unknown>> {
    return await waitStatus(
        `desired center ${x},${z}`,
        (status) => configMatches(traversal(status).desired, x, z),
    );
}

async function manualPublish(x = 64, z = 64): Promise<Record<string, unknown>> {
    const before = await event("composition.status");
    const previous = before.published
        ? field<number>(object(before, "published"), "publicationCount", "number")
        : 0;
    const config = loadConfig(x, z);
    await event("composition.schedule", config);
    await event("workbench.resume");
    return await waitStatus(
        `manual publication ${x},${z}`,
        (status) =>
            status.pending === null && configMatches(object(status, "published").config, x, z) &&
            field<number>(object(status, "published"), "publicationCount", "number") >=
                previous + 1,
    );
}

async function setPosition(x: number, z: number): Promise<void> {
    await event("camera.set_pose", {
        position: [x, 6, z],
        target: [x, 1, z - 3],
        vertical_fov_degrees: 60,
    });
}

async function setCenter(x: number, z: number): Promise<void> {
    await setPosition(worldCenter(x), worldCenter(z));
}

async function moveAndPublish(x: number, z: number): Promise<Record<string, unknown>> {
    const before = traversal(await event("composition.status"));
    const publication = field<number>(before, "automaticPublicationCount", "number") + 1;
    await setCenter(x, z);
    return await waitPublished(x, z, publication);
}

async function prepare(): Promise<void> {
    await event("terrain.open", { path: output });
    await event("composition.fixture", { fixture: "arbitrary-q8" });
    await manualPublish();
    await event("skeletal.configure", {
        animated_percent: 100,
        bone_count: 64,
        phase_count: 64,
        time_tick: 0,
        unique_poses: false,
        forced_lod: null,
    });
    await event("terrain.lod.configure", {
        near_patch_radius: 2,
        middle_patch_radius: 6,
        forced_lod: null,
    });
    await event("terrain.lod.enable");
    await event("composition.enable");
    await event("composition.order", { order: "terrain-first" });
    await setCenter(64, 64);
    await event("workbench.resume");
}

async function enableTraversal(): Promise<Record<string, unknown>> {
    await event("composition.traversal.enable");
    const status = await waitDesired(64, 64);
    const state = traversal(status);
    const basis = object(state, "basis");
    if (
        state.enabled !== true || basis.worldRegionSide !== 128 || basis.activeRadius !== 2
    ) fail("camera traversal did not retain the published world/radius basis");
    return status;
}

async function probe(requireVisible = true): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateLodCompositionProbe(value, requireVisible);
    return value;
}

async function capture(id: string): Promise<Record<string, unknown>> {
    const value = await event("perception.capture", {
        id,
        collection: "0018-camera-driven-region-traversal",
        samples: [{ x: 640, y: 360 }],
    });
    return captureEvidence(value);
}

async function boundaryAndClamp(): Promise<Record<string, unknown>> {
    const samples = [];
    for (
        const sample of [
            { name: "positive-x-before", position: [7.999, 0], expected: [64, 64] },
            { name: "positive-x-exact", position: [8, 0], expected: [65, 64] },
            { name: "origin", position: [0, 0], expected: [64, 64] },
            { name: "negative-x-exact", position: [-8, 0], expected: [64, 64] },
            { name: "negative-x-after", position: [-8.001, 0], expected: [63, 64] },
            { name: "origin-again", position: [0, 0], expected: [64, 64] },
            { name: "positive-z-exact", position: [0, 8], expected: [64, 65] },
            { name: "negative-z-after", position: [0, -8.001], expected: [64, 63] },
            { name: "origin-final", position: [0, 0], expected: [64, 64] },
        ] as const
    ) {
        const before = traversal(await event("composition.status"));
        const beforePublications = field<number>(before, "automaticPublicationCount", "number");
        await setPosition(sample.position[0], sample.position[1]);
        const desired = await waitDesired(sample.expected[0], sample.expected[1]);
        const expectedCurrent = configMatches(
            object(desired, "published").config,
            sample.expected[0],
            sample.expected[1],
        );
        const status = expectedCurrent
            ? desired
            : await waitPublished(sample.expected[0], sample.expected[1], beforePublications + 1);
        samples.push({ ...sample, status });
    }

    await setPosition(-10_000, -10_000);
    const low = await waitPublished(2, 2);
    await setPosition(10_000, 10_000);
    const high = await waitPublished(125, 125);
    await setCenter(64, 64);
    const restored = await waitPublished(64, 64);
    return { samples, clamp: { low, high, restored } };
}

async function corridor(): Promise<Record<string, unknown>[]> {
    const evidence = [];
    for (const x of [65, 66, 67, 68, 67, 66, 65, 64]) {
        evidence.push({ x, status: await moveAndPublish(x, 64) });
    }
    return evidence;
}

async function heldLatestWins(): Promise<Record<string, unknown>> {
    const beforeStatus = await event("composition.status");
    const before = traversal(beforeStatus);
    const schedules = field<number>(before, "automaticScheduleCount", "number");
    const publications = field<number>(before, "automaticPublicationCount", "number");
    await event("terrain.io_gate.arm");
    await setCenter(65, 64);
    const pending = await waitStatus("held pair", (status) => {
        const pair = status.pending;
        return pair !== null && configMatches(object(status, "pending").config, 65, 64) &&
            field<number>(traversal(status), "automaticScheduleCount", "number") === schedules + 1;
    });
    const queued = [];
    for (const x of [66, 67, 68]) {
        await setCenter(x, 64);
        const status = await waitDesired(x, 64);
        if (!configMatches(traversal(status).queued, x, 64)) {
            fail(`held traversal did not retain latest center ${x},64`);
        }
        queued.push(status);
    }
    const heldProbe = await probe(false);
    const pair = object(heldProbe, "pair");
    if (
        !configMatches(object(pair, "published").config, 64, 64) ||
        !configMatches(object(pair, "pending").config, 65, 64) ||
        !configMatches(traversal(pair).queued, 68, 64)
    ) fail("held probe did not use the complete old pair with the latest queued center");

    await event("terrain.io_gate.release");
    const settled = await waitPublished(68, 64, publications + 2);
    const finalTraversal = traversal(settled);
    if (
        finalTraversal.automaticScheduleCount !== schedules + 2 ||
        finalTraversal.automaticPublicationCount !== publications + 2 ||
        finalTraversal.queued !== null || finalTraversal.blocked !== null ||
        field<number>(finalTraversal, "coalescedReplacementCount", "number") < 2
    ) fail("latest-wins release did not produce exactly the held and final pairs");
    return { before: beforeStatus, pending, queued, heldProbe, settled };
}

async function failureDoesNotRetry(): Promise<Record<string, unknown>> {
    const current = await event("composition.status");
    if (!configMatches(object(current, "published").config, 64, 64)) {
        await moveAndPublish(64, 64);
    }
    const before = traversal(await event("composition.status"));
    const attempts = field<number>(before, "automaticAttemptCount", "number");
    const schedules = field<number>(before, "automaticScheduleCount", "number");
    const publications = field<number>(before, "automaticPublicationCount", "number");
    await setCenter(80, 80);
    const blocked = await waitStatus("missing terrain block", (status) => {
        const state = traversal(status);
        return status.pending === null && state.blocked !== null &&
            configMatches(object(state, "blocked").config, 80, 80);
    });
    await sleep(250);
    const stable = await event("composition.status");
    const state = traversal(stable);
    if (
        state.automaticAttemptCount !== attempts + 1 ||
        state.automaticScheduleCount !== schedules ||
        state.automaticPublicationCount !== publications ||
        !configMatches(object(stable, "published").config, 64, 64)
    ) {
        fail(
            `blocked desired center retried or changed the old publication: ${
                JSON.stringify({
                    expectedAttempts: attempts + 1,
                    expectedSchedules: schedules,
                    expectedPublications: publications,
                    traversal: state,
                    published: object(stable, "published"),
                })
            }`,
        );
    }
    await setCenter(64, 64);
    const recovered = await waitDesired(64, 64);
    if (traversal(recovered).blocked !== null) fail("desired change did not clear traversal block");
    return { before, blocked, stable, recovered };
}

async function disableAndCatchUp(): Promise<Record<string, unknown>> {
    const before = await event("composition.status");
    const scheduleCount = field<number>(traversal(before), "automaticScheduleCount", "number");
    await event("composition.traversal.disable");
    await setCenter(65, 64);
    await sleep(150);
    const disabled = await event("composition.status");
    if (
        traversal(disabled).enabled !== false || disabled.pending !== null ||
        !configMatches(object(disabled, "published").config, 64, 64) ||
        traversal(disabled).automaticScheduleCount !== scheduleCount
    ) fail("disabled traversal reacted to camera movement");
    await event("composition.traversal.enable");
    const caughtUp = await waitPublished(65, 64);
    if (traversal(caughtUp).automaticScheduleCount !== scheduleCount + 1) {
        fail("traversal re-enable did not schedule exactly one catch-up pair");
    }
    return { before, disabled, caughtUp };
}

async function measuredTraversal(): Promise<Record<string, unknown>> {
    const probes = [];
    const catchupMs = [];
    const publicationMs = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const x = index % 2 === 0 ? 65 : 64;
        const start = performance.now();
        const status = await moveAndPublish(x, 64);
        catchupMs.push(performance.now() - start);
        publicationMs.push(field<number>(object(status, "published"), "publicationMs", "number"));
        probes.push(await probe());
    }
    return {
        sampleCount: SAMPLE_COUNT,
        catchupMs: distribution(catchupMs),
        publicationMs: distribution(publicationMs),
        gpu: compositionTimings(probes),
        finalStatus: await event("composition.status"),
    };
}

async function runCompatibility(): Promise<Record<string, unknown>> {
    await run("runseal", [":lod-composition"], "Experiment 0017 compatibility workflow");
    const report = JSON.parse(await Deno.readTextFile(`${root}/${compatibilityReport}`));
    if (report.outcome !== "pass") fail("Experiment 0017 compatibility report did not pass");
    return { report: compatibilityReport, revision: report.revision };
}

await lifecycle("stop");
sidecarConfig = "sidecar.benchmark.toml";
await lifecycle("stop");
sidecarConfig = "sidecar.toml";
const compatibility = await runCompatibility();
const centers = [
    [63, 64],
    [64, 64],
    [65, 64],
    [66, 64],
    [67, 64],
    [68, 64],
    [64, 63],
    [64, 65],
    [2, 2],
    [125, 125],
    [96, 96],
];
const cookArgs = ["run", "--locked", "--release", "-p", "terrain-cooker", "--", output];
for (const [x, z] of centers) cookArgs.push("--center", `${x}`, `${z}`);
await run("cargo", cookArgs, "traversal terrain cooker");
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;
try {
    await lifecycle("start");
    await prepare();
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const renderer = object(firstStatus, "renderer");
    if (renderer.debugLayer !== true || renderer.deviceRemovedReason !== null) {
        fail("debug traversal capability gate failed");
    }
    const initial = await enableTraversal();
    const baselineProbe = await probe();
    const baselineCapture = await capture("baseline");
    const boundaries = await boundaryAndClamp();
    const walked = await corridor();
    const held = await heldLatestWins();

    const teleport = await moveAndPublish(96, 96);
    const teleportProbe = await probe();
    const teleportCapture = await capture("teleport");
    const revisit = await moveAndPublish(64, 64);
    const revisitProbe = await probe();
    const revisitCapture = await capture("revisit");
    same(logicalEvidence(revisitProbe), logicalEvidence(baselineProbe), "logical revisit probe");
    same(revisitCapture, baselineCapture, "logical revisit attachments");

    const failure = await failureDoesNotRetry();
    const disabled = await disableAndCatchUp();

    await lifecycle("restart");
    await prepare();
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("traversal process survived restart");
    const restartInitial = await enableTraversal();
    const restartProbe = await probe();
    const restartCapture = await capture("restart");
    same(logicalEvidence(restartProbe), logicalEvidence(baselineProbe), "restart probe");
    same(restartCapture, baselineCapture, "restart attachments");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await prepare();
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release traversal capability gate failed");
    }
    await enableTraversal();
    const benchmark = await measuredTraversal();
    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: TRAVERSAL_REVISION,
        environment,
        compatibility,
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        initial,
        baseline: { probe: baselineProbe, capture: baselineCapture },
        boundaries,
        walked,
        held,
        teleport: { status: teleport, probe: teleportProbe, capture: teleportCapture },
        revisit: { status: revisit, probe: revisitProbe, capture: revisitCapture },
        failure,
        disabled,
        restart: { status: restartInitial, probe: restartProbe, capture: restartCapture },
        benchmark,
    };
} finally {
    await lifecycle("stop");
    sidecarConfig = "sidecar.toml";
    await lifecycle("stop");
}

if (!finalReport) fail("camera traversal experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/0018-camera-driven-region-traversal`, {
    recursive: true,
});
await Deno.writeTextFile(`${root}/${reportPath}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: "pass", report: reportPath }, null, 2));
