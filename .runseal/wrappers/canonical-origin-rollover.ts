import {
    capture,
    expectPairMovement,
    probe,
    stableFrame,
} from "../support/canonical-object-composition.ts";
import {
    capability,
    objectTimings,
    packCenters,
} from "../support/canonical-origin-rollover-evidence.ts";
import {
    boundary,
    disableCatchUp,
    failures,
    heldRollover,
    normalization,
    type ScenarioContext,
} from "../support/canonical-origin-rollover-scenarios.ts";
import {
    type Coord,
    enable,
    publication,
    rollover,
    setCamera,
    startClean,
    target,
    traversal,
    waitPublished,
} from "../support/canonical-origin-rollover.ts";
import { compositionTimings } from "../support/composition.ts";
import {
    type GlobalConfig,
    globalConfig,
    prepare,
    publishPair,
} from "../support/global-composition.ts";
import {
    assertStopped,
    event,
    lifecycle,
    root,
    run,
    transactionDistributions,
    useSidecar,
} from "../support/global-terrain.ts";
import { cookSigned, corruptSignedPayload } from "../support/signed-terrain-storage.ts";
import { collectEnvironment, distribution, fail, field, object, same } from "../support/terrain.ts";
import { traversal as baseTraversal } from "../support/traversal.ts";

const REVISION = "canonical-origin-rollover-v1";
const COLLECTION = "0026-canonical-origin-rollover";
const PACK = `out/terrain/${COLLECTION}/terrain.wlt`;
const CORRUPT = `out/terrain/${COLLECTION}/terrain-corrupt.wlt`;
const PACK_0023 = "out/terrain/0023-signed-terrain-storage/terrain-a.wlt";
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: Coord = [FAR, -FAR];
const OTHER: Coord = [-FAR, FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-origin-rollover");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run("runseal", [":canonical-object-composition"], "Experiment 0025 compatibility workflow");
const compatibility = {
    canonicalComposition: "out/captures/0025-canonical-object-composition/acceptance.json",
    frozenOriginTraversal: "out/captures/0022-signed-camera-traversal/acceptance.json",
};
await Deno.mkdir(`${root}/out/terrain/${COLLECTION}`, { recursive: true });
const cooked = await cookSigned(PACK, packCenters([BASE, OTHER]));
await Deno.copyFile(`${root}/${PACK}`, `${root}/${CORRUPT}`);
const corruption = await corruptSignedPayload(CORRUPT, [BASE[0] + 72, BASE[1]]);
const scenarios: ScenarioContext = {
    base: BASE,
    collection: COLLECTION,
    pack: PACK,
    corruptPack: CORRUPT,
    missingPack: PACK_0023,
    corruption,
};
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const firstStatus = await event("workbench.status");
    const firstProcess = field<number>(firstStatus, "processId", "number");
    const correctnessRenderer = capability(firstStatus, true);
    const initial = await prepare(PACK, target(BASE, 97));
    expectPairMovement(initial, 0, 25, 102_400, 512_000);
    const normalized = await normalization(BASE, [97, 64], "normalize", scenarios);
    const boundaries = {
        positiveX: await boundary("positive-x", BASE, [96, 64], [97, 64], scenarios),
        negativeX: await boundary("negative-x", BASE, [32, 64], [31, 64], scenarios),
        positiveZ: await boundary("positive-z", BASE, [64, 96], [64, 97], scenarios),
        negativeZ: await boundary("negative-z", BASE, [64, 32], [64, 31], scenarios),
        positiveDiagonal: await boundary(
            "positive-diagonal",
            OTHER,
            [96, 96],
            [97, 97],
            scenarios,
        ),
        negativeDiagonal: await boundary(
            "negative-diagonal",
            OTHER,
            [32, 32],
            [31, 31],
            scenarios,
        ),
    };
    const holds = {
        terrainIo: await heldRollover("terrain-io", scenarios),
        terrainCopy: await heldRollover("terrain-copy", scenarios),
        objectCopy: await heldRollover("object-copy", scenarios),
    };
    const failureEvidence = await failures(scenarios);
    const disabled = await disableCatchUp(scenarios);

    await lifecycle("restart");
    const restartStatus = await event("workbench.status");
    const restartProcess = field<number>(restartStatus, "processId", "number");
    if (restartProcess === firstProcess) fail("rollover workbench process survived restart");
    const restartInitial = await prepare(PACK, target(BASE, 97));
    const restarted = await normalization(BASE, [97, 64], "restart", scenarios);
    same(
        stableFrame(object(restarted, "afterProbe"), object(restarted, "afterCapture")),
        stableFrame(object(normalized, "afterProbe"), object(normalized, "afterCapture")),
        "rollover restart frame",
    );

    await lifecycle("stop");
    await startClean("sidecar.benchmark.toml");
    const benchmarkStatus = await event("workbench.status");
    const benchmarkProcess = field<number>(benchmarkStatus, "processId", "number");
    const benchmarkRenderer = capability(benchmarkStatus, false);
    await prepare(PACK, target(BASE));
    await setCamera(64);
    await enable();
    const ordinaryTerrain = [];
    const ordinaryObjects = [];
    const ordinaryProbes = [];
    const ordinaryPairMs = [];
    const ordinaryCaptureMs = [];
    for (let offset = 1; offset <= 32; offset += 1) {
        const expected = globalConfig(BASE, [BASE[0] + offset, BASE[1]]);
        const before = traversal(await event("composition.status"));
        await setCamera(64 + offset);
        const status = await waitPublished(
            expected,
            field<number>(before, "automaticPublicationCount", "number") + 1,
        );
        const report = await publication(status, expected);
        expectPairMovement(report, 20, 5, 20_480, 102_400);
        const value = await probe(expected);
        const started = performance.now();
        await capture(`benchmark-ordinary-${offset}`, COLLECTION, value);
        ordinaryCaptureMs.push(performance.now() - started);
        ordinaryTerrain.push(object(object(report, "halves"), "terrain"));
        ordinaryObjects.push(object(object(report, "halves"), "instance"));
        ordinaryProbes.push(value);
        ordinaryPairMs.push(field<number>(object(report, "published"), "publicationMs", "number"));
    }

    await event("composition.traversal.disable");
    await publishPair(target(BASE));
    await setCamera(64);
    const benchmarkBaseProbe = await probe(target(BASE));
    const benchmarkBaseCapture = await capture("benchmark-base", COLLECTION, benchmarkBaseProbe);
    const benchmarkBaseFrame = stableFrame(benchmarkBaseProbe, benchmarkBaseCapture);
    const rolloverTerrain = [];
    const rolloverObjects = [];
    const rolloverProbes = [];
    const rolloverPairMs = [];
    const rolloverCaptureMs = [];
    const rolloverDeltas = [];
    for (let index = 0; index < 32; index += 1) {
        await publishPair(target(BASE, 97));
        await setCamera(97);
        const before = baseTraversal(await event("composition.status"));
        await enable();
        const status = await waitPublished(
            target(BASE),
            field<number>(before, "automaticPublicationCount", "number") + 1,
        );
        const report = await publication(status, target(BASE));
        expectPairMovement(report, 25, 0, 0, 0);
        const value = await probe(target(BASE));
        const started = performance.now();
        const attachments = await capture(`benchmark-rollover-${index}`, COLLECTION, value);
        rolloverCaptureMs.push(performance.now() - started);
        same(stableFrame(value, attachments), benchmarkBaseFrame, `benchmark rollover ${index}`);
        rolloverTerrain.push(object(object(report, "halves"), "terrain"));
        rolloverObjects.push(object(object(report, "halves"), "instance"));
        rolloverProbes.push(value);
        rolloverPairMs.push(field<number>(object(report, "published"), "publicationMs", "number"));
        rolloverDeltas.push(object(rollover(status), "last"));
        await event("composition.traversal.disable");
    }

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility,
        cooked,
        corruption,
        capability: { correctness: correctnessRenderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartProcess, benchmark: benchmarkProcess },
        initial,
        normalized,
        boundaries,
        holds,
        failures: failureEvidence,
        disabled,
        restart: { initial: restartInitial, normalization: restarted },
        benchmark: {
            ordinary: {
                terrain: transactionDistributions(ordinaryTerrain),
                objects: objectTimings(ordinaryObjects),
                composition: compositionTimings(ordinaryProbes),
                pairPublicationMs: distribution(ordinaryPairMs),
                captureMs: distribution(ordinaryCaptureMs),
            },
            rollover: {
                terrain: transactionDistributions(rolloverTerrain),
                objects: objectTimings(rolloverObjects),
                composition: compositionTimings(rolloverProbes),
                pairPublicationMs: distribution(rolloverPairMs),
                captureMs: distribution(rolloverCaptureMs),
                deltas: rolloverDeltas,
            },
        },
    };
} finally {
    await lifecycle("stop");
    useSidecar("sidecar.toml");
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("canonical origin rollover did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({ outcome: finalReport.outcome, report: REPORT }));
