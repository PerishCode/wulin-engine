import {
    assertStopped,
    event,
    expectCounts,
    lifecycle,
    prepare,
    publish,
    root,
    run,
    transaction,
    transactionDistributions,
    useSidecar,
} from "../support/global-terrain.ts";
import {
    aliasConfig,
    aliasEvidence,
    projectionHold,
    projectionProbe,
    projectionShape,
    setAliasCamera,
    stableFrame,
    stableProjection,
} from "../support/camera-relative-terrain.ts";
import { captureJoined, globalContent, globalSlots } from "../support/signed-terrain-storage.ts";
import { collectEnvironment, distribution, fail, field, object, same } from "../support/terrain.ts";

const REVISION = "camera-relative-terrain-v1";
const COLLECTION = "0024-camera-relative-terrain";
const PACK = "out/terrain/0023-signed-terrain-storage/terrain-a.wlt";
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];
const ALIASES = [2, 64, 96, 125];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :camera-relative-terrain");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

function capability(status: Record<string, unknown>, debug: boolean) {
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== debug || renderer.deviceRemovedReason !== null) {
        fail(`${debug ? "debug" : "release"} terrain projection capability gate failed`);
    }
    return renderer;
}

async function startClean(config: string): Promise<void> {
    useSidecar(config);
    await lifecycle("stop");
    await lifecycle("start");
}

async function exactAlias(
    center: number,
    referenceProbe: Record<string, unknown>,
    referenceCapture: Record<string, unknown>,
    lodEnabled: boolean,
    id: string,
): Promise<Record<string, unknown>> {
    const config = aliasConfig(BASE, center);
    const publication = await publish("terrain.global.schedule", config);
    const report = transaction(publication);
    expectCounts(report, {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        evictedRegionCount: 0,
        residentRegionCount: 25,
        payloadBytes: 0,
    });
    if (object(report, "io").payloadBytes !== 0) fail("terrain alias performed I/O");
    await setAliasCamera(center);
    const probe = await projectionProbe(config, lodEnabled);
    const capture = await captureJoined(id, COLLECTION, probe);
    same(stableProjection(probe), stableProjection(referenceProbe), `${id} projection`);
    same(globalSlots(probe), globalSlots(referenceProbe), `${id} canonical slots`);
    same(stableFrame(probe, capture), stableFrame(referenceProbe, referenceCapture), `${id} frame`);
    return { center, publication, projection: aliasEvidence(probe), probe, capture };
}

await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":signed-terrain-storage"], "Experiment 0023 compatibility workflow");
const compatibility = {
    signedTerrainStorage: "out/captures/0023-signed-terrain-storage/acceptance.json",
};
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const correctnessRenderer = capability(firstStatus, true);
    const baseConfig = aliasConfig(BASE, 64);
    const initial = await prepare(PACK, baseConfig);
    const baseProbe = await projectionProbe(baseConfig);
    const baseCapture = await captureJoined("base", COLLECTION, baseProbe);
    const baseFrame = stableFrame(baseProbe, baseCapture);

    const fullAliases = [];
    for (const [index, center] of [2, 64, 96, 125, 96, 64, 2, 125].entries()) {
        fullAliases.push(
            await exactAlias(center, baseProbe, baseCapture, false, `full-${index}-${center}`),
        );
    }

    await publish("terrain.global.schedule", baseConfig);
    await setAliasCamera(64);
    await event("terrain.lod.enable");
    const lodBaseProbe = await projectionProbe(baseConfig, true);
    const lodBaseCapture = await captureJoined("lod-base", COLLECTION, lodBaseProbe);
    const lodAliases = [];
    for (const center of ALIASES) {
        lodAliases.push(
            await exactAlias(center, lodBaseProbe, lodBaseCapture, true, `lod-${center}`),
        );
    }
    await event("terrain.lod.disable");
    await publish("terrain.global.schedule", baseConfig);
    await setAliasCamera(64);

    const projectionTemplate = projectionShape(baseProbe);
    const corridor = [];
    let currentConfig = baseConfig;
    for (let offset = 1; offset <= 8; offset += 1) {
        const center = ALIASES[(offset - 1) % ALIASES.length];
        const config = aliasConfig([FAR + offset, -FAR], center);
        const publication = await publish("terrain.global.schedule", config);
        const report = transaction(publication);
        expectCounts(report, {
            retainedRegionCount: 20,
            uploadedRegionCount: 5,
            residentRegionCount: Math.min(50, 25 + offset * 5),
            payloadBytes: 20_480,
        });
        await setAliasCamera(center);
        const probe = await projectionProbe(config);
        same(projectionShape(probe), projectionTemplate, `corridor ${offset} projection shape`);
        globalContent(probe);
        const capture = await captureJoined(`corridor-${offset}`, COLLECTION, probe);
        corridor.push({ offset, center, publication, probe, capture });
        currentConfig = config;
    }

    const ioConfig = aliasConfig([FAR + 9, -FAR], 2);
    const ioHold = await projectionHold("io", currentConfig, ioConfig, COLLECTION);
    await setAliasCamera(2);
    const ioProbe = await projectionProbe(ioConfig);
    same(projectionShape(ioProbe), projectionTemplate, "I/O publication projection shape");
    const copyConfig = aliasConfig([FAR + 10, -FAR], 125);
    const copyHold = await projectionHold("copy", ioConfig, copyConfig, COLLECTION);
    await setAliasCamera(125);
    const copyProbe = await projectionProbe(copyConfig);
    same(projectionShape(copyProbe), projectionTemplate, "copy publication projection shape");

    await lifecycle("restart");
    const restartStatus = await event("workbench.status");
    const restartProcess = field<number>(restartStatus, "processId", "number");
    if (restartProcess === firstProcess) fail("terrain projection process survived restart");
    const restarted = await prepare(PACK, baseConfig);
    const restartProbe = await projectionProbe(baseConfig);
    const restartCapture = await captureJoined("restart", COLLECTION, restartProbe);
    same(stableFrame(restartProbe, restartCapture), baseFrame, "terrain projection restart");

    await lifecycle("stop");
    await startClean("sidecar.benchmark.toml");
    const benchmarkStatus = await event("workbench.status");
    const benchmarkProcess = field<number>(benchmarkStatus, "processId", "number");
    const benchmarkRenderer = capability(benchmarkStatus, false);
    await prepare(PACK, baseConfig);
    const benchmarkBaseProbe = await projectionProbe(baseConfig);
    const benchmarkBaseCapture = await captureJoined(
        "benchmark-base",
        COLLECTION,
        benchmarkBaseProbe,
    );
    const benchmarkBaseFrame = stableFrame(benchmarkBaseProbe, benchmarkBaseCapture);
    const aliasTransactions = [];
    const aliasOperator = [];
    const aliasGpu = [];
    const aliasCapture = [];
    for (let index = 0; index < 32; index += 1) {
        const center = ALIASES[index % ALIASES.length];
        const config = aliasConfig(BASE, center);
        const publication = await publish("terrain.global.schedule", config);
        const report = transaction(publication);
        expectCounts(report, {
            retainedRegionCount: 25,
            uploadedRegionCount: 0,
            residentRegionCount: 25,
            payloadBytes: 0,
        });
        await setAliasCamera(center);
        const probe = await projectionProbe(config);
        const captureStarted = performance.now();
        const capture = await captureJoined(`benchmark-alias-${index}`, COLLECTION, probe);
        aliasCapture.push(performance.now() - captureStarted);
        same(stableFrame(probe, capture), benchmarkBaseFrame, `benchmark alias ${index}`);
        aliasTransactions.push(report);
        aliasOperator.push(field<number>(publication, "operatorPublicationMs", "number"));
        aliasGpu.push(field<number>(object(probe, "timing"), "totalMs", "number"));
    }

    const adjacentTransactions = [];
    const adjacentOperator = [];
    const adjacentGpu = [];
    const adjacentCapture = [];
    const benchmarkShape = projectionShape(benchmarkBaseProbe);
    let benchmarkFinal: Record<string, unknown> | undefined;
    for (let offset = 1; offset <= 32; offset += 1) {
        const center = ALIASES[(offset - 1) % ALIASES.length];
        const config = aliasConfig([FAR + offset, -FAR], center);
        const publication = await publish("terrain.global.schedule", config);
        const report = transaction(publication);
        expectCounts(report, {
            retainedRegionCount: 20,
            uploadedRegionCount: 5,
            residentRegionCount: Math.min(50, 25 + offset * 5),
            payloadBytes: 20_480,
        });
        await setAliasCamera(center);
        const probe = await projectionProbe(config);
        same(projectionShape(probe), benchmarkShape, `benchmark adjacent ${offset} shape`);
        const captureStarted = performance.now();
        const capture = await captureJoined(`benchmark-adjacent-${offset}`, COLLECTION, probe);
        adjacentCapture.push(performance.now() - captureStarted);
        adjacentTransactions.push(report);
        adjacentOperator.push(field<number>(publication, "operatorPublicationMs", "number"));
        adjacentGpu.push(field<number>(object(probe, "timing"), "totalMs", "number"));
        benchmarkFinal = { offset, center, probe, capture };
    }
    if (!benchmarkFinal) fail("terrain projection benchmark produced no final sample");

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility,
        capability: { correctness: correctnessRenderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartProcess, benchmark: benchmarkProcess },
        initial,
        base: { probe: baseProbe, capture: baseCapture },
        fullAliases,
        lod: { base: { probe: lodBaseProbe, capture: lodBaseCapture }, aliases: lodAliases },
        corridor,
        holds: { io: ioHold, copy: copyHold },
        restart: { publication: restarted, probe: restartProbe, capture: restartCapture },
        benchmark: {
            alias: {
                transactions: transactionDistributions(aliasTransactions),
                operatorPublicationMs: distribution(aliasOperator),
                terrainGpuMs: distribution(aliasGpu),
                captureMs: distribution(aliasCapture),
            },
            adjacent: {
                transactions: transactionDistributions(adjacentTransactions),
                operatorPublicationMs: distribution(adjacentOperator),
                terrainGpuMs: distribution(adjacentGpu),
                captureMs: distribution(adjacentCapture),
            },
            final: benchmarkFinal,
        },
    };
} finally {
    await lifecycle("stop");
    useSidecar("sidecar.toml");
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("camera-relative terrain experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: REPORT }));
