import {
    assertStopped,
    capture,
    cook,
    event,
    expectCounts,
    failedEvent,
    failedRawEvent,
    globalConfig,
    globalProbe,
    hold,
    lifecycle,
    localProbe,
    probe,
    publish,
    root,
    run,
    transaction,
    transactionDistributions,
    useSidecar,
} from "../support/global-terrain.ts";
import {
    CANONICAL_MAPPING,
    CANONICAL_PAYLOAD,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    object,
    same,
} from "../support/terrain.ts";

const REVISION = "signed-terrain-addressing-v1";
const COLLECTION = "0020-signed-terrain-addressing";
const OUTPUT = `out/terrain/${COLLECTION}/terrain.wlt`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :global-terrain");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

function publishedTransaction(publication: Record<string, unknown>) {
    return transaction(publication);
}

function capability(status: Record<string, unknown>, debug: boolean) {
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== debug || renderer.deviceRemovedReason !== null) {
        fail(`${debug ? "debug" : "release"} terrain capability gate failed`);
    }
    return renderer;
}

function mappingHash(value: Record<string, unknown>): string {
    return field<string>(object(value, "globalAddressing"), "mappingSha256", "string");
}

function ensureBounded(report: Record<string, unknown>): void {
    if (
        field<number>(report, "residentRegionCount", "number") > 50 ||
        field<number>(report, "protectedRegionCount", "number") > 25
    ) fail("signed terrain transaction exceeded bounded cache ownership");
}

async function compatibility(): Promise<Record<string, string>> {
    await run("runseal", [":terrain"], "Experiment 0013 compatibility workflow");
    await run("runseal", [":region-traversal"], "Experiment 0018 compatibility workflow");
    return {
        terrain: "out/captures/0013-gpu-streamed-terrain/acceptance.json",
        traversal: "out/captures/0018-camera-driven-region-traversal/acceptance.json",
    };
}

async function startClean(config: string): Promise<void> {
    useSidecar(config);
    await lifecycle("stop");
    await lifecycle("start");
}

async function prepareGlobal(config: ReturnType<typeof globalConfig>) {
    await event("terrain.open", { path: OUTPUT });
    const publication = await publish("terrain.global.schedule", config);
    await event("terrain.enable");
    await event("camera.reset");
    await event("workbench.pause");
    return publication;
}

async function verifyRejectedState(
    beforeProbe: Record<string, unknown>,
    beforeCapture: Record<string, unknown>,
    transactionId: number,
    label: string,
): Promise<void> {
    const status = await event("terrain.status");
    const renderer = object(status, "renderer");
    if (
        object(status, "stream").pending !== null ||
        object(renderer, "transfer").reservation !== null ||
        object(object(renderer, "published"), "transaction").transactionId !== transactionId
    ) fail(`${label} mutated terrain transaction state`);
    same(globalProbe(await probe()), globalProbe(beforeProbe), `${label} mapping`);
    same(await capture(`${label}-held`, COLLECTION), beforeCapture, `${label} attachments`);
}

await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
const acceptedCompatibility = await compatibility();
const centers: [number, number][] = [];
for (let x = 63; x <= 96; x += 1) centers.push([x, 64]);
centers.push([63, 65], [64, 65], [65, 65]);
const cooked = await cook(OUTPUT, centers);
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const correctnessRenderer = capability(firstStatus, true);

    await event("terrain.open", { path: OUTPUT });
    const legacyPublication = await publish("terrain.schedule", loadConfig());
    await event("terrain.enable");
    await event("camera.reset");
    const legacyProbe = await probe();
    if (
        legacyProbe.globalAddressing !== undefined ||
        legacyProbe.activeMappingSha256 !== CANONICAL_MAPPING ||
        legacyProbe.payloadSha256 !== CANONICAL_PAYLOAD
    ) fail("legacy terrain contract changed");
    const legacyCapture = await capture("legacy", COLLECTION);

    const zero = globalConfig([0, 0]);
    const zeroPublication = await publish("terrain.global.schedule", zero);
    expectCounts(publishedTransaction(zeroPublication), {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        residentRegionCount: 50,
        payloadBytes: 102_400,
    });
    const zeroProbe = await probe(zero);
    const zeroCapture = await capture("anchor-zero", COLLECTION);
    same(localProbe(zeroProbe), localProbe(legacyProbe), "legacy/global local GPU output");
    same(zeroCapture, legacyCapture, "legacy/global attachments");

    const anchors: [number, number][] = [[FAR, -FAR], [-FAR, FAR]];
    const anchorEvidence: Record<string, unknown>[] = [];
    const hashes = new Set([mappingHash(zeroProbe)]);
    let farBaselineProbe: Record<string, unknown> | undefined;
    let farBaselineCapture: Record<string, unknown> | undefined;
    for (const [index, anchor] of anchors.entries()) {
        const config = globalConfig(anchor);
        const publication = await publish("terrain.global.schedule", config);
        const report = publishedTransaction(publication);
        expectCounts(report, {
            retainedRegionCount: 0,
            uploadedRegionCount: 25,
            evictedRegionCount: 25,
            residentRegionCount: 50,
            payloadBytes: 102_400,
        });
        const anchorProbe = await probe(config);
        const anchorCapture = await capture(`anchor-${index}`, COLLECTION);
        same(localProbe(anchorProbe), localProbe(zeroProbe), `anchor ${anchor} local output`);
        same(anchorCapture, zeroCapture, `anchor ${anchor} attachments`);
        hashes.add(mappingHash(anchorProbe));
        anchorEvidence.push({
            anchor,
            publication,
            probe: globalProbe(anchorProbe),
            capture: anchorCapture,
        });
        farBaselineProbe = anchorProbe;
        farBaselineCapture = anchorCapture;
    }
    if (hashes.size !== 3 || !farBaselineProbe || !farBaselineCapture) {
        fail("global anchor mapping hashes are not distinct");
    }

    const farOrigin: [number, number] = [-FAR, FAR];
    const xConfig = globalConfig(farOrigin, [-FAR + 1, FAR]);
    const xMove = await publish("terrain.global.schedule", xConfig);
    expectCounts(publishedTransaction(xMove), {
        retainedRegionCount: 20,
        uploadedRegionCount: 5,
        evictedRegionCount: 5,
        residentRegionCount: 50,
        payloadBytes: 20_480,
    });
    const baseConfig = globalConfig(farOrigin);
    const xRevisit = await publish("terrain.global.schedule", baseConfig);
    expectCounts(publishedTransaction(xRevisit), {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        residentRegionCount: 50,
        payloadBytes: 0,
    });
    await publish("terrain.global.schedule", xConfig);
    const xzConfig = globalConfig(farOrigin, [-FAR + 1, FAR + 1]);
    const zMove = await publish("terrain.global.schedule", xzConfig);
    expectCounts(publishedTransaction(zMove), {
        retainedRegionCount: 20,
        uploadedRegionCount: 5,
        evictedRegionCount: 5,
        residentRegionCount: 50,
        payloadBytes: 20_480,
    });
    const unionRevisit = await publish("terrain.global.schedule", baseConfig);
    expectCounts(publishedTransaction(unionRevisit), {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        residentRegionCount: 50,
        payloadBytes: 0,
    });
    same(globalProbe(await probe(baseConfig)), globalProbe(farBaselineProbe), "far revisit");
    same(await capture("far-revisit", COLLECTION), farBaselineCapture, "far revisit attachments");

    const ioHold = await hold(
        "io",
        globalConfig(farOrigin, [-FAR - 1, FAR]),
        COLLECTION,
    );
    const copyHold = await hold(
        "copy",
        globalConfig(farOrigin, [-FAR - 1, FAR + 1]),
        COLLECTION,
    );

    const beforeRejectProbe = await probe();
    const beforeRejectCapture = await capture("reject-before", COLLECTION);
    const beforeRejectStatus = await event("terrain.status");
    const beforeTransaction = field<number>(
        object(object(object(beforeRejectStatus, "renderer"), "published"), "transaction"),
        "transactionId",
        "number",
    );
    const missing = await failedEvent(
        "terrain.global.schedule",
        globalConfig(farOrigin, [-FAR, FAR + 32]),
        "stream_failed",
    );
    await verifyRejectedState(beforeRejectProbe, beforeRejectCapture, beforeTransaction, "missing");
    const outside = await failedEvent(
        "terrain.global.schedule",
        globalConfig([0, 0], [64, 0]),
        "invalid_global_terrain_config",
    );
    const overflow = await failedRawEvent(
        "terrain.global.schedule",
        '{"origin_x":-9223372036854775808,"origin_z":0,"center_x":9223372036854775807,"center_z":0,"active_radius":2}',
        "invalid_global_terrain_config",
    );
    await verifyRejectedState(beforeRejectProbe, beforeRejectCapture, beforeTransaction, "range");

    const aliasOrigin: [number, number] = [2 ** 39, 2 ** 39];
    const aliasConfig = globalConfig(aliasOrigin, [aliasOrigin[0] - 1, aliasOrigin[1] + 1]);
    const aliasPublication = await publish("terrain.global.schedule", aliasConfig);
    expectCounts(publishedTransaction(aliasPublication), {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        evictedRegionCount: 25,
        residentRegionCount: 50,
        payloadBytes: 102_400,
    });
    const aliasProbe = await probe(aliasConfig);
    const aliasCapture = await capture("alias-rebind", COLLECTION);
    same(localProbe(aliasProbe), localProbe(beforeRejectProbe), "changed-origin local output");
    same(aliasCapture, beforeRejectCapture, "changed-origin attachments");

    await lifecycle("restart");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("workbench process survived restart");
    await prepareGlobal(baseConfig);
    const restartProbe = await probe(baseConfig);
    const restartCapture = await capture("restart", COLLECTION);
    same(globalProbe(restartProbe), globalProbe(farBaselineProbe), "restart global mapping");
    same(localProbe(restartProbe), localProbe(farBaselineProbe), "restart local output");
    same(restartCapture, farBaselineCapture, "restart attachments");

    await lifecycle("stop");
    await startClean("sidecar.benchmark.toml");
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = capability(benchmarkStatus, false);
    const benchmarkOrigin: [number, number] = [FAR, -FAR];
    await prepareGlobal(globalConfig(benchmarkOrigin));
    const adjacent: Record<string, unknown>[] = [];
    const revisits: Record<string, unknown>[] = [];
    const operatorMs: number[] = [];
    const combinedGpuMs: number[] = [];
    for (let index = 1; index <= 32; index += 1) {
        const previous = globalConfig(benchmarkOrigin, [FAR + index - 1, -FAR]);
        const next = globalConfig(benchmarkOrigin, [FAR + index, -FAR]);
        const moved = await publish("terrain.global.schedule", next);
        const movedTransaction = publishedTransaction(moved);
        expectCounts(movedTransaction, {
            retainedRegionCount: 20,
            uploadedRegionCount: 5,
            payloadBytes: 20_480,
        });
        ensureBounded(movedTransaction);
        adjacent.push(movedTransaction);
        operatorMs.push(field<number>(moved, "operatorPublicationMs", "number"));
        const measuredProbe = await probe(next);
        combinedGpuMs.push(field<number>(object(measuredProbe, "timing"), "totalMs", "number"));

        const revisited = await publish("terrain.global.schedule", previous);
        const revisitTransaction = publishedTransaction(revisited);
        expectCounts(revisitTransaction, {
            retainedRegionCount: 25,
            uploadedRegionCount: 0,
            payloadBytes: 0,
        });
        ensureBounded(revisitTransaction);
        revisits.push(revisitTransaction);
        const restored = publishedTransaction(await publish("terrain.global.schedule", next));
        expectCounts(restored, {
            retainedRegionCount: 25,
            uploadedRegionCount: 0,
            payloadBytes: 0,
        });
    }
    const benchmarkCapture = await capture("benchmark-final", COLLECTION);

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility: acceptedCompatibility,
        cooked,
        capability: { correctness: correctnessRenderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        legacy: {
            publication: legacyPublication,
            probe: localProbe(legacyProbe),
            capture: legacyCapture,
        },
        zero: { publication: zeroPublication, probe: globalProbe(zeroProbe), capture: zeroCapture },
        anchors: anchorEvidence,
        movement: { xMove, xRevisit, zMove, unionRevisit },
        holds: { io: ioHold, copy: copyHold },
        rejection: { missing, outside, overflow },
        aliasRebind: {
            publication: aliasPublication,
            probe: globalProbe(aliasProbe),
            capture: aliasCapture,
        },
        restart: {
            processId: restartedProcess,
            probe: globalProbe(restartProbe),
            capture: restartCapture,
        },
        benchmark: {
            adjacent: transactionDistributions(adjacent),
            revisit: transactionDistributions(revisits),
            operatorPublicationMs: distribution(operatorMs),
            combinedGpuMs: distribution(combinedGpuMs),
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
if (!finalReport) fail("global terrain experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: REPORT }));
