import {
    capture,
    expectCounts,
    globalProbe,
    localProbe,
    prepare,
    probe,
    publishPair,
    waitPair,
} from "../support/global-composition.ts";
import {
    type Coord,
    enable,
    moveAndPublish,
    publication,
    setCenter,
    setPosition,
    target,
    targetMatches,
    traversal,
    waitDesired,
    waitPublished,
    waitStatus,
} from "../support/global-traversal.ts";
import {
    assertStopped,
    cook,
    event,
    failedEvent,
    lifecycle,
    rawEvent,
    root,
    run,
    sleep,
    transactionDistributions,
    useSidecar,
} from "../support/global-terrain.ts";
import {
    captureEvidence,
    collectEnvironment,
    distribution,
    fail,
    field,
    object,
    same,
} from "../support/terrain.ts";

const REVISION = "signed-camera-traversal-v1";
const COLLECTION = "0022-signed-camera-traversal";
const OUTPUT = `out/terrain/${COLLECTION}/terrain.wlt`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :global-traversal");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

function capability(status: Record<string, unknown>, debug: boolean) {
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== debug || renderer.deviceRemovedReason !== null) {
        fail(`${debug ? "debug" : "release"} traversal capability gate failed`);
    }
    return renderer;
}

async function startClean(config: string): Promise<void> {
    useSidecar(config);
    await lifecycle("stop");
    await lifecycle("start");
}

async function prepareSession(origin: Coord): Promise<Record<string, unknown>> {
    const initial = await prepare(OUTPUT, target(origin, 64, 64));
    await setCenter(64, 64);
    const enabled = await enable(origin);
    return { initial, enabled };
}

function ensureBytes(
    value: Record<string, unknown>,
    terrainBytes: number,
    instanceBytes: number,
): void {
    const halves = object(value, "halves");
    if (
        object(halves, "terrain").payloadBytes !== terrainBytes ||
        object(halves, "instance").instanceBytes !== instanceBytes
    ) fail("traversal half payload bytes mismatch");
}

function ensureBounded(value: Record<string, unknown>): void {
    const halves = object(value, "halves");
    for (const name of ["terrain", "instance"]) {
        const report = object(halves, name);
        if (
            field<number>(report, "residentRegionCount", "number") > 50 ||
            field<number>(report, "protectedRegionCount", "number") > 25
        ) fail(`${name} traversal cache exceeded bounded ownership`);
    }
}

async function captureAny(id: string): Promise<Record<string, unknown>> {
    const value = await event("perception.capture", {
        id,
        collection: COLLECTION,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    return {
        ...captureEvidence(value),
        png: field<string>(object(value, "image"), "pngSha256", "string"),
    };
}

function instanceDistributions(samples: Record<string, unknown>[]) {
    const values = (name: string) => samples.map((sample) => field<number>(sample, name, "number"));
    return {
        sampleCount: samples.length,
        instanceBytes: [...new Set(values("instanceBytes"))],
        generationMs: distribution(values("generationMs"), "object generation", true),
        scheduleMs: distribution(values("scheduleMs"), "object schedule", true),
        pendingMs: distribution(values("pendingMs"), "object pending", true),
    };
}

async function boundaries(origin: Coord): Promise<Record<string, unknown>> {
    const evidence = [];
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
        const before = traversal(await event("composition.status"), origin);
        const count = field<number>(before, "automaticPublicationCount", "number");
        await setPosition(sample.position[0], sample.position[1]);
        const desired = await waitDesired(origin, sample.expected[0], sample.expected[1]);
        const status = targetMatches(
                desired.published,
                sample.expected[0],
                sample.expected[1],
                origin,
            )
            ? desired
            : await waitPublished(
                origin,
                sample.expected[0],
                sample.expected[1],
                count + 1,
            );
        evidence.push({ ...sample, status });
    }
    await setPosition(-10_000, -10_000);
    const low = await waitPublished(origin, 2, 2);
    await setPosition(10_000, 10_000);
    const high = await waitPublished(origin, 125, 125);
    await setCenter(64, 64);
    const restored = await waitPublished(origin, 64, 64);
    return { evidence, clamp: { low, high, restored } };
}

async function corridor(origin: Coord): Promise<Record<string, unknown>[]> {
    const evidence = [];
    for (let x = 65; x <= 72; x += 1) {
        const status = await moveAndPublish(origin, x, 64);
        const value = await publication(status, target(origin, x, 64));
        expectCounts(value, { retainedRegionCount: 20, uploadedRegionCount: 5 });
        ensureBytes(value, 20_480, 102_400);
        evidence.push({ x, status, transaction: value });
    }
    await moveAndPublish(origin, 64, 64);
    return evidence;
}

async function heldLatestWins(origin: Coord): Promise<Record<string, unknown>> {
    const beforeStatus = await event("composition.status");
    const before = traversal(beforeStatus, origin);
    const schedules = field<number>(before, "automaticScheduleCount", "number");
    const publications = field<number>(before, "automaticPublicationCount", "number");
    await event("terrain.io_gate.arm");
    await setCenter(65, 64);
    const pending = await waitStatus(
        "held signed pair",
        (status) =>
            status.pending !== null && targetMatches(status.pending, 65, 64, origin) &&
            field<number>(traversal(status, origin), "automaticScheduleCount", "number") ===
                schedules + 1,
    );
    const queued = [];
    for (const x of [66, 67, 68]) {
        await setCenter(x, 64);
        const status = await waitDesired(origin, x, 64);
        if (!targetMatches(traversal(status, origin).queued, x, 64, origin)) {
            fail(`held signed traversal did not retain latest center ${x},64`);
        }
        queued.push(status);
    }
    const heldProbe = await probe(target(origin, 64, 64), false);
    const pair = object(heldProbe, "pair");
    if (
        !targetMatches(pair.published, 64, 64, origin) ||
        !targetMatches(pair.pending, 65, 64, origin) ||
        !targetMatches(traversal(pair, origin).queued, 68, 64, origin)
    ) fail("held probe did not retain one complete old signed pair");

    const released = await event("terrain.io_gate.release");
    const settled = await waitPublished(origin, 68, 64, publications + 2);
    const finalState = traversal(settled, origin);
    if (
        finalState.automaticScheduleCount !== schedules + 2 ||
        finalState.automaticPublicationCount !== publications + 2 ||
        finalState.queued !== null || finalState.blocked !== null ||
        field<number>(finalState, "coalescedReplacementCount", "number") < 2
    ) fail("signed latest-wins release did not publish only held and final targets");
    return { before: beforeStatus, pending, queued, heldProbe, released, settled };
}

async function failureAndRecovery(origin: Coord): Promise<Record<string, unknown>> {
    const before = traversal(await event("composition.status"), origin);
    const attempts = field<number>(before, "automaticAttemptCount", "number");
    const schedules = field<number>(before, "automaticScheduleCount", "number");
    const publications = field<number>(before, "automaticPublicationCount", "number");
    await setCenter(68, 96);
    const blocked = await waitStatus("signed missing terrain block", (status) => {
        const state = traversal(status, origin);
        return status.pending === null && state.blocked !== null &&
            targetMatches(state.blocked, 68, 96, origin);
    });
    const blockedProbe = await probe(target(origin, 68, 64), false);
    const blockedCapture = await captureAny("blocked");
    await sleep(250);
    const stable = await event("composition.status");
    const stableState = traversal(stable, origin);
    if (
        stableState.automaticAttemptCount !== attempts + 1 ||
        stableState.automaticScheduleCount !== schedules ||
        stableState.automaticPublicationCount !== publications ||
        !targetMatches(stable.published, 68, 64, origin)
    ) fail("blocked signed target retried or changed the old pair");
    same(await captureAny("blocked-stable"), blockedCapture, "blocked attachments");
    const recovered = await moveAndPublish(origin, 69, 64);
    if (traversal(recovered, origin).blocked !== null) {
        fail("different signed target did not clear traversal block");
    }
    return { before, blocked, blockedProbe, blockedCapture, stable, recovered };
}

async function disableCatchUp(origin: Coord): Promise<Record<string, unknown>> {
    const before = await event("composition.status");
    const schedules = field<number>(
        traversal(before, origin),
        "automaticScheduleCount",
        "number",
    );
    await event("composition.traversal.disable");
    await setCenter(70, 64);
    await sleep(150);
    const disabled = await event("composition.status");
    if (
        traversal(disabled).enabled !== false || disabled.pending !== null ||
        !targetMatches(disabled.published, 69, 64, origin) ||
        traversal(disabled).automaticScheduleCount !== schedules
    ) fail("disabled signed traversal reacted to camera movement");
    await enable(origin, 70, 64);
    const caughtUp = await waitPublished(origin, 70, 64);
    if (traversal(caughtUp, origin).automaticScheduleCount !== schedules + 1) {
        fail("signed traversal re-enable did not schedule one catch-up pair");
    }
    return { before, disabled, caughtUp };
}

async function overflowEnable(origin: Coord): Promise<Record<string, unknown>> {
    await event("composition.traversal.disable");
    const before = traversal(await event("composition.status"));
    const scheduled = await rawEvent(
        "composition.global.schedule",
        '{"origin_x":9223372036854775805,"origin_z":0,"center_x":9223372036854775805,"center_z":0,"active_radius":2}',
    );
    const token = field<number>(scheduled, "token", "number");
    await event("workbench.resume");
    await waitPair(token);
    const rejected = await failedEvent("composition.traversal.enable", {}, "stream_failed");
    const after = traversal(await event("composition.status"));
    same(after, before, "overflow traversal state");
    await publishPair(target(origin, 64, 64));
    await setCenter(64, 64);
    await enable(origin);
    return { scheduled: { token }, rejected, before, after };
}

await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":global-composition"], "Experiment 0021 compatibility workflow");
const compatibility = {
    globalComposition: "out/captures/0021-signed-atomic-composition/acceptance.json",
    localTraversal: "out/captures/0018-camera-driven-region-traversal/acceptance.json",
};
const centers: [number, number][] = [];
for (let x = 63; x <= 96; x += 1) centers.push([x, 64]);
centers.push([64, 63], [64, 65], [2, 2], [125, 125]);
const cooked = await cook(OUTPUT, centers);
const environment = await collectEnvironment(root);
const origin: Coord = [FAR, -FAR];
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const correctnessRenderer = capability(firstStatus, true);
    const session = await prepareSession(origin);
    const baselineProbe = await probe(target(origin, 64, 64));
    const baselineCapture = await capture("baseline", COLLECTION);
    const boundaryEvidence = await boundaries(origin);
    const corridorEvidence = await corridor(origin);
    const held = await heldLatestWins(origin);
    const failure = await failureAndRecovery(origin);
    const disabled = await disableCatchUp(origin);
    const overflow = await overflowEnable(origin);

    await lifecycle("restart");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("workbench process survived restart");
    const restartSession = await prepareSession(origin);
    const restartProbe = await probe(target(origin, 64, 64));
    const restartCapture = await capture("restart", COLLECTION);
    same(globalProbe(restartProbe), globalProbe(baselineProbe), "restart global pair");
    same(localProbe(restartProbe), localProbe(baselineProbe), "restart local output");
    same(restartCapture, baselineCapture, "restart attachments");

    await lifecycle("stop");
    await startClean("sidecar.benchmark.toml");
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = capability(benchmarkStatus, false);
    await prepareSession(origin);
    const terrainSamples: Record<string, unknown>[] = [];
    const instanceSamples: Record<string, unknown>[] = [];
    const pairMs: number[] = [];
    const operatorMs: number[] = [];
    const gpuMs: number[] = [];
    for (let x = 65; x <= 96; x += 1) {
        const started = performance.now();
        const status = await moveAndPublish(origin, x, 64);
        operatorMs.push(performance.now() - started);
        const value = await publication(status, target(origin, x, 64));
        expectCounts(value, { retainedRegionCount: 20, uploadedRegionCount: 5 });
        ensureBytes(value, 20_480, 102_400);
        ensureBounded(value);
        const halves = object(value, "halves");
        terrainSamples.push(object(halves, "terrain"));
        instanceSamples.push(object(halves, "instance"));
        pairMs.push(field<number>(object(value, "published"), "publicationMs", "number"));
        gpuMs.push(
            field<number>(
                object(await probe(target(origin, x, 64), false), "timing"),
                "combinedGpuMs",
                "number",
            ),
        );
    }
    await event("composition.traversal.disable");
    const restored = await publishPair(target(origin, 64, 64));
    await setCenter(64, 64);
    await probe(target(origin, 64, 64));
    const benchmarkCapture = await capture("benchmark-final", COLLECTION);

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility,
        cooked,
        capability: { correctness: correctnessRenderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        session,
        baseline: { probe: globalProbe(baselineProbe), capture: baselineCapture },
        boundaries: boundaryEvidence,
        corridor: corridorEvidence,
        held,
        failure,
        disabled,
        overflow,
        restart: {
            processId: restartedProcess,
            session: restartSession,
            probe: globalProbe(restartProbe),
            capture: restartCapture,
        },
        benchmark: {
            terrain: transactionDistributions(terrainSamples),
            instance: instanceDistributions(instanceSamples),
            pairPublicationMs: distribution(pairMs),
            operatorPublicationMs: distribution(operatorMs),
            combinedGpuMs: distribution(gpuMs),
            restored,
            capture: benchmarkCapture,
        },
    };
} finally {
    await lifecycle("stop");
    useSidecar("sidecar.toml");
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("signed camera traversal experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: REPORT }));
