import {
    assertStopped,
    capture,
    cookObjects,
    cookTerrain,
    type Coord,
    corruptObjects,
    corruptTerrain,
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
    run,
    same,
    setAliasCamera,
    setPosition,
    sleep,
    startClean,
    status,
    string,
    target,
    targetMatches,
    traversalSweep,
    useSidecar,
    waitStatus,
} from "../support/canonical-runtime.ts";

const REVISION = "canonical-runtime-convergence-v1";
const COLLECTION = "0031-canonical-runtime-convergence";
const DIRECTORY = `out/cooked/${COLLECTION}`;
const TERRAIN = `${DIRECTORY}/terrain.wlt`;
const OBJECTS_A = `${DIRECTORY}/objects-a.wlr`;
const OBJECTS_B = `${DIRECTORY}/objects-b.wlr`;
const OBJECTS_CORRUPT = `${DIRECTORY}/objects-corrupt.wlr`;
const TERRAIN_CORRUPT = `${DIRECTORY}/terrain-corrupt.wlt`;
const REPORT = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;
const BASE: Coord = [FAR, -FAR];

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :canonical-runtime");
    Deno.exit(0);
}
if (Deno.args.length !== 0) fail(`canonical-runtime: unexpected argument ${Deno.args[0]}`);

await Deno.mkdir(`${root}/${DIRECTORY}`, { recursive: true });
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
useSidecar("sidecar.toml");
await lifecycle("stop");
useSidecar("sidecar.benchmark.toml");
await lifecycle("stop");
useSidecar("sidecar.toml");

await run(
    "cargo",
    [
        "test",
        "--locked",
        "-p",
        "terrain-format",
        "-p",
        "region-format",
        "-p",
        "terrain-cooker",
        "-p",
        "region-cooker",
    ],
    "canonical codec and cooker tests",
);

const centers: Coord[] = [];
for (let offset = -1; offset <= 80; offset += 1) {
    centers.push([BASE[0] + offset, BASE[1]]);
}
centers.push([BASE[0] + 1, BASE[1] + 1]);
const terrainCook = await cookTerrain(TERRAIN, centers);
const objectCookA = await cookObjects(OBJECTS_A, centers, "a");
const objectCookB = await cookObjects(OBJECTS_B, centers, "b");
const metadataA = object(objectCookA, "metadata");
const metadataB = object(objectCookB, "metadata");
if (
    metadataA.payloadSchema !== 2 || metadataB.payloadSchema !== 2 ||
    metadataA.stableSeedNamespaceSha256 !== metadataB.stableSeedNamespaceSha256 ||
    metadataA.sourceNamespaceSha256 === metadataB.sourceNamespaceSha256 ||
    string(objectCookA, "fileSha256") === string(objectCookB, "fileSha256")
) fail("canonical object order/source identity gate failed");

await Deno.copyFile(`${root}/${OBJECTS_A}`, `${root}/${OBJECTS_CORRUPT}`);
await Deno.copyFile(`${root}/${TERRAIN}`, `${root}/${TERRAIN_CORRUPT}`);
const objectCorruption = await corruptObjects(OBJECTS_CORRUPT, [BASE[0] + 70, BASE[1]]);
const terrainCorruption = await corruptTerrain(TERRAIN_CORRUPT, [BASE[0] + 75, BASE[1]]);

let acceptance: Json | undefined;
try {
    console.log("==> canonical correctness and failure gates");
    await startClean();
    const idle = await status();
    if (object(idle, "workload").mode !== "idle-shell") {
        fail("workbench did not start in the idle shell");
    }
    await openSources(TERRAIN, OBJECTS_A);
    const basePublication = await publish(target(BASE));
    const orderA = await frame("order-a", COLLECTION);

    await event("source.objects.open", { path: OBJECTS_B });
    const orderBPublication = await publish(target(BASE));
    const orderB = await frame("order-b", COLLECTION);
    same(orderB.stable, orderA.stable, "physical object order A/B behavior");

    await event("source.objects.open", { path: OBJECTS_A });
    const revisitPublication = await publish(target(BASE));
    const revisit = await frame("order-a-revisit", COLLECTION);
    same(revisit.stable, orderA.stable, "object source revisit");

    const adjacentPublication = await publish(target([BASE[0] + 1, BASE[1]]));
    const adjacent = await frame("adjacent", COLLECTION);
    const diagonalPublication = await publish(target([BASE[0] + 1, BASE[1] + 1]));
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
    await event("source.objects.open", { path: OBJECTS_CORRUPT });
    const objectFailure = await failedPair(
        target([BASE[0] + 70, BASE[1]]),
        beforeFailure,
        COLLECTION,
        "object-corrupt",
    );
    await event("source.objects.open", { path: OBJECTS_A });
    await event("source.terrain.open", { path: TERRAIN_CORRUPT });
    const terrainFailure = await failedPair(
        target([BASE[0] + 75, BASE[1]]),
        beforeFailure,
        COLLECTION,
        "terrain-corrupt",
    );
    await event("source.terrain.open", { path: TERRAIN });

    const firstProcessId = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(firstProcessId);
    await lifecycle("start");
    await openSources(TERRAIN, OBJECTS_A);
    const restartPublication = await publish(target(BASE));
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
        storage: {
            terrain: terrainCook,
            objectsA: objectCookA,
            objectsB: objectCookB,
            objectCorruption,
            terrainCorruption,
        },
        correctness: {
            idle,
            basePublication,
            orderA,
            orderBPublication,
            orderB,
            revisitPublication,
            revisit,
            adjacentPublication,
            adjacent,
            diagonalPublication,
            diagonal,
            returnedPublication,
            returned,
            aliasPublication,
            alias,
            holds,
            objectFailure,
            terrainFailure,
            restartPublication,
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
    useSidecar("sidecar.toml");
    await lifecycle("stop");
    useSidecar("sidecar.benchmark.toml");
    await lifecycle("stop");
}

if (!acceptance) fail("canonical runtime workflow did not produce acceptance evidence");
await Deno.writeTextFile(`${root}/${REPORT}`, `${JSON.stringify(acceptance, null, 2)}\n`);
console.log(JSON.stringify({ outcome: acceptance.outcome, report: REPORT }, null, 2));
