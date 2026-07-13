import {
    capture,
    expectCounts,
    globalConfig,
    globalProbe,
    hold,
    localProbe,
    prepare,
    probe,
    publishPair,
    waitPair,
} from "../support/global-composition.ts";
import { validateLodCompositionProbe } from "../support/composition.ts";
import {
    assertStopped,
    cook,
    event,
    failedEvent,
    failedRawEvent,
    lifecycle,
    root,
    run,
    transactionDistributions,
    useSidecar,
} from "../support/global-terrain.ts";
import {
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    object,
    same,
} from "../support/terrain.ts";

const REVISION = "signed-atomic-composition-v1";
const COLLECTION = "0021-signed-atomic-composition";
const OUTPUT = `out/terrain/${COLLECTION}/terrain.wlt`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :global-composition");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

function capability(status: Record<string, unknown>, debug: boolean) {
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== debug || renderer.deviceRemovedReason !== null) {
        fail(`${debug ? "debug" : "release"} composition capability gate failed`);
    }
    return renderer;
}

async function startClean(config: string): Promise<void> {
    useSidecar(config);
    await lifecycle("stop");
    await lifecycle("start");
}

async function publishLocal(): Promise<Record<string, unknown>> {
    const started = performance.now();
    const scheduled = await event("composition.schedule", loadConfig());
    const token = field<number>(scheduled, "token", "number");
    await event("workbench.resume");
    const status = await waitPair(token);
    await event("workbench.pause");
    return {
        scheduled,
        published: object(status, "published"),
        operatorPublicationMs: performance.now() - started,
    };
}

async function configureComposition(): Promise<void> {
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
    await event("camera.reset");
    await event("workbench.pause");
}

async function prepareLocal(): Promise<Record<string, unknown>> {
    await event("terrain.open", { path: OUTPUT });
    await event("composition.fixture", { fixture: "arbitrary-q8" });
    const publication = await publishLocal();
    await configureComposition();
    return publication;
}

async function probeLocal(): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateLodCompositionProbe(value);
    return value;
}

function ensureBytes(
    publication: Record<string, unknown>,
    terrainBytes: number,
    instanceBytes: number,
): void {
    const halves = object(publication, "halves");
    if (
        object(halves, "terrain").payloadBytes !== terrainBytes ||
        object(halves, "instance").instanceBytes !== instanceBytes
    ) fail("composition half payload bytes mismatch");
}

function ensureBounded(publication: Record<string, unknown>): void {
    const halves = object(publication, "halves");
    for (const name of ["terrain", "instance"]) {
        const report = object(halves, name);
        if (
            field<number>(report, "residentRegionCount", "number") > 50 ||
            field<number>(report, "protectedRegionCount", "number") > 25
        ) fail(`${name} cache exceeded bounded ownership`);
    }
}

function instanceDistributions(samples: Record<string, unknown>[]) {
    const values = (name: string) => samples.map((sample) => field<number>(sample, name, "number"));
    return {
        sampleCount: samples.length,
        payloadSources: [...new Set(samples.map((sample) => sample.payloadSource))],
        instanceBytes: [...new Set(values("instanceBytes"))],
        payloadPreparationMs: distribution(
            values("payloadPreparationMs"),
            "object payload preparation",
            true,
        ),
        generationMs: distribution(values("generationMs"), "object generation", true),
        scheduleMs: distribution(values("scheduleMs"), "object schedule", true),
        pendingMs: distribution(values("pendingMs"), "object pending", true),
    };
}

async function rejectionState(): Promise<Record<string, unknown>> {
    const pair = await event("composition.status");
    const terrain = await event("terrain.status");
    const terrainRenderer = object(terrain, "renderer");
    const terrainTransfer = object(terrainRenderer, "transfer");
    const instance = await event("async.status");
    return {
        pair: {
            nextToken: pair.nextToken,
            pending: pair.pending,
            published: pair.published,
        },
        terrain: {
            streamPending: object(terrain, "stream").pending,
            reservation: terrainTransfer.reservation,
            copyPending: terrainTransfer.copyPending,
            published: terrainRenderer.published,
        },
        instance: {
            reservation: instance.reservation,
            pending: instance.pending,
            published: instance.published,
            lastCompleted: instance.lastCompleted,
            nextCopyFence: object(instance, "copy").nextFence,
        },
    };
}

async function verifyRejected(
    beforeState: Record<string, unknown>,
    beforeProbe: Record<string, unknown>,
    beforeCapture: Record<string, unknown>,
    label: string,
): Promise<void> {
    same(await rejectionState(), beforeState, `${label} transaction state`);
    const current = await probeFromPublished();
    same(globalProbe(current), globalProbe(beforeProbe), `${label} global pair`);
    same(localProbe(current), localProbe(beforeProbe), `${label} local pair`);
    same(await capture(`${label}-held`, COLLECTION), beforeCapture, `${label} attachments`);
}

async function probeFromPublished(): Promise<Record<string, unknown>> {
    const status = await event("composition.status");
    const pair = object(status, "published");
    const config = object(pair, "globalConfig");
    return await probe({
        origin_x: field<number>(object(config, "globalOrigin"), "x", "number"),
        origin_z: field<number>(object(config, "globalOrigin"), "z", "number"),
        center_x: field<number>(object(config, "globalCenter"), "x", "number"),
        center_z: field<number>(object(config, "globalCenter"), "z", "number"),
        active_radius: field<number>(config, "activeRadius", "number"),
    });
}

await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":global-terrain"], "Experiment 0020 compatibility workflow");
const compatibility = {
    replayed: true,
    globalTerrain: "out/captures/0020-signed-terrain-addressing/acceptance.json",
    traversal: "out/captures/0018-camera-driven-region-traversal/acceptance.json",
    localComposition: "out/captures/0017-gpu-lod-terrain-composition/acceptance.json",
};
const centers: [number, number][] = [];
for (let x = 63; x <= 96; x += 1) centers.push([x, 64]);
centers.push([62, 64], [62, 63], [63, 65], [64, 65], [65, 65]);
const cooked = await cook(OUTPUT, centers);
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const correctnessRenderer = capability(firstStatus, true);

    const legacyPublication = await prepareLocal();
    const legacyProbe = await probeLocal();
    const legacyCapture = await capture("legacy", COLLECTION);

    const zeroConfig = globalConfig([0, 0]);
    const zeroPublication = await publishPair(zeroConfig);
    expectCounts(zeroPublication, {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        residentRegionCount: 50,
    });
    ensureBytes(zeroPublication, 102_400, 512_000);
    const zeroProbe = await probe(zeroConfig);
    const zeroCapture = await capture("anchor-zero", COLLECTION);
    same(localProbe(zeroProbe), localProbe(legacyProbe), "legacy/global local GPU output");
    same(zeroCapture, legacyCapture, "legacy/global attachments");

    const anchors: [number, number][] = [[FAR, -FAR], [-FAR, FAR]];
    const anchorEvidence: Record<string, unknown>[] = [];
    const hashes = new Set([
        field<string>(globalProbe(zeroProbe), "mappingSha256", "string"),
    ]);
    let farBaselineProbe: Record<string, unknown> | undefined;
    let farBaselineCapture: Record<string, unknown> | undefined;
    for (const [index, anchor] of anchors.entries()) {
        const config = globalConfig(anchor);
        const publication = await publishPair(config);
        expectCounts(publication, {
            retainedRegionCount: 0,
            uploadedRegionCount: 25,
            evictedRegionCount: 25,
            residentRegionCount: 50,
        });
        ensureBytes(publication, 102_400, 512_000);
        const value = await probe(config);
        const attachments = await capture(`anchor-${index}`, COLLECTION);
        same(localProbe(value), localProbe(zeroProbe), `anchor ${anchor} local output`);
        same(attachments, zeroCapture, `anchor ${anchor} attachments`);
        hashes.add(field<string>(globalProbe(value), "mappingSha256", "string"));
        anchorEvidence.push({ anchor, publication, probe: globalProbe(value), attachments });
        farBaselineProbe = value;
        farBaselineCapture = attachments;
    }
    if (hashes.size !== 3 || !farBaselineProbe || !farBaselineCapture) {
        fail("global pair hashes are not distinct");
    }

    const farOrigin: [number, number] = [-FAR, FAR];
    const baseConfig = globalConfig(farOrigin);
    const xConfig = globalConfig(farOrigin, [-FAR + 1, FAR]);
    const xMove = await publishPair(xConfig);
    expectCounts(xMove, { retainedRegionCount: 20, uploadedRegionCount: 5 });
    ensureBytes(xMove, 20_480, 102_400);
    const xRevisit = await publishPair(baseConfig);
    expectCounts(xRevisit, { retainedRegionCount: 25, uploadedRegionCount: 0 });
    ensureBytes(xRevisit, 0, 0);
    await publishPair(xConfig);
    const xzConfig = globalConfig(farOrigin, [-FAR + 1, FAR + 1]);
    const zMove = await publishPair(xzConfig);
    expectCounts(zMove, { retainedRegionCount: 20, uploadedRegionCount: 5 });
    ensureBytes(zMove, 20_480, 102_400);
    const unionRevisit = await publishPair(baseConfig);
    expectCounts(unionRevisit, { retainedRegionCount: 25, uploadedRegionCount: 0 });
    ensureBytes(unionRevisit, 0, 0);
    same(globalProbe(await probe(baseConfig)), globalProbe(farBaselineProbe), "far revisit");
    same(await capture("far-revisit", COLLECTION), farBaselineCapture, "far revisit attachments");

    const ioHold = await hold(
        "terrain-io",
        globalConfig(farOrigin, [-FAR - 1, FAR]),
        COLLECTION,
    );
    const terrainCopyHold = await hold(
        "terrain-copy",
        globalConfig(farOrigin, [-FAR - 2, FAR]),
        COLLECTION,
    );
    const objectCopyHold = await hold(
        "object-copy",
        globalConfig(farOrigin, [-FAR - 2, FAR - 1]),
        COLLECTION,
    );
    for (const held of [ioHold, terrainCopyHold, objectCopyHold]) {
        expectCounts(held, { retainedRegionCount: 20, uploadedRegionCount: 5 });
        ensureBytes(held, 20_480, 102_400);
    }

    const beforeRejectState = await rejectionState();
    const beforeRejectProbe = await probeFromPublished();
    const beforeRejectCapture = await capture("reject-before", COLLECTION);
    const missing = await failedEvent(
        "composition.global.schedule",
        globalConfig(farOrigin, [-FAR, FAR + 32]),
        "stream_failed",
    );
    await verifyRejected(
        beforeRejectState,
        beforeRejectProbe,
        beforeRejectCapture,
        "missing",
    );
    const outside = await failedEvent(
        "composition.global.schedule",
        globalConfig([0, 0], [64, 0]),
        "invalid_global_composition_config",
    );
    await verifyRejected(beforeRejectState, beforeRejectProbe, beforeRejectCapture, "range");
    const overflow = await failedRawEvent(
        "composition.global.schedule",
        '{"origin_x":-9223372036854775808,"origin_z":0,"center_x":9223372036854775807,"center_z":0,"active_radius":2}',
        "invalid_global_composition_config",
    );
    await verifyRejected(beforeRejectState, beforeRejectProbe, beforeRejectCapture, "overflow");

    const aliasOrigin: [number, number] = [2 ** 39, 2 ** 39];
    const aliasConfig = globalConfig(aliasOrigin, [aliasOrigin[0] - 2, aliasOrigin[1] - 1]);
    const aliasPublication = await publishPair(aliasConfig);
    expectCounts(aliasPublication, {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        evictedRegionCount: 25,
        residentRegionCount: 50,
    });
    ensureBytes(aliasPublication, 102_400, 512_000);
    const aliasProbe = await probe(aliasConfig);
    const aliasCapture = await capture("alias-rebind", COLLECTION);
    same(localProbe(aliasProbe), localProbe(beforeRejectProbe), "changed-origin local output");
    same(aliasCapture, beforeRejectCapture, "changed-origin attachments");

    await lifecycle("restart");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("workbench process survived restart");
    const restartPublication = await prepare(OUTPUT, baseConfig);
    const restartProbe = await probe(baseConfig);
    const restartCapture = await capture("restart", COLLECTION);
    same(globalProbe(restartProbe), globalProbe(farBaselineProbe), "restart global pair");
    same(localProbe(restartProbe), localProbe(farBaselineProbe), "restart local output");
    same(restartCapture, farBaselineCapture, "restart attachments");

    await lifecycle("stop");
    await startClean("sidecar.benchmark.toml");
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = capability(benchmarkStatus, false);
    const benchmarkOrigin: [number, number] = [FAR, -FAR];
    await prepare(OUTPUT, globalConfig(benchmarkOrigin));
    const adjacentTerrain: Record<string, unknown>[] = [];
    const adjacentInstance: Record<string, unknown>[] = [];
    const revisitTerrain: Record<string, unknown>[] = [];
    const revisitInstance: Record<string, unknown>[] = [];
    const pairMs: number[] = [];
    const operatorMs: number[] = [];
    const combinedGpuMs: number[] = [];
    for (let index = 1; index <= 32; index += 1) {
        const previous = globalConfig(benchmarkOrigin, [FAR + index - 1, -FAR]);
        const next = globalConfig(benchmarkOrigin, [FAR + index, -FAR]);
        const moved = await publishPair(next);
        expectCounts(moved, { retainedRegionCount: 20, uploadedRegionCount: 5 });
        ensureBytes(moved, 20_480, 102_400);
        ensureBounded(moved);
        const movedHalves = object(moved, "halves");
        adjacentTerrain.push(object(movedHalves, "terrain"));
        adjacentInstance.push(object(movedHalves, "instance"));
        pairMs.push(field<number>(object(moved, "published"), "publicationMs", "number"));
        operatorMs.push(field<number>(moved, "operatorPublicationMs", "number"));
        combinedGpuMs.push(
            field<number>(object(await probe(next, false), "timing"), "combinedGpuMs", "number"),
        );

        const revisited = await publishPair(previous);
        expectCounts(revisited, { retainedRegionCount: 25, uploadedRegionCount: 0 });
        ensureBytes(revisited, 0, 0);
        ensureBounded(revisited);
        const revisitHalves = object(revisited, "halves");
        revisitTerrain.push(object(revisitHalves, "terrain"));
        revisitInstance.push(object(revisitHalves, "instance"));
        const restored = await publishPair(next);
        expectCounts(restored, { retainedRegionCount: 25, uploadedRegionCount: 0 });
        ensureBytes(restored, 0, 0);
    }
    const benchmarkRestored = await publishPair(globalConfig(benchmarkOrigin));
    ensureBounded(benchmarkRestored);
    await probe(globalConfig(benchmarkOrigin));
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
        legacy: { publication: legacyPublication, probe: localProbe(legacyProbe), legacyCapture },
        zero: { publication: zeroPublication, probe: globalProbe(zeroProbe), zeroCapture },
        anchors: anchorEvidence,
        movement: { xMove, xRevisit, zMove, unionRevisit },
        holds: { terrainIo: ioHold, terrainCopy: terrainCopyHold, objectCopy: objectCopyHold },
        rejection: { missing, outside, overflow },
        aliasRebind: {
            publication: aliasPublication,
            probe: globalProbe(aliasProbe),
            aliasCapture,
        },
        restart: {
            processId: restartedProcess,
            publication: restartPublication,
            probe: globalProbe(restartProbe),
            restartCapture,
        },
        benchmark: {
            adjacent: {
                terrain: transactionDistributions(adjacentTerrain),
                instance: instanceDistributions(adjacentInstance),
            },
            revisit: {
                terrain: transactionDistributions(revisitTerrain),
                instance: instanceDistributions(revisitInstance),
            },
            pairPublicationMs: distribution(pairMs),
            operatorPublicationMs: distribution(operatorMs),
            combinedGpuMs: distribution(combinedGpuMs),
            restored: benchmarkRestored,
            benchmarkCapture,
        },
    };
} finally {
    await lifecycle("stop");
    useSidecar("sidecar.toml");
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("global composition experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: REPORT }));
