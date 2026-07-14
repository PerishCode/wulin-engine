import { capability, packCenters } from "../support/canonical-origin-rollover-evidence.ts";
import {
    publication,
    startClean,
    target,
    traversal,
    waitPublished,
} from "../support/canonical-origin-rollover.ts";
import { expectHalfCounts, expectPairMovement } from "../support/canonical-object-composition.ts";
import {
    baseEvidence,
    cookObjects,
    corruptObjectPayload,
    objectStatus,
    validateObjectTransaction,
} from "../support/cooked-canonical-objects/mod.ts";
import { prepare, publishPair } from "../support/global-composition.ts";
import {
    prefetch,
    type PrefetchContext,
    setPosition,
    setupPrefetch,
    waitPrefetchFailure,
} from "../support/canonical-traversal-prefetch.ts";
import {
    preparedDirection,
    preparedRollover,
    promotedPrefetch,
} from "../support/canonical-traversal-prefetch-scenarios.ts";
import {
    preparedSweep,
    unpreparedSweep,
} from "../support/canonical-traversal-prefetch-evidence.ts";
import {
    assertStopped,
    event,
    lifecycle,
    root,
    run,
    sleep,
    useSidecar,
} from "../support/global-terrain.ts";
import { cookSigned } from "../support/signed-terrain-storage.ts";
import { collectEnvironment, fail, field, object, same } from "../support/terrain.ts";

const REVISION = "cooked-canonical-object-v1";
const COLLECTION = "0028-cooked-canonical-objects";
const TERRAIN_A = `out/terrain/${COLLECTION}/terrain-a.wlt`;
const TERRAIN_B = `out/terrain/${COLLECTION}/terrain-b.wlt`;
const OBJECT_A = `out/cooked/${COLLECTION}/objects-a.wlr`;
const OBJECT_REPEAT = `out/cooked/${COLLECTION}/objects-a-repeat.wlr`;
const OBJECT_B = `out/cooked/${COLLECTION}/objects-b.wlr`;
const OBJECT_MISSING = `out/cooked/${COLLECTION}/objects-missing.wlr`;
const OBJECT_CORRUPT = `out/cooked/${COLLECTION}/objects-corrupt.wlr`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: [number, number] = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :cooked-canonical-objects");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);

useSidecar("sidecar.toml");
await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");
await run(
    "runseal",
    [":canonical-traversal-prefetch"],
    "Experiment 0027 compatibility workflow",
);
await run(
    "cargo",
    ["test", "--locked", "-p", "region-format", "-p", "canonical-object-fixture"],
    "signed object codec tests",
);

const centers = [
    ...packCenters([BASE]),
    [BASE[0] + 1, BASE[1] + 1] as [number, number],
    [BASE[0] - 1, BASE[1] - 1] as [number, number],
];
await Deno.mkdir(`${root}/out/terrain/${COLLECTION}`, { recursive: true });
await Deno.mkdir(`${root}/out/cooked/${COLLECTION}`, { recursive: true });
const terrainA = await cookSigned(TERRAIN_A, centers);
const terrainB = await cookSigned(TERRAIN_B, centers, 1);
const objectA = await cookObjects(OBJECT_A, centers);
const objectRepeat = await cookObjects(OBJECT_REPEAT, centers);
const objectB = await cookObjects(OBJECT_B, [
    ...centers,
    [BASE[0] + 200, BASE[1] + 200],
]);
const objectMissing = await cookObjects(OBJECT_MISSING, [BASE]);
if (
    object(objectA, "metadata").sourceNamespaceSha256 !==
        object(objectRepeat, "metadata").sourceNamespaceSha256 ||
    objectA.fileSha256 !== objectRepeat.fileSha256
) fail("signed object cooker is not deterministic");
if (
    object(objectA, "metadata").sourceNamespaceSha256 ===
        object(objectB, "metadata").sourceNamespaceSha256
) fail("distinct signed object indexes share a source namespace");
await Deno.copyFile(`${root}/${OBJECT_A}`, `${root}/${OBJECT_CORRUPT}`);
const corruption = await corruptObjectPayload(OBJECT_CORRUPT, [BASE[0] + 73, BASE[1]]);

const environment = await collectEnvironment(root);
const baseConfig = target(BASE);
const context: PrefetchContext = {
    collection: COLLECTION,
    pack: TERRAIN_A,
    missingPack: "out/terrain/0023-signed-terrain-storage/terrain-a.wlt",
    corruptPack: "out/terrain/0027-canonical-traversal-prefetch/terrain-corrupt.wlt",
    objectPack: OBJECT_A,
    base: BASE,
};
let finalReport: Record<string, unknown> | undefined;

try {
    await startClean("sidecar.toml");
    const generatedBase = await baseEvidence(TERRAIN_A, baseConfig, COLLECTION);
    const generatedInstance = object(object(generatedBase, "halves"), "instance");
    await lifecycle("stop");

    await startClean("sidecar.toml");
    const cookedBase = await baseEvidence(TERRAIN_A, baseConfig, COLLECTION, OBJECT_A);
    const cookedInstance = object(object(cookedBase, "halves"), "instance");
    same(generatedBase.frame, cookedBase.frame, "generated/cooked canonical frame");
    if (
        generatedInstance.uploadedSha256 !== cookedInstance.uploadedSha256 ||
        generatedInstance.payloadSource !== "generated" ||
        cookedInstance.payloadSource !== "cooked-pack" || cookedInstance.generationMs !== 0
    ) fail("generated/cooked object payloads diverged");

    const adjacentConfig = target([BASE[0] + 1, BASE[1]], 65);
    const adjacent = await publishPair(adjacentConfig);
    expectPairMovement(adjacent, 20, 5, 5 * 4_096, 5 * 20_480);
    validateObjectTransaction(
        await objectStatus(),
        field<number>(object(object(adjacent, "halves"), "instance"), "transactionId", "number"),
        5,
    );
    const revisit = await publishPair(baseConfig);
    expectPairMovement(revisit, 25, 0, 0, 0);
    validateObjectTransaction(
        await objectStatus(),
        field<number>(object(object(revisit, "halves"), "instance"), "transactionId", "number"),
        0,
    );
    const alias = await publishPair(target(BASE, 63));
    expectPairMovement(alias, 25, 0, 0, 0);
    await lifecycle("stop");

    await startClean("sidecar.toml");
    await prepare(TERRAIN_A, baseConfig, OBJECT_A);
    const diagonalConfig = target([BASE[0] + 1, BASE[1] + 1], 65, 65);
    const diagonal = await publishPair(diagonalConfig);
    expectPairMovement(diagonal, 16, 9, 9 * 4_096, 9 * 20_480);
    validateObjectTransaction(
        await objectStatus(),
        field<number>(object(object(diagonal, "halves"), "instance"), "transactionId", "number"),
        9,
    );
    await lifecycle("stop");

    await startClean("sidecar.toml");
    await prepare(TERRAIN_A, baseConfig, OBJECT_A);
    await event("objects.open", { path: OBJECT_B });
    const objectSwitch = await publishPair(baseConfig);
    expectHalfCounts(objectSwitch, "terrain", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        payloadBytes: 0,
    });
    expectHalfCounts(objectSwitch, "instance", {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        instanceBytes: 25 * 20_480,
    });
    validateObjectTransaction(
        await objectStatus(),
        field<number>(
            object(object(objectSwitch, "halves"), "instance"),
            "transactionId",
            "number",
        ),
        25,
    );
    await event("terrain.open", { path: TERRAIN_B });
    const terrainSwitch = await publishPair(baseConfig);
    expectHalfCounts(terrainSwitch, "terrain", {
        retainedRegionCount: 0,
        uploadedRegionCount: 25,
        payloadBytes: 25 * 4_096,
    });
    expectHalfCounts(terrainSwitch, "instance", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        instanceBytes: 0,
    });
    validateObjectTransaction(
        await objectStatus(),
        field<number>(
            object(object(terrainSwitch, "halves"), "instance"),
            "transactionId",
            "number",
        ),
        0,
    );
    await lifecycle("stop");

    const prepared = await preparedDirection("cooked-positive-x", [1, 0], context, true);
    const promotions = {
        objectIo: await promotedPrefetch("object-io", context),
        objectCopy: await promotedPrefetch("object-copy", context),
    };
    const rollover = await preparedRollover(context);
    const missing = await objectFailure("missing", OBJECT_MISSING, context);
    const corrupt = await objectFailure("corrupt", OBJECT_CORRUPT, context);

    await startClean("sidecar.toml");
    await prepare(TERRAIN_A, baseConfig, OBJECT_A);
    const disabled = await event("objects.disable");
    const generatedAfterDisable = await publishPair(baseConfig);
    const disabledInstance = object(object(generatedAfterDisable, "halves"), "instance");
    if (
        disabled.disabled !== true || disabledInstance.payloadSource !== "generated" ||
        disabledInstance.uploadedRegionCount !== 25
    ) fail("cooked object disable did not restore generated sourcing");
    await lifecycle("stop");

    await startClean("sidecar.toml");
    const restart = await baseEvidence(TERRAIN_A, baseConfig, COLLECTION, OBJECT_A);
    same(cookedBase.frame, restart.frame, "cooked object restart frame");
    const correctnessStatus = await event("workbench.status");
    const correctnessRenderer = capability(correctnessStatus, true);
    const correctnessProcess = field<number>(correctnessStatus, "processId", "number");
    await lifecycle("stop");

    const control = await unpreparedSweep(context);
    const preparedRelease = await preparedSweep(context);
    finalReport = {
        revision: REVISION,
        outcome: "pass",
        environment,
        compatibility: {
            canonicalTraversalPrefetch:
                "out/captures/0027-canonical-traversal-prefetch/acceptance.json",
        },
        storage: {
            terrainA,
            terrainB,
            objectA,
            objectRepeat,
            objectB,
            objectMissing,
            corruption,
        },
        correctness: {
            processId: correctnessProcess,
            renderer: correctnessRenderer,
            generatedBase,
            cookedBase,
            adjacent,
            revisit,
            alias,
            diagonal,
            objectSwitch,
            terrainSwitch,
            prepared,
            promotions,
            rollover,
            missing,
            corrupt,
            disabled,
            generatedAfterDisable,
            restart,
        },
        benchmark: { control, prepared: preparedRelease },
    };
} finally {
    useSidecar("sidecar.toml");
    await lifecycle("stop");
    useSidecar("sidecar.benchmark.toml");
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("cooked canonical object workflow did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));

async function objectFailure(
    label: string,
    failingPack: string,
    context: PrefetchContext,
): Promise<Record<string, unknown>> {
    const start: [number, number] = [BASE[0] + 70, BASE[1]];
    const config = target(start);
    await setupPrefetch(TERRAIN_A, config, [0, 0], "sidecar.toml", OBJECT_A);
    const before = await event("composition.status");
    const token = field<number>(object(before, "published"), "token", "number");
    const publications = field<number>(traversal(before), "automaticPublicationCount", "number");
    const expected = target([start[0] + 1, start[1]], 65);
    await event("objects.open", { path: failingPack });
    await setPosition([5, 0]);
    const failed = await waitPrefetchFailure(expected, 1);
    if (
        field<number>(object(failed, "published"), "token", "number") !== token ||
        traversal(failed).blocked !== null
    ) fail(`${label} object failure mutated demand state`);
    const failedObjects = await objectStatus();
    if (!failedObjects.lastFailure) fail(`${label} object failure omitted diagnostics`);
    await sleep(100);
    const stable = await event("composition.status");
    if (prefetch(stable).failureCount !== 1) fail(`${label} object failure retried without motion`);
    await event("objects.open", { path: OBJECT_A });
    await setPosition([9, 0]);
    const published = await waitPublished(expected, publications + 1);
    await event("workbench.pause");
    const report = await publication(published, expected);
    expectHalfCounts(report, "terrain", {
        retainedRegionCount: 25,
        uploadedRegionCount: 0,
        payloadBytes: 0,
    });
    expectHalfCounts(report, "instance", {
        retainedRegionCount: 20,
        uploadedRegionCount: 5,
        instanceBytes: 5 * 20_480,
    });
    return { failed, failedObjects, stable, published, report };
}
