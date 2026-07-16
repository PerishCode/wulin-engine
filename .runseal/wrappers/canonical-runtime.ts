import {
    assertObjectCopies,
    assertStopped,
    type Coord,
    event,
    fail,
    failedPair,
    frame,
    holdPair,
    type Json,
    lifecycle,
    lifecycleCycles,
    number,
    object,
    openSources,
    probe,
    publish,
    resourcePlateau,
    root,
    same,
    setAliasCamera,
    setPosition,
    sleep,
    startClean,
    status,
    stopCanonicalProcesses,
    target,
    targetMatches,
    traversalSweep,
    waitStatus,
} from "../support/canonical-runtime.ts";
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
import { bootstrapGates as bootstrapGate } from "../support/runtime-bootstrap.ts";
import { prototypeHostGates } from "../support/prototype/host.ts";
import { terrainQueryGates, unavailableTerrainQueryGate } from "../support/terrain/query.ts";
import {
    terrainContactGates as contactGates,
    unavailableTerrainContactGate as unavailableContact,
} from "../support/terrain/contact.ts";
import { compatibilityRemovalGates } from "../support/compatibility-removal.ts";
import { actorGates } from "../support/actor/lifecycle.ts";
import { simulationActorGates } from "../support/actor/simulation.ts";
import {
    objectQueryGates,
    queryObject,
    queryObjectSamples,
    rejectedObjectQuery,
    sameObjectQueries,
    unavailableObjectQueryGate,
} from "../support/object-query.ts";

const REVISION = "canonical-runtime-v1";
const COLLECTION = "canonical-runtime";
const FAR = 2 ** 40;
const BASE: Coord = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-runtime");
    Deno.exit(0);
}
if (Deno.args.length !== 0) fail(`canonical-runtime: unexpected argument ${Deno.args[0]}`);

const setup = await prepareCanonicalSetup(COLLECTION, BASE);
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
    const bootstrap = await bootstrapGate(TERRAIN, OBJECTS_A, OBJECTS_CORRUPT, BASE, COLLECTION);
    const prototype = await prototypeHostGates(TERRAIN, OBJECTS_A, OBJECTS_CORRUPT, BASE);
    const actor = await actorGates();
    const simulationActor = await simulationActorGates(TERRAIN, OBJECTS_A, BASE);
    await startClean();
    const idle = await status();
    const compatibilityRemoval = await compatibilityRemovalGates(COLLECTION, idle);
    const unavailableObjectQuery = await unavailableObjectQueryGate(BASE);
    const unavailableTerrainQuery = await unavailableTerrainQueryGate(BASE);
    const unavailableTerrainContact = await unavailableContact(BASE);
    await openSources(TERRAIN, OBJECTS_A);
    const basePublication = await publish(target(BASE));
    assertObjectCopies(basePublication, 25, "cold publication");
    const objectQuery = await objectQueryGates(OBJECTS_A, BASE, unavailableObjectQuery);
    const orderAObjectQueries = objectQuery.samples as Json[];
    const orderA = await frame("order-a", COLLECTION);
    const terrainQuery = await terrainQueryGates(BASE, orderA, unavailableTerrainQuery);
    const terrainContact = await contactGates(BASE, orderA, unavailableTerrainContact);
    assertCanonicalFrame(orderA, "canonical order A");

    console.log("==> deterministic presentation time gates");
    const temporal = await temporalGates(orderA, COLLECTION);

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
        const mutated = await frame(`presentation-${label}`, COLLECTION);
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
        if (
            label !== "animation" && mutatedCapture.color === baseCapture.color &&
            mutatedCapture.png === baseCapture.png
        ) fail(`${label} presentation mutation did not change rendered color evidence`);
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
    );

    await event("source.objects.open", { path: OBJECTS_A });
    await publish(target(BASE));

    await event("source.objects.open", { path: OBJECTS_B });
    const orderBPublication = await publish(target(BASE));
    assertObjectCopies(orderBPublication, 25, "order B source publication");
    const orderBObjectQueries = await queryObjectSamples(
        OBJECTS_B,
        BASE,
        [0, 511, 1_023],
    );
    sameObjectQueries(orderBObjectQueries, orderAObjectQueries, "physical object order A/B query");
    const orderB = await frame("order-b", COLLECTION);
    same(orderB.stable, orderA.stable, "physical object order A/B behavior");

    await event("source.objects.open", { path: OBJECTS_A });
    const revisitPublication = await publish(target(BASE));
    assertObjectCopies(revisitPublication, 0, "order A source revisit");
    const revisitObjectQueries = await queryObjectSamples(
        OBJECTS_A,
        BASE,
        [0, 511, 1_023],
    );
    sameObjectQueries(revisitObjectQueries, orderAObjectQueries, "object source revisit query");
    const revisit = await frame("order-a-revisit", COLLECTION);
    same(revisit.stable, orderA.stable, "object source revisit");

    const retiredAdjacentRegion: Coord = [BASE[0] - 2, BASE[1]];
    const admittedAdjacentRegion: Coord = [BASE[0] + 3, BASE[1]];
    const adjacentOldBefore = await queryObject(OBJECTS_A, retiredAdjacentRegion, 0);
    const adjacentPublication = await publish(target([BASE[0] + 1, BASE[1]]));
    assertObjectCopies(adjacentPublication, 5, "adjacent publication");
    const adjacentOldAfter = await rejectedObjectQuery(
        retiredAdjacentRegion,
        0,
        "retired adjacent-window object query",
    );
    const adjacentNew = await queryObject(OBJECTS_A, admittedAdjacentRegion, 0);
    const adjacentObjectQuery = { adjacentOldBefore, adjacentOldAfter, adjacentNew };
    const adjacent = await frame("adjacent", COLLECTION);
    const diagonalBasePublication = await publish(target([BASE[0] + 40, BASE[1]]));
    assertObjectCopies(diagonalBasePublication, 25, "diagonal cold base");
    const diagonalPublication = await publish(target([BASE[0] + 41, BASE[1] + 1]));
    assertObjectCopies(diagonalPublication, 9, "diagonal publication");
    const diagonal = await frame("diagonal", COLLECTION);
    const returnedPublication = await publish(target(BASE));
    const returned = await frame("returned", COLLECTION);
    same(returned.stable, orderA.stable, "movement revisit");

    const aliasPublication = await publish(target(BASE, 65));
    await setAliasCamera(65);
    const alias = await frame("compensated-alias", COLLECTION);
    same(alias.stable, orderA.stable, "compensated alias frame");
    await event("camera.reset");
    await publish(target(BASE));

    const temporalHeld = await temporalHold(
        await frame("temporal-hold-before", COLLECTION),
        COLLECTION,
        BASE,
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
        const before = await frame(`hold-${index}-before`, COLLECTION);
        holds.push(
            await holdPair(
                gate,
                target([BASE[0] + index + 2, BASE[1]]),
                before,
                COLLECTION,
            ),
        );
    }
    const beforeFailure = await frame("failure-before", COLLECTION);
    const failurePublishedRegion: Coord = [BASE[0] + 5, BASE[1]];
    const failureObjectBefore = await queryObject(OBJECTS_A, failurePublishedRegion, 511);
    await event("source.objects.open", { path: OBJECTS_CORRUPT });
    const objectFailure = await failedPair(
        target([BASE[0] + 70, BASE[1]]),
        beforeFailure,
        COLLECTION,
        "object-corrupt",
    );
    const failureObjectAfterObject = await queryObject(OBJECTS_A, failurePublishedRegion, 511);
    sameObjectQueries(
        [failureObjectAfterObject],
        [failureObjectBefore],
        "object-corrupt rollback query",
    );
    await event("source.objects.open", { path: OBJECTS_A });
    await event("source.terrain.open", { path: TERRAIN_CORRUPT });
    const terrainFailure = await failedPair(
        target([BASE[0] + 75, BASE[1]]),
        beforeFailure,
        COLLECTION,
        "terrain-corrupt",
    );
    const failureObjectAfterTerrain = await queryObject(OBJECTS_A, failurePublishedRegion, 511);
    sameObjectQueries(
        [failureObjectAfterTerrain],
        [failureObjectBefore],
        "terrain-corrupt rollback object query",
    );
    const failureObjectQuery = {
        before: failureObjectBefore,
        afterObjectFailure: failureObjectAfterObject,
        afterTerrainFailure: failureObjectAfterTerrain,
    };
    await event("source.terrain.open", { path: TERRAIN });

    const firstProcessId = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(firstProcessId);
    await lifecycle("start");
    await openSources(TERRAIN, OBJECTS_A);
    const restartPublication = await publish(target(BASE));
    const restartObjectQueries = await queryObjectSamples(
        OBJECTS_A,
        BASE,
        [0, 511, 1_023],
    );
    sameObjectQueries(restartObjectQueries, orderAObjectQueries, "canonical restart object query");
    const restarted = await frame("restart", COLLECTION);
    same(restarted.stable, orderA.stable, "canonical restart frame");

    console.log("==> prepared rollover gate");
    const rolloverBase = await publish(target(BASE, 96));
    await setPosition([512, 0]);
    await event("canonical.traversal.enable");
    await event("canonical.prefetch.enable");
    await event("workbench.resume");
    await sleep(30);
    const rolloverBefore = await event("canonical.status");
    const traversalBefore = object(rolloverBefore, "traversal");
    const automaticBefore = number(traversalBefore, "automaticPublicationCount");
    const rolloverCount = number(object(traversalBefore, "rollover"), "count");
    const prefetchBefore = number(object(traversalBefore, "prefetch"), "completionCount");
    const rolloverTarget = target([BASE[0] + 1, BASE[1]]);
    await setPosition([517, 0]);
    const rolloverPrepared = await waitStatus("rollover preparation", (value) => {
        if (value.pending !== null) return false;
        const prefetch = object(object(value, "traversal"), "prefetch");
        return number(prefetch, "completionCount") === prefetchBefore + 1 &&
            targetMatches(prefetch.lastCompleted, rolloverTarget);
    });
    if (
        number(object(object(rolloverPrepared, "traversal"), "rollover"), "count") !== rolloverCount
    ) {
        fail("prepared rollover committed before demand");
    }
    await setPosition([521, 0]);
    const rolloverPublished = await waitStatus("rollover publication", (value) => {
        if (value.pending !== null || !targetMatches(value.published, rolloverTarget)) return false;
        const traversal = object(value, "traversal");
        return number(traversal, "automaticPublicationCount") === automaticBefore + 1 &&
            number(object(traversal, "rollover"), "count") === rolloverCount + 1;
    });
    await event("workbench.pause");
    const rolloverProbe = await probe();
    await event("canonical.traversal.disable");
    await lifecycle("stop");

    console.log("==> 32 reactive crossings");
    await startClean("sidecar.benchmark.toml");
    await openSources(TERRAIN, OBJECTS_A);
    await publish(target(BASE));
    const reactive = await traversalSweep(BASE, false);
    const reactiveProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(reactiveProcess);

    console.log("==> 32 prepared crossings");
    await startClean("sidecar.benchmark.toml");
    await openSources(TERRAIN, OBJECTS_A);
    await publish(target(BASE));
    const prepared = await traversalSweep(BASE, true);
    const preparedProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(preparedProcess);

    console.log("==> 64-publication same-process resource plateau");
    await startClean();
    await openSources(TERRAIN, OBJECTS_A);
    await publish(target(BASE));
    const plateau = await resourcePlateau(BASE);
    const plateauProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(plateauProcess);

    console.log("==> 16 complete lifecycle cycles");
    const lifecycleEvidence = await lifecycleCycles(TERRAIN, OBJECTS_A, target(BASE));

    acceptance = {
        revision: REVISION,
        outcome: "pass",
        storage: setup.storage,
        correctness: {
            bootstrap,
            prototype,
            actor,
            simulationActor,
            compatibilityRemoval,
            objectQuery,
            terrainQuery,
            terrainContact,
            basePublication,
            orderA,
            temporal,
            presentationMutations,
            importedPublication,
            importedFrame,
            importedTickSixteen,
            sourceDuration,
            orderBPublication,
            orderBObjectQueries,
            orderB,
            revisitPublication,
            revisitObjectQueries,
            revisit,
            adjacentPublication,
            adjacentObjectQuery,
            adjacent,
            diagonalBasePublication,
            diagonalPublication,
            diagonal,
            returnedPublication,
            returned,
            aliasPublication,
            alias,
            temporalHeld,
            holds,
            objectFailure,
            terrainFailure,
            failureObjectQuery,
            restartPublication,
            restartObjectQueries,
            restarted,
            rolloverBase,
            rolloverPrepared,
            rolloverPublished,
            rolloverProbe,
        },
        traversal: { reactive, prepared },
        resources: plateau,
        lifecycle: lifecycleEvidence,
    };
} finally {
    await stopCanonicalProcesses();
}

if (!acceptance) fail("canonical runtime workflow did not produce acceptance evidence");
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(acceptance, null, 2)}\n`);
console.log(JSON.stringify({ outcome: acceptance.outcome, report: REPORT }, null, 2));
