import {
    assertObjectCopies,
    assertStopped,
    collectionInventory,
    type Coord,
    event,
    fail,
    frame,
    holdPair,
    type Json,
    lifecycle,
    lifecycleCycles,
    number,
    object,
    openSources,
    operationMetrics,
    probe,
    publish,
    recordStage,
    resourceCheckpoint,
    root,
    same,
    setAliasCamera,
    startClean,
    status,
    stopCanonicalProcesses,
    target,
    traversalSweep,
} from "../support/canonical-runtime.ts";
import { preparedRolloverGate } from "../support/canonical-rollover.ts";
import { assertCanonicalFrame } from "../support/canonical-frame.ts";
import { prepareCanonicalSetup } from "../support/canonical-setup.ts";
import {
    presentationInvariant,
    temporalGates,
    temporalHold,
} from "../support/temporal-presentation.ts";
import {
    importedPresentationGates,
    sourceDurationGates,
} from "../support/cooked-gltf-presentation.ts";
import {
    bootstrapGates as bootstrapGate,
    prototypeHostCheckpointGates,
} from "../support/runtime-bootstrap.ts";
import { terrainQueryGates, unavailableTerrainQueryGate } from "../support/terrain/query.ts";
import { idleShellGates } from "../support/idle-shell.ts";
import { actorGates } from "../support/actor/lifecycle.ts";
import { simulationActorGates } from "../support/actor/simulation.ts";
import {
    OBJECT_RESOLUTION_SAMPLE_IDS,
    objectResolutionGates,
    resolveObjectSamples,
    sameObjectResolutions,
    unavailableObjectResolutionGate,
} from "../support/object/query.ts";
import {
    objectNearestGates,
    objectNearestSamples,
    queryObjectNearestSamples,
    sameObjectNearestQueries,
    unavailableObjectNearestGate,
} from "../support/object/nearest.ts";
import * as objectIntegration from "../support/object/integration.ts";
import {
    assertUntargetedFrame,
    beginTargetLifecycle,
    confirmTargetRevisit,
    objectSuppressionLifecycle,
    targetDepartureReturn,
    visibleObjectTarget,
} from "../support/object/feedback.ts";
const REVISION = "canonical-runtime-v13";
const COLLECTION = "canonical-runtime";
const BASE: Coord = [2 ** 40, -(2 ** 40)];
if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-runtime");
    Deno.exit(0);
}
if (Deno.args.length !== 0) fail(`canonical-runtime: unexpected argument ${Deno.args[0]}`);
const started = performance.now();
const stageTimings: Json[] = [];
const setupStarted = performance.now();
const setup = await prepareCanonicalSetup(COLLECTION, BASE);
recordStage(stageTimings, "setup", setupStarted);
const {
    terrain: TERRAIN,
    objectsA: OBJECTS_A,
    objectsB: OBJECTS_B,
    objectsArchetype: OBJECTS_ARCHETYPE,
    objectsMaterial: OBJECTS_MATERIAL,
    objectsYaw: OBJECTS_YAW,
    objectsAnimation: OBJECTS_ANIMATION,
    objectsImported: OBJECTS_IMPORTED,
    objectsImportedDuration: OBJECTS_IMPORTED_DURATION,
    objectsCorrupt: OBJECTS_CORRUPT,
    terrainCorrupt: TERRAIN_CORRUPT,
    report: REPORT,
} = setup.paths;

let acceptance: Json | undefined;
try {
    console.log("==> canonical correctness and failure gates");
    const bootstrapStarted = performance.now();
    const bootstrap = await bootstrapGate(TERRAIN, OBJECTS_A, OBJECTS_CORRUPT, BASE, COLLECTION);
    recordStage(stageTimings, "bootstrap", bootstrapStarted);
    const prototypeStarted = performance.now();
    const prototype = await prototypeHostCheckpointGates(
        TERRAIN,
        OBJECTS_A,
        OBJECTS_CORRUPT,
        BASE,
    );
    recordStage(stageTimings, "prototype", prototypeStarted);
    const actorStarted = performance.now();
    const actor = await actorGates();
    recordStage(stageTimings, "actor-lifecycle", actorStarted);
    const simulationActorStarted = performance.now();
    const simulationActor = await simulationActorGates(TERRAIN, OBJECTS_A, BASE);
    recordStage(stageTimings, "simulation-actor", simulationActorStarted);
    const correctnessStarted = performance.now();
    await startClean();
    const idle = await status();
    const idleShell = await idleShellGates(COLLECTION, idle);
    const unavailableObjectResolution = await unavailableObjectResolutionGate(BASE);
    const unavailableObjectNearest = await unavailableObjectNearestGate(BASE);
    const unavailableTerrainQuery = await unavailableTerrainQueryGate(BASE);
    await openSources(TERRAIN, OBJECTS_A);
    const basePublication = await publish(target(BASE));
    assertObjectCopies(basePublication, 25, "cold publication");
    const objectResolution = await objectResolutionGates(
        OBJECTS_A,
        BASE,
        unavailableObjectResolution,
        basePublication,
    );
    const orderAObjectResolutions = objectResolution.samples as Json[];
    const nearestSamples = objectNearestSamples(BASE);
    const objectNearest = await objectNearestGates(
        OBJECTS_A,
        BASE,
        unavailableObjectNearest,
    );
    const orderAObjectNearest = objectNearest.samples as Json[];
    const orderA = await frame("order-a", COLLECTION);
    const terrainQuery = await terrainQueryGates(BASE, orderA, unavailableTerrainQuery);
    assertCanonicalFrame(orderA, "canonical order A");
    const feedbackTarget = visibleObjectTarget(
        orderA,
        object(objectResolution, "snapshot").sourceNamespace as string,
        BASE,
    );

    console.log("==> deterministic presentation time gates");
    const temporal = await temporalGates(orderA, COLLECTION, false);

    const presentationMutations: Json[] = [];
    for (
        const [label, path] of [
            ["archetype", OBJECTS_ARCHETYPE],
            ["material", OBJECTS_MATERIAL],
            ["yaw", OBJECTS_YAW],
            ["animation", OBJECTS_ANIMATION],
        ] as const
    ) {
        await event("source.objects.open", { path });
        const publication = await publish(target(BASE));
        assertObjectCopies(publication, 25, `${label} source publication`);
        const mutated = await frame(`presentation-${label}`, COLLECTION, false, false);
        const baseStable = object(orderA, "stable");
        const mutatedStable = object(mutated, "stable");
        same(
            presentationInvariant(mutatedStable),
            presentationInvariant(baseStable),
            `${label} presentation spatial/identity invariants`,
        );
        const baseObjects = object(baseStable, "objects");
        const mutatedObjects = object(mutatedStable, "objects");
        if (
            mutatedObjects.presentationKeyedSha256 === baseObjects.presentationKeyedSha256
        ) fail(`${label} presentation mutation did not change cooked authority`);
        const baseSkeletal = object(baseStable, "skeletal");
        const mutatedSkeletal = object(mutatedStable, "skeletal");
        if (label === "material" || label === "yaw") {
            same(mutatedSkeletal.gpu, baseSkeletal.gpu, `${label} skeletal invariance`);
        } else if (JSON.stringify(mutatedSkeletal.gpu) === JSON.stringify(baseSkeletal.gpu)) {
            fail(`${label} presentation mutation did not change skeletal evidence`);
        }
        const baseCapture = object(baseStable, "capture");
        const mutatedCapture = object(mutatedStable, "capture");
        if (label !== "animation" && mutatedCapture.color === baseCapture.color) {
            fail(`${label} presentation mutation did not change rendered color evidence`);
        }
        presentationMutations.push({ label, publication, frame: mutated });
    }

    console.log("==> cooked glTF geometry/material/skeletal animation gate");
    const imported = await importedPresentationGates(
        orderA,
        OBJECTS_IMPORTED,
        BASE,
        COLLECTION,
    );
    const importedPublication = object(imported, "publication");
    const importedFrame = object(imported, "tickZero");
    const importedTickSixteen = object(imported, "tickSixteen");

    console.log("==> source-duration Walk loop gate");
    const sourceDuration = await sourceDurationGates(
        importedFrame,
        OBJECTS_IMPORTED_DURATION,
        BASE,
        COLLECTION,
        false,
    );

    const initialTargetFeedback = await beginTargetLifecycle(
        feedbackTarget,
        orderA,
        OBJECTS_A,
        BASE,
        COLLECTION,
    );
    const targetPixelsBeforeReplacement = number(initialTargetFeedback, "pixels");

    await event("source.objects.open", { path: OBJECTS_B });
    const orderBPublication = await publish(target(BASE));
    assertObjectCopies(orderBPublication, 25, "order B source publication");
    const orderBObjects = await objectIntegration.differentObjectSourceGates(
        OBJECTS_B,
        BASE,
        OBJECT_RESOLUTION_SAMPLE_IDS,
        orderAObjectResolutions,
        nearestSamples,
        orderAObjectNearest,
    );
    const orderBObjectResolutions = orderBObjects.resolutions as Json[];
    const orderBObjectNearest = orderBObjects.nearest as Json[];
    const orderB = await frame("order-b", COLLECTION, false, false);
    assertUntargetedFrame(orderB, "source-replaced target frame");
    same(orderB.stable, orderA.stable, "physical object order A/B behavior");

    await event("source.objects.open", { path: OBJECTS_A });
    const revisitPublication = await publish(target(BASE));
    assertObjectCopies(revisitPublication, 0, "order A source revisit");
    const staleOrderBIdentity = await objectIntegration.resolveStaleObjectIdentity(
        orderBObjectResolutions[0],
        "source-replaced",
        "order B identity after order A revisit",
    );
    const revisitObjectResolutions = await resolveObjectSamples(
        OBJECTS_A,
        BASE,
        OBJECT_RESOLUTION_SAMPLE_IDS,
    );
    sameObjectResolutions(
        revisitObjectResolutions,
        orderAObjectResolutions,
        "object source revisit resolution",
    );
    const revisitObjectNearest = await queryObjectNearestSamples(OBJECTS_A, nearestSamples);
    sameObjectNearestQueries(
        revisitObjectNearest,
        orderAObjectNearest,
        "object source revisit nearest query",
    );
    const revisitTargetFeedback = await confirmTargetRevisit(
        feedbackTarget,
        orderA,
        targetPixelsBeforeReplacement,
        COLLECTION,
    );
    const revisit = await frame("order-a-revisit", COLLECTION, false, false);
    same(revisit.stable, orderA.stable, "object source revisit");

    const adjacentObjects = await objectIntegration.adjacentObjectGates(
        OBJECTS_A,
        BASE,
        nearestSamples,
        orderAObjectNearest,
    );
    const adjacent = await frame("adjacent", COLLECTION, false, false);
    const departureTargetFeedback = await targetDepartureReturn(
        feedbackTarget,
        orderA,
        targetPixelsBeforeReplacement,
        BASE,
        COLLECTION,
    );
    const suppressionLifecycle = await objectSuppressionLifecycle(
        feedbackTarget,
        orderA,
        OBJECTS_A,
        OBJECTS_B,
        BASE,
        COLLECTION,
    );

    const aliasPublication = await publish(target(BASE, 65));
    await setAliasCamera(65);
    const alias = await frame("compensated-alias", COLLECTION, false, false);
    same(alias.stable, orderA.stable, "compensated alias frame");
    await event("camera.reset");
    await publish(target(BASE));

    const temporalHeld = await temporalHold(
        await frame("temporal-hold-before", COLLECTION, false, false),
        COLLECTION,
        BASE,
        false,
    );

    const holds: Json[] = [];
    for (
        const [index, gate] of [
            "canonical.objects.io_gate",
            "canonical.objects.copy_gate",
            "canonical.terrain.io_gate",
            "canonical.terrain.copy_gate",
        ].entries()
    ) {
        const before = await frame(`hold-${index}-before`, COLLECTION, false, false);
        holds.push(
            await holdPair(
                gate,
                target([BASE[0] + index + 2, BASE[1]]),
                before,
                COLLECTION,
                false,
            ),
        );
    }
    const failures = await objectIntegration.objectFailureGates(
        COLLECTION,
        TERRAIN,
        OBJECTS_A,
        OBJECTS_CORRUPT,
        TERRAIN_CORRUPT,
        BASE,
    );

    const firstProcessId = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(firstProcessId);
    await lifecycle("start");
    await openSources(TERRAIN, OBJECTS_A);
    const restartPublication = await publish(target(BASE));
    const restartObjectResolutions = await resolveObjectSamples(
        OBJECTS_A,
        BASE,
        OBJECT_RESOLUTION_SAMPLE_IDS,
    );
    sameObjectResolutions(
        restartObjectResolutions,
        orderAObjectResolutions,
        "canonical restart object resolution",
    );
    const restartObjectNearest = await queryObjectNearestSamples(OBJECTS_A, nearestSamples);
    sameObjectNearestQueries(
        restartObjectNearest,
        orderAObjectNearest,
        "canonical restart nearest query",
    );
    const restarted = await frame("restart", COLLECTION, false, false);
    same(restarted.stable, orderA.stable, "canonical restart frame");

    console.log("==> prepared rollover gate");
    const rollover = await preparedRolloverGate(BASE);
    recordStage(stageTimings, "canonical-correctness", correctnessStarted);

    console.log("==> 32 reactive crossings");
    const reactiveStarted = performance.now();
    await startClean("sidecar.benchmark.toml");
    await openSources(TERRAIN, OBJECTS_A);
    await publish(target(BASE));
    const reactive = await traversalSweep(BASE, false);
    const reactiveProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(reactiveProcess);
    recordStage(stageTimings, "reactive-traversal", reactiveStarted);

    console.log("==> 32 prepared crossings");
    const preparedStarted = performance.now();
    await startClean("sidecar.benchmark.toml");
    await openSources(TERRAIN, OBJECTS_A);
    await publish(target(BASE));
    const prepared = await traversalSweep(BASE, true);
    const preparedProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(preparedProcess);
    recordStage(stageTimings, "prepared-traversal", preparedStarted);

    console.log("==> 8-publication same-process resource checkpoint");
    const resourcesStarted = performance.now();
    await startClean();
    await openSources(TERRAIN, OBJECTS_A);
    await publish(target(BASE));
    const resources = await resourceCheckpoint(BASE);
    const plateauProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(plateauProcess);
    recordStage(stageTimings, "resource-checkpoint", resourcesStarted);

    console.log("==> 2 complete lifecycle checkpoint cycles");
    const lifecycleStarted = performance.now();
    const lifecycleEvidence = await lifecycleCycles(TERRAIN, OBJECTS_A, target(BASE), 2);
    recordStage(stageTimings, "lifecycle-checkpoint", lifecycleStarted);

    acceptance = {
        revision: REVISION,
        outcome: "pass",
        storage: setup.storage,
        correctness: {
            bootstrap,
            prototype,
            actor,
            simulationActor,
            idleShell,
            objectResolution,
            objectNearest,
            terrainQuery,
            objectTargetFeedback: {
                identity: feedbackTarget.identity,
                targetSetBeforeReplacement: initialTargetFeedback.targetSet,
                targetBeforeReplacement: initialTargetFeedback.rendered,
                targetPixelsBeforeReplacement,
                sourceReplaced: orderB,
                sourceRevisited: revisitTargetFeedback.rendered,
                revisitTargetedPixels: revisitTargetFeedback.pixels,
                targetClearedAfterReplacement: revisitTargetFeedback.cleared,
                targetSetBeforeDeparture: departureTargetFeedback.targetSet,
                sameSourceDeparted: departureTargetFeedback.departed,
                sameSourceReturned: departureTargetFeedback.returnedTargeted,
                returnedTargetedPixels: departureTargetFeedback.pixels,
                targetClearedAfterReturn: departureTargetFeedback.cleared,
            },
            objectSuppression: suppressionLifecycle,
            basePublication,
            orderA,
            temporal,
            presentationMutations,
            importedPublication,
            importedFrame,
            importedTickSixteen,
            sourceDuration,
            orderBPublication,
            staleOrderAIdentity: orderBObjects.staleIdentity,
            orderBObjectResolutions,
            orderBObjectNearest,
            orderB,
            revisitPublication,
            staleOrderBIdentity,
            revisitObjectResolutions,
            revisitObjectNearest,
            revisit,
            adjacentPublication: adjacentObjects.publication,
            adjacentObjectResolution: adjacentObjects.resolution,
            adjacentObjectNearest: adjacentObjects.nearest,
            adjacent,
            aliasPublication,
            alias,
            temporalHeld,
            holds,
            objectFailure: failures.objectFailure,
            terrainFailure: failures.terrainFailure,
            failureObjectResolution: failures.objectResolution,
            failureObjectNearest: failures.objectNearest,
            restartPublication,
            restartObjectResolutions,
            restartObjectNearest,
            restarted,
            rolloverBase: rollover.basePublication,
            rolloverPrepared: rollover.prepared,
            rolloverPublished: rollover.published,
            rolloverProbe: rollover.evidence,
        },
        traversal: { reactive, prepared },
        resources,
        lifecycle: lifecycleEvidence,
    };
} finally {
    await stopCanonicalProcesses();
}

if (!acceptance) fail("canonical runtime workflow did not produce acceptance evidence");
acceptance.metrics = {
    elapsedMilliseconds: performance.now() - started,
    stages: stageTimings,
    operations: operationMetrics(),
    artifacts: await collectionInventory(COLLECTION),
    deepResourceAndLifecycleOwner: "runseal :canonical-resources",
};
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(acceptance, null, 2)}\n`);
console.log(
    JSON.stringify(
        { outcome: acceptance.outcome, report: REPORT, metrics: acceptance.metrics },
        null,
        2,
    ),
);
