import {
    canonicalObjects,
    capture,
    expectHalfCounts,
    expectPairMovement,
    hold,
    probe,
    probeFromPublished,
    publishPair,
    retainedObjectEvidence,
    stableFrame,
    stableObjectEvidence,
    waitPairFailure,
} from "../support/canonical-object-composition.ts";
import {
    aliasConfig,
    projectionShape,
    setAliasCamera,
} from "../support/camera-relative-terrain.ts";
import { compositionTimings } from "../support/composition.ts";
import { prepare } from "../support/global-composition.ts";
import {
    assertStopped,
    event,
    failedEvent,
    lifecycle,
    root,
    run,
    transactionDistributions,
    useSidecar,
} from "../support/global-terrain.ts";
import { corruptSignedPayload, restoreByte } from "../support/signed-terrain-storage.ts";
import { collectEnvironment, distribution, fail, field, object, same } from "../support/terrain.ts";

const REVISION = "canonical-generated-object-composition-v1";
const COLLECTION = "0025-canonical-object-composition";
const PACK_A = "out/terrain/0023-signed-terrain-storage/terrain-a.wlt";
const PACK_B = "out/terrain/0023-signed-terrain-storage/terrain-b.wlt";
const CORRUPT = `out/terrain/${COLLECTION}/terrain-corrupt.wlt`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];
const ALIASES = [2, 64, 96, 125];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-object-composition");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

function capability(status: Record<string, unknown>, debug: boolean) {
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== debug || renderer.deviceRemovedReason !== null) {
        fail(`${debug ? "debug" : "release"} canonical composition capability gate failed`);
    }
    return renderer;
}

async function startClean(config: string): Promise<void> {
    useSidecar(config);
    await lifecycle("stop");
    await lifecycle("start");
}

function half(
    publication: Record<string, unknown>,
    name: "terrain" | "instance",
): Record<string, unknown> {
    return object(object(publication, "halves"), name);
}

function objectDistributions(samples: Record<string, unknown>[]) {
    const values = (name: string) => samples.map((sample) => field<number>(sample, name, "number"));
    return {
        sampleCount: samples.length,
        payloadSources: [...new Set(samples.map((sample) => sample.payloadSource))],
        instanceBytes: [...new Set(values("instanceBytes"))],
        payloadPreparationMs: distribution(
            values("payloadPreparationMs"),
            "canonical object preparation",
            true,
        ),
        generationMs: distribution(
            values("generationMs"),
            "canonical object generation",
            true,
        ),
        scheduleMs: distribution(values("scheduleMs")),
        pendingMs: distribution(values("pendingMs")),
    };
}

async function exactAlias(
    center: number,
    referenceProbe: Record<string, unknown>,
    referenceCapture: Record<string, unknown>,
    id: string,
): Promise<Record<string, unknown>> {
    const config = aliasConfig(BASE, center);
    const publication = await publishPair(config);
    expectPairMovement(publication, 25, 0, 0, 0);
    await setAliasCamera(center);
    const value = await probe(config);
    const attachments = await capture(id, COLLECTION, value);
    same(
        stableFrame(value, attachments),
        stableFrame(referenceProbe, referenceCapture),
        `${id} canonical frame`,
    );
    return { center, publication, probe: value, capture: attachments };
}

async function sourceSwitch(
    baseConfig: ReturnType<typeof aliasConfig>,
    baseProbe: Record<string, unknown>,
    baseCapture: Record<string, unknown>,
): Promise<Record<string, unknown>> {
    await event("terrain.open", { path: PACK_B });
    const variant = await publishPair(baseConfig);
    expectHalfCounts(variant, "terrain", {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        payloadBytes: 102_400,
    });
    expectHalfCounts(variant, "instance", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        instanceBytes: 0,
    });
    const variantProbe = await probe(baseConfig);
    const variantCapture = await capture("source-b", COLLECTION, variantProbe);
    same(
        stableObjectEvidence(variantProbe),
        stableObjectEvidence(baseProbe),
        "terrain source switch object identity",
    );
    if (
        object(object(variantProbe, "terrain"), "globalContent").sourceNamespace ===
            object(object(baseProbe, "terrain"), "globalContent").sourceNamespace
    ) fail("terrain source switch retained the old namespace");

    await event("terrain.open", { path: PACK_A });
    const returned = await publishPair(baseConfig);
    expectHalfCounts(returned, "terrain", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        payloadBytes: 0,
    });
    expectHalfCounts(returned, "instance", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        instanceBytes: 0,
    });
    const returnedProbe = await probe(baseConfig);
    const returnedCapture = await capture("source-a-return", COLLECTION, returnedProbe);
    same(
        stableFrame(returnedProbe, returnedCapture),
        stableFrame(baseProbe, baseCapture),
        "terrain source revisit canonical frame",
    );
    return { variant, variantProbe, variantCapture, returned, returnedProbe, returnedCapture };
}

async function verifyCurrentFrame(
    beforeProbe: Record<string, unknown>,
    beforeCapture: Record<string, unknown>,
    label: string,
): Promise<Record<string, unknown>> {
    const afterProbe = await probeFromPublished();
    const afterCapture = await capture(`${label}-stable`, COLLECTION, afterProbe);
    same(
        stableFrame(afterProbe, afterCapture),
        stableFrame(beforeProbe, beforeCapture),
        `${label} complete published frame`,
    );
    return { probe: afterProbe, capture: afterCapture };
}

await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":camera-relative-terrain"], "Experiment 0024 compatibility workflow");
const compatibility = {
    cameraRelativeTerrain: "out/captures/0024-camera-relative-terrain/acceptance.json",
    legacyComposition: "out/captures/0021-signed-atomic-composition/acceptance.json",
};
await Deno.mkdir(`${root}/out/terrain/${COLLECTION}`, { recursive: true });
await Deno.copyFile(`${root}/${PACK_A}`, `${root}/${CORRUPT}`);
const corruption = await corruptSignedPayload(CORRUPT, [FAR + 20, -FAR]);
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const correctnessRenderer = capability(firstStatus, true);
    const baseConfig = aliasConfig(BASE, 64);
    const initial = await prepare(PACK_A, baseConfig);
    expectPairMovement(initial, 0, 25, 102_400, 512_000);
    const baseProbe = await probe(baseConfig);
    const baseCapture = await capture("base", COLLECTION, baseProbe);
    const baseFrame = stableFrame(baseProbe, baseCapture);

    const aliases = [];
    for (const [index, center] of [2, 64, 96, 125, 96, 64, 2, 125].entries()) {
        aliases.push(await exactAlias(center, baseProbe, baseCapture, `alias-${index}-${center}`));
    }
    await exactAlias(64, baseProbe, baseCapture, "alias-return-base");

    const sources = await sourceSwitch(baseConfig, baseProbe, baseCapture);

    const corridor = [];
    let previousProbe = await probe(baseConfig);
    let currentConfig = baseConfig;
    const projectionTemplate = projectionShape(object(baseProbe, "terrain"));
    for (let offset = 1; offset <= 8; offset += 1) {
        const center = ALIASES[(offset - 1) % ALIASES.length];
        const config = aliasConfig([FAR + offset, -FAR], center);
        const publication = await publishPair(config);
        expectPairMovement(publication, 20, 5, 20_480, 102_400);
        await setAliasCamera(center);
        const value = await probe(config);
        same(
            projectionShape(object(value, "terrain")),
            projectionTemplate,
            `corridor ${offset} projection shape`,
        );
        const retained = retainedObjectEvidence(previousProbe, value);
        const attachments = await capture(`corridor-${offset}`, COLLECTION, value);
        corridor.push({
            offset,
            center,
            publication,
            retained,
            probe: value,
            capture: attachments,
        });
        previousProbe = value;
        currentConfig = config;
    }

    const holds: Record<string, unknown> = {};
    for (const [index, kind] of ["terrain-io", "terrain-copy", "object-copy"].entries()) {
        const center = ALIASES[(index + 1) % ALIASES.length];
        const config = aliasConfig([FAR + 9 + index, -FAR], center);
        const held = await hold(
            kind as "terrain-io" | "terrain-copy" | "object-copy",
            config,
            COLLECTION,
        );
        expectPairMovement(held, 20, 5, 20_480, 102_400);
        await setAliasCamera(center);
        const value = await probe(config);
        const attachments = await capture(`${kind}-published`, COLLECTION, value);
        holds[kind] = { ...held, publishedProbe: value, publishedCapture: attachments };
        currentConfig = config;
    }

    const beforeRejectProbe = await probe(currentConfig);
    const beforeRejectCapture = await capture("reject-before", COLLECTION, beforeRejectProbe);
    const missing = await failedEvent(
        "composition.global.schedule",
        aliasConfig([FAR + 11, -FAR + 32], 64),
        "stream_failed",
    );
    const missingStable = await verifyCurrentFrame(
        beforeRejectProbe,
        beforeRejectCapture,
        "missing",
    );
    const meshletRejected = await failedEvent("meshlet.enable", {}, "meshlet_unavailable");
    const skeletalRejected = await failedEvent("skeletal.enable", {}, "skeletal_unavailable");
    const surfaceRejected = await failedEvent("surface.enable", {}, "surface_unavailable");
    const modeStable = await verifyCurrentFrame(
        beforeRejectProbe,
        beforeRejectCapture,
        "standalone-mode",
    );

    await event("terrain.open", { path: CORRUPT });
    const corruptConfig = aliasConfig([FAR + 20, -FAR], 64);
    const corruptScheduled = await event("composition.global.schedule", corruptConfig);
    const corruptToken = field<number>(corruptScheduled, "token", "number");
    await event("workbench.resume");
    const corruptFailure = await waitPairFailure(corruptToken);
    await event("workbench.pause");
    const corruptStable = await verifyCurrentFrame(
        beforeRejectProbe,
        beforeRejectCapture,
        "corrupt",
    );
    await restoreByte(CORRUPT, corruption);
    const retry = await publishPair(corruptConfig);
    expectHalfCounts(retry, "terrain", { uploadedRegionCount: 25 });
    expectHalfCounts(retry, "instance", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        instanceBytes: 0,
    });
    await setAliasCamera(64);
    const retryProbe = await probe(corruptConfig);
    const retryCapture = await capture("corrupt-retry", COLLECTION, retryProbe);

    await lifecycle("restart");
    const restartStatus = await event("workbench.status");
    const restartProcess = field<number>(restartStatus, "processId", "number");
    if (restartProcess === firstProcess) fail("canonical composition process survived restart");
    const restarted = await prepare(PACK_A, baseConfig);
    const restartProbe = await probe(baseConfig);
    const restartCapture = await capture("restart", COLLECTION, restartProbe);
    same(stableFrame(restartProbe, restartCapture), baseFrame, "canonical composition restart");

    await lifecycle("stop");
    await startClean("sidecar.benchmark.toml");
    const benchmarkStatus = await event("workbench.status");
    const benchmarkProcess = field<number>(benchmarkStatus, "processId", "number");
    const benchmarkRenderer = capability(benchmarkStatus, false);
    await prepare(PACK_A, baseConfig);
    const benchmarkBaseProbe = await probe(baseConfig);
    const benchmarkBaseCapture = await capture(
        "benchmark-base",
        COLLECTION,
        benchmarkBaseProbe,
    );
    const benchmarkBaseFrame = stableFrame(benchmarkBaseProbe, benchmarkBaseCapture);

    const aliasTerrain = [];
    const aliasObjects = [];
    const aliasProbes = [];
    const aliasPairMs = [];
    const aliasOperatorMs = [];
    const aliasCaptureMs = [];
    for (let index = 0; index < 32; index += 1) {
        const center = ALIASES[index % ALIASES.length];
        const config = aliasConfig(BASE, center);
        const publication = await publishPair(config);
        expectPairMovement(publication, 25, 0, 0, 0);
        await setAliasCamera(center);
        const value = await probe(config);
        const captureStarted = performance.now();
        const attachments = await capture(`benchmark-alias-${index}`, COLLECTION, value);
        aliasCaptureMs.push(performance.now() - captureStarted);
        same(
            stableFrame(value, attachments),
            benchmarkBaseFrame,
            `benchmark alias ${index}`,
        );
        aliasTerrain.push(half(publication, "terrain"));
        aliasObjects.push(half(publication, "instance"));
        aliasProbes.push(value);
        aliasPairMs.push(
            field<number>(object(publication, "published"), "publicationMs", "number"),
        );
        aliasOperatorMs.push(field<number>(publication, "operatorPublicationMs", "number"));
    }

    const adjacentTerrain = [];
    const adjacentObjects = [];
    const adjacentProbes = [];
    const adjacentPairMs = [];
    const adjacentOperatorMs = [];
    const adjacentCaptureMs = [];
    let benchmarkPrevious = benchmarkBaseProbe;
    let benchmarkFinal: Record<string, unknown> | undefined;
    for (let offset = 1; offset <= 32; offset += 1) {
        const center = ALIASES[(offset - 1) % ALIASES.length];
        const config = aliasConfig([FAR + offset, -FAR], center);
        const publication = await publishPair(config);
        expectPairMovement(publication, 20, 5, 20_480, 102_400);
        await setAliasCamera(center);
        const value = await probe(config);
        const retained = retainedObjectEvidence(benchmarkPrevious, value);
        const captureStarted = performance.now();
        const attachments = await capture(`benchmark-adjacent-${offset}`, COLLECTION, value);
        adjacentCaptureMs.push(performance.now() - captureStarted);
        adjacentTerrain.push(half(publication, "terrain"));
        adjacentObjects.push(half(publication, "instance"));
        adjacentProbes.push(value);
        adjacentPairMs.push(
            field<number>(object(publication, "published"), "publicationMs", "number"),
        );
        adjacentOperatorMs.push(field<number>(publication, "operatorPublicationMs", "number"));
        benchmarkPrevious = value;
        benchmarkFinal = { offset, center, retained, probe: value, capture: attachments };
    }
    if (!benchmarkFinal) fail("canonical composition benchmark produced no final sample");

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
        aliases,
        sources,
        corridor,
        holds,
        rejection: {
            missing,
            missingStable,
            meshletRejected,
            skeletalRejected,
            surfaceRejected,
            modeStable,
        },
        corruption: {
            corruption,
            scheduled: corruptScheduled,
            failure: corruptFailure,
            stable: corruptStable,
            retry,
            retryProbe,
            retryCapture,
        },
        restart: { publication: restarted, probe: restartProbe, capture: restartCapture },
        benchmark: {
            alias: {
                terrain: transactionDistributions(aliasTerrain),
                objects: objectDistributions(aliasObjects),
                composition: compositionTimings(aliasProbes),
                pairPublicationMs: distribution(aliasPairMs),
                operatorPublicationMs: distribution(aliasOperatorMs),
                captureMs: distribution(aliasCaptureMs),
            },
            adjacent: {
                terrain: transactionDistributions(adjacentTerrain),
                objects: objectDistributions(adjacentObjects),
                composition: compositionTimings(adjacentProbes),
                pairPublicationMs: distribution(adjacentPairMs),
                operatorPublicationMs: distribution(adjacentOperatorMs),
                captureMs: distribution(adjacentCaptureMs),
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
if (!finalReport) fail("canonical generated-object composition did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: REPORT }));
