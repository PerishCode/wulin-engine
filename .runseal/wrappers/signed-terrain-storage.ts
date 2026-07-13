import {
    assertStopped,
    capture,
    event,
    expectCounts,
    failedEvent,
    globalConfig,
    globalProbe,
    hold,
    lifecycle,
    probe,
    publish,
    root,
    run,
    transaction,
    transactionDistributions,
    useSidecar,
} from "../support/global-terrain.ts";
import {
    canonicalEvidence,
    captureJoined,
    cookSigned,
    corruptSignedIndexOffset,
    corruptSignedPayload,
    globalContent,
    globalSlots,
    restoreByte,
    waitFailure,
} from "../support/signed-terrain-storage.ts";
import {
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    object,
    same,
} from "../support/terrain.ts";

const REVISION = "signed-terrain-storage-v1";
const COLLECTION = "0023-signed-terrain-storage";
const OUTPUT_A = `out/terrain/${COLLECTION}/terrain-a.wlt`;
const OUTPUT_A_REPEAT = `out/terrain/${COLLECTION}/terrain-a-repeat.wlt`;
const OUTPUT_B = `out/terrain/${COLLECTION}/terrain-b.wlt`;
const CORRUPT = `out/terrain/${COLLECTION}/terrain-corrupt.wlt`;
const BAD_INDEX = `out/terrain/${COLLECTION}/terrain-bad-index.wlt`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :signed-terrain-storage");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

function config(x: number, originX = FAR) {
    return globalConfig([originX, -FAR], [x, -FAR]);
}

function publishedTransaction(publication: Record<string, unknown>) {
    return transaction(publication);
}

function capability(status: Record<string, unknown>, debug: boolean) {
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== debug || renderer.deviceRemovedReason !== null) {
        fail(`${debug ? "debug" : "release"} signed storage capability gate failed`);
    }
    return renderer;
}

function sourceNamespace(cooked: Record<string, unknown>): string {
    return field<string>(object(cooked, "metadata"), "sourceNamespaceSha256", "string");
}

function ensureBounded(report: Record<string, unknown>): void {
    if (
        field<number>(report, "residentRegionCount", "number") > 50 ||
        field<number>(report, "protectedRegionCount", "number") > 25
    ) fail("signed terrain storage exceeded bounded cache ownership");
}

function ensureSource(report: Record<string, unknown>, expected: string): void {
    if (field<string>(report, "sourceNamespace", "string") !== expected) {
        fail("signed terrain transaction source namespace mismatch");
    }
}

async function setAliasCamera(localCenterX: number): Promise<void> {
    const offset = (localCenterX - 64) * 16;
    await event("camera.set_pose", {
        position: [9 + offset, 6, 12],
        target: [offset, 1, -3],
        vertical_fov_degrees: 60,
    });
}

async function startClean(sidecar: string): Promise<void> {
    useSidecar(sidecar);
    await lifecycle("stop");
    await lifecycle("start");
}

async function prepare(path: string, target = config(FAR)): Promise<Record<string, unknown>> {
    await event("terrain.open", { path });
    const publication = await publish("terrain.global.schedule", target);
    await event("terrain.enable");
    await event("camera.reset");
    await event("workbench.pause");
    return publication;
}

async function verifyStableFailure(
    beforeProbe: Record<string, unknown>,
    beforeCapture: Record<string, unknown>,
    transactionId: number,
    label: string,
): Promise<Record<string, unknown>> {
    const status = await event("terrain.status");
    const renderer = object(status, "renderer");
    if (
        object(status, "stream").pending !== null ||
        object(renderer, "transfer").reservation !== null ||
        object(renderer, "transfer").copyPending !== null ||
        object(object(renderer, "published"), "transaction").transactionId !== transactionId
    ) fail(`${label} mutated signed terrain state`);
    same(canonicalEvidence(await probe()), canonicalEvidence(beforeProbe), `${label} content`);
    same(await capture(`${label}-stable`, COLLECTION), beforeCapture, `${label} attachments`);
    return status;
}

async function aliasRebind(
    sourceNamespace: string,
    baseProbe: Record<string, unknown>,
): Promise<Record<string, unknown>> {
    const shifted = config(FAR, FAR - 32);
    const publication = await publish("terrain.global.schedule", shifted);
    const report = publishedTransaction(publication);
    expectCounts(report, {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        evictedRegionCount: 0,
        residentRegionCount: 25,
        payloadBytes: 0,
    });
    ensureSource(report, sourceNamespace);
    if (object(report, "io").payloadBytes !== 0) fail("alias rebind performed terrain I/O");
    await setAliasCamera(96);
    const shiftedProbe = await probe(shifted);
    same(canonicalEvidence(shiftedProbe), canonicalEvidence(baseProbe), "alias content");
    same(globalSlots(shiftedProbe), globalSlots(baseProbe), "alias global slots");
    if (
        shiftedProbe.payloadSha256 === baseProbe.payloadSha256 ||
        shiftedProbe.activeMappingSha256 === baseProbe.activeMappingSha256
    ) fail("alias rebind did not change local projection evidence");
    const shiftedCapture = await captureJoined("alias-96", COLLECTION, shiftedProbe);

    const revisits = [];
    for (let index = 0; index < 8; index += 1) {
        const localCenter = index % 2 === 0 ? 64 : 96;
        const target = localCenter === 64 ? config(FAR) : shifted;
        const value = await publish("terrain.global.schedule", target);
        const transaction = publishedTransaction(value);
        expectCounts(transaction, {
            retainedRegionCount: 25,
            uploadedRegionCount: 0,
            residentRegionCount: 25,
            payloadBytes: 0,
        });
        ensureSource(transaction, sourceNamespace);
        await setAliasCamera(localCenter);
        const valueProbe = await probe(target);
        same(canonicalEvidence(valueProbe), canonicalEvidence(baseProbe), "alias revisit content");
        same(globalSlots(valueProbe), globalSlots(baseProbe), "alias revisit slots");
        revisits.push({ localCenter, transaction, probe: globalProbe(valueProbe) });
    }
    await publish("terrain.global.schedule", config(FAR));
    await setAliasCamera(64);
    return { publication, report, probe: shiftedProbe, capture: shiftedCapture, revisits };
}

async function adjacentCorridor(sourceNamespace: string): Promise<Record<string, unknown>[]> {
    const evidence = [];
    for (let offset = 1; offset <= 8; offset += 1) {
        const target = config(FAR + offset);
        const publication = await publish("terrain.global.schedule", target);
        const report = publishedTransaction(publication);
        expectCounts(report, {
            retainedRegionCount: 20,
            uploadedRegionCount: 5,
            residentRegionCount: Math.min(50, 25 + offset * 5),
            payloadBytes: 20_480,
        });
        ensureSource(report, sourceNamespace);
        ensureBounded(report);
        if (object(report, "io").payloadBytes !== 20_480) {
            fail("adjacent signed terrain read volume mismatch");
        }
        await setAliasCamera(64 + offset);
        const valueProbe = await probe(target);
        globalContent(valueProbe);
        evidence.push({ offset, publication, probe: canonicalEvidence(valueProbe) });
    }
    const revisit = await publish("terrain.global.schedule", config(FAR));
    expectCounts(publishedTransaction(revisit), {
        retainedRegionCount: 10,
        uploadedRegionCount: 15,
        residentRegionCount: 50,
        payloadBytes: 61_440,
    });
    await setAliasCamera(64);
    evidence.push({ offset: 0, revisit, probe: canonicalEvidence(await probe(config(FAR))) });
    return evidence;
}

async function sourceSwitch(
    namespaceA: string,
    namespaceB: string,
    baseProbe: Record<string, unknown>,
): Promise<Record<string, unknown>> {
    await event("terrain.open", { path: OUTPUT_B });
    const variant = await publish("terrain.global.schedule", config(FAR));
    const variantReport = publishedTransaction(variant);
    expectCounts(variantReport, {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        residentRegionCount: 50,
        payloadBytes: 102_400,
    });
    ensureSource(variantReport, namespaceB);
    const variantProbe = await probe(config(FAR));
    if (
        globalContent(variantProbe).contentSha256 === globalContent(baseProbe).contentSha256
    ) fail("variant signed terrain content did not change");

    await event("terrain.open", { path: OUTPUT_A });
    const returned = await publish("terrain.global.schedule", config(FAR));
    const returnedReport = publishedTransaction(returned);
    expectCounts(returnedReport, {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        residentRegionCount: 50,
        payloadBytes: 0,
    });
    ensureSource(returnedReport, namespaceA);
    const returnedProbe = await probe(config(FAR));
    same(canonicalEvidence(returnedProbe), canonicalEvidence(baseProbe), "source revisit");
    return { variant, variantProbe, returned, returnedProbe };
}

await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":global-traversal"], "Experiment 0022 compatibility workflow");
const compatibility = {
    signedTraversal: "out/captures/0022-signed-camera-traversal/acceptance.json",
};
const centers: [number, number][] = [];
for (let offset = 0; offset <= 32; offset += 1) centers.push([FAR + offset, -FAR]);
centers.push([0, 0], [-FAR, FAR]);
const cookedA = await cookSigned(OUTPUT_A, centers);
const cookedARepeat = await cookSigned(OUTPUT_A_REPEAT, centers);
const cookedB = await cookSigned(OUTPUT_B, centers, 1);
if (
    cookedA.fileSha256 !== cookedARepeat.fileSha256 ||
    sourceNamespace(cookedA) !== sourceNamespace(cookedARepeat)
) fail("signed terrain recook was not deterministic");
if (sourceNamespace(cookedA) === sourceNamespace(cookedB)) {
    fail("variant packs share a source namespace");
}
await Deno.copyFile(`${root}/${OUTPUT_A}`, `${root}/${CORRUPT}`);
await corruptSignedIndexOffset(OUTPUT_A, BAD_INDEX);
const environment = await collectEnvironment(root);
const namespaceA = sourceNamespace(cookedA);
const namespaceB = sourceNamespace(cookedB);
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const correctnessRenderer = capability(firstStatus, true);
    const initial = await prepare(OUTPUT_A);
    const initialReport = publishedTransaction(initial);
    ensureSource(initialReport, namespaceA);
    const baseProbe = await probe(config(FAR));
    const baseCapture = await captureJoined("base", COLLECTION, baseProbe);
    const alias = await aliasRebind(namespaceA, baseProbe);
    const corridor = await adjacentCorridor(namespaceA);
    const sources = await sourceSwitch(namespaceA, namespaceB, baseProbe);

    const ioHold = await hold("io", config(FAR + 9), COLLECTION);
    const copyHold = await hold("copy", config(FAR + 10), COLLECTION);

    const beforeRejectProbe = await probe(config(FAR + 10));
    const beforeRejectCapture = await capture("reject-before", COLLECTION);
    const beforeRejectStatus = await event("terrain.status");
    const beforeRejectTransaction = field<number>(
        object(object(object(beforeRejectStatus, "renderer"), "published"), "transaction"),
        "transactionId",
        "number",
    );
    const missing = await failedEvent(
        "terrain.global.schedule",
        globalConfig(BASE, [FAR, -FAR + 32]),
        "stream_failed",
    );
    const missingState = await verifyStableFailure(
        beforeRejectProbe,
        beforeRejectCapture,
        beforeRejectTransaction,
        "missing-global",
    );
    const localRejected = await failedEvent("terrain.schedule", loadConfig(), "stream_failed");
    const compositionRejected = await failedEvent(
        "composition.schedule",
        loadConfig(),
        "stream_failed",
    );
    const unsupportedState = await verifyStableFailure(
        beforeRejectProbe,
        beforeRejectCapture,
        beforeRejectTransaction,
        "unsupported-mode",
    );

    await event("terrain.open", { path: CORRUPT });
    await publish("terrain.global.schedule", config(FAR));
    await setAliasCamera(64);
    const beforeCorruptProbe = await probe(config(FAR));
    const beforeCorruptCapture = await capture("corrupt-before", COLLECTION);
    const corruptStatus = await event("terrain.status");
    const corruptTransaction = field<number>(
        object(object(object(corruptStatus, "renderer"), "published"), "transaction"),
        "transactionId",
        "number",
    );
    const corruption = await corruptSignedPayload(CORRUPT, [FAR + 20, -FAR]);
    const corruptScheduled = await event("terrain.global.schedule", config(FAR + 20));
    await event("workbench.resume");
    const corruptFailure = await waitFailure();
    await event("workbench.pause");
    const corruptState = await verifyStableFailure(
        beforeCorruptProbe,
        beforeCorruptCapture,
        corruptTransaction,
        "corrupt-payload",
    );
    await restoreByte(CORRUPT, corruption);
    const retry = await publish("terrain.global.schedule", config(FAR + 20));
    const retryProbe = await probe(config(FAR + 20));
    const badIndex = await failedEvent("terrain.open", { path: BAD_INDEX }, "pack_open_failed");

    await lifecycle("restart");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("signed terrain process survived restart");
    const restarted = await prepare(OUTPUT_A);
    const restartProbe = await probe(config(FAR));
    const restartCapture = await captureJoined("restart", COLLECTION, restartProbe);
    same(canonicalEvidence(restartProbe), canonicalEvidence(baseProbe), "restart content");
    same(restartCapture, baseCapture, "restart signed terrain capture");

    await lifecycle("stop");
    await startClean("sidecar.benchmark.toml");
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = capability(benchmarkStatus, false);
    await prepare(OUTPUT_A);
    const adjacentTransactions = [];
    const adjacentOperator = [];
    const adjacentGpu = [];
    for (let offset = 1; offset <= 32; offset += 1) {
        const target = config(FAR + offset);
        const publication = await publish("terrain.global.schedule", target);
        const report = publishedTransaction(publication);
        expectCounts(report, {
            retainedRegionCount: 20,
            uploadedRegionCount: 5,
            payloadBytes: 20_480,
        });
        ensureBounded(report);
        adjacentTransactions.push(report);
        adjacentOperator.push(field<number>(publication, "operatorPublicationMs", "number"));
        await setAliasCamera(64 + offset);
        adjacentGpu.push(field<number>(object(await probe(target), "timing"), "totalMs", "number"));
    }

    await publish("terrain.global.schedule", config(FAR));
    await setAliasCamera(64);
    const benchmarkBase = await probe(config(FAR));
    const aliasTransactions = [];
    const aliasOperator = [];
    const aliasGpu = [];
    for (let index = 0; index < 32; index += 1) {
        const localCenter = index % 2 === 0 ? 96 : 64;
        const target = localCenter === 96 ? config(FAR, FAR - 32) : config(FAR);
        const publication = await publish("terrain.global.schedule", target);
        const report = publishedTransaction(publication);
        expectCounts(report, {
            retainedRegionCount: 25,
            uploadedRegionCount: 0,
            residentRegionCount: 50,
            payloadBytes: 0,
        });
        ensureBounded(report);
        aliasTransactions.push(report);
        aliasOperator.push(field<number>(publication, "operatorPublicationMs", "number"));
        await setAliasCamera(localCenter);
        const valueProbe = await probe(target);
        same(
            canonicalEvidence(valueProbe),
            canonicalEvidence(benchmarkBase),
            "benchmark alias content",
        );
        aliasGpu.push(field<number>(object(valueProbe, "timing"), "totalMs", "number"));
    }
    const benchmarkCapture = await captureJoined(
        "benchmark-final",
        COLLECTION,
        await probe(config(FAR)),
    );

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility,
        cooked: { primary: cookedA, repeat: cookedARepeat, variant: cookedB },
        capability: { correctness: correctnessRenderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        initial,
        base: { probe: baseProbe, capture: baseCapture },
        alias,
        corridor,
        sources,
        holds: { io: ioHold, copy: copyHold },
        rejection: {
            missing,
            missingState,
            localRejected,
            compositionRejected,
            unsupportedState,
        },
        corruption: {
            corruption,
            scheduled: corruptScheduled,
            failure: corruptFailure,
            state: corruptState,
            retry,
            retryProbe,
            badIndex,
        },
        restart: { publication: restarted, probe: restartProbe, capture: restartCapture },
        benchmark: {
            adjacent: {
                transactions: transactionDistributions(adjacentTransactions),
                operatorPublicationMs: distribution(adjacentOperator),
                terrainGpuMs: distribution(adjacentGpu),
            },
            alias: {
                transactions: transactionDistributions(aliasTransactions),
                operatorPublicationMs: distribution(aliasOperator),
                terrainGpuMs: distribution(aliasGpu),
            },
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
if (!finalReport) fail("signed terrain storage experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: REPORT }));
