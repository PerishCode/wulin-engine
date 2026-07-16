import {
    assertObjectCopies,
    type Coord,
    event,
    fail,
    failedPair,
    frame,
    type Json,
    number,
    object,
    publish,
    target,
} from "../canonical-runtime.ts";
import {
    type ObjectNearestSample,
    objectNearestSamples,
    queryObjectNearest,
    queryObjectNearestSamples,
    sameObjectNearestContent,
    sameObjectNearestQueries,
} from "./nearest.ts";
import {
    canonicalObjectSource,
    resolvedObject,
    resolveObject,
    resolveObjectIdentity,
    resolveObjectSamples,
    sameObjectResolutionContent,
    sameObjectResolutions,
} from "./query.ts";

export async function resolveStaleObjectIdentity(
    resolved: Json,
    expectedOutcome: "source-replaced" | "outside-published-window",
    label: string,
): Promise<Json> {
    const canonicalObject = resolvedObject(resolved);
    const identity = object(canonicalObject, "identity");
    const region = object(identity, "region");
    const value = await resolveObjectIdentity(
        canonicalObjectSource(canonicalObject),
        [number(region, "x"), number(region, "z")],
        number(identity, "authoredLocalId"),
    );
    if (object(value, "resolution").outcome !== expectedOutcome) {
        fail(`${label} did not resolve as ${expectedOutcome}`);
    }
    return value;
}

export async function differentObjectSourceGates(
    objects: string,
    base: Coord,
    localIds: number[],
    referenceResolutions: Json[],
    nearestSamples: ObjectNearestSample[],
    referenceNearest: Json[],
): Promise<Json> {
    const staleIdentity = await resolveStaleObjectIdentity(
        referenceResolutions[0],
        "source-replaced",
        "prior identity after replacement publication",
    );
    const resolutions = await resolveObjectSamples(objects, base, localIds);
    sameObjectResolutionContent(
        resolutions,
        referenceResolutions,
        "physical object order A/B resolution content",
    );
    const nearest = await queryObjectNearestSamples(objects, nearestSamples);
    sameObjectNearestContent(
        nearest,
        referenceNearest,
        "physical object order A/B nearest content",
    );
    return { staleIdentity, resolutions, nearest };
}

export async function adjacentObjectGates(
    objects: string,
    base: Coord,
    nearestSamples: ObjectNearestSample[],
    referenceNearest: Json[],
): Promise<Json> {
    const retiredRegion: Coord = [base[0] - 2, base[1]];
    const admittedRegion: Coord = [base[0] + 3, base[1]];
    const oldBefore = await resolveObject(objects, retiredRegion, 0);
    const publication = await publish(target([base[0] + 1, base[1]]));
    assertObjectCopies(publication, 5, "adjacent publication");
    const oldAfter = await resolveStaleObjectIdentity(
        oldBefore,
        "outside-published-window",
        "retired adjacent-window object resolution",
    );
    const admitted = await resolveObject(objects, admittedRegion, 1_023);
    const nearest = await queryObjectNearestSamples(
        objects,
        nearestSamples,
        [base[0] + 1, base[1]],
    );
    sameObjectNearestQueries(nearest, referenceNearest, "adjacent-window nearest query");
    return {
        publication,
        resolution: {
            adjacentOldBefore: oldBefore,
            adjacentOldAfter: oldAfter,
            adjacentNew: admitted,
        },
        nearest,
    };
}

export async function objectFailureGates(
    collection: string,
    terrain: string,
    objects: string,
    corruptObjects: string,
    corruptTerrain: string,
    base: Coord,
): Promise<Json> {
    const beforeFrame = await frame("failure-before", collection, false, false);
    const publishedRegion: Coord = [base[0] + 5, base[1]];
    const objectBefore = await resolveObject(objects, publishedRegion, 1_023);
    const nearestSample = objectNearestSamples(publishedRegion)[0];
    const nearestBefore = await queryObjectNearest(objects, nearestSample, publishedRegion);

    await event("source.objects.open", { path: corruptObjects });
    const objectFailure = await failedPair(
        target([base[0] + 70, base[1]]),
        beforeFrame,
        collection,
        "object-corrupt",
        false,
    );
    const objectAfterObject = await resolveObject(objects, publishedRegion, 1_023);
    const nearestAfterObject = await queryObjectNearest(objects, nearestSample, publishedRegion);
    sameObjectResolutions(
        [objectAfterObject],
        [objectBefore],
        "object-corrupt rollback resolution",
    );
    sameObjectNearestQueries(
        [nearestAfterObject],
        [nearestBefore],
        "object-corrupt rollback nearest query",
    );

    await event("source.objects.open", { path: objects });
    await event("source.terrain.open", { path: corruptTerrain });
    const terrainFailure = await failedPair(
        target([base[0] + 75, base[1]]),
        beforeFrame,
        collection,
        "terrain-corrupt",
        false,
    );
    const objectAfterTerrain = await resolveObject(objects, publishedRegion, 1_023);
    const nearestAfterTerrain = await queryObjectNearest(objects, nearestSample, publishedRegion);
    sameObjectResolutions(
        [objectAfterTerrain],
        [objectBefore],
        "terrain-corrupt rollback object resolution",
    );
    sameObjectNearestQueries(
        [nearestAfterTerrain],
        [nearestBefore],
        "terrain-corrupt rollback nearest query",
    );
    await event("source.terrain.open", { path: terrain });

    return {
        objectFailure,
        terrainFailure,
        objectResolution: {
            before: objectBefore,
            afterObjectFailure: objectAfterObject,
            afterTerrainFailure: objectAfterTerrain,
        },
        objectNearest: {
            before: nearestBefore,
            afterObjectFailure: nearestAfterObject,
            afterTerrainFailure: nearestAfterTerrain,
        },
    };
}
