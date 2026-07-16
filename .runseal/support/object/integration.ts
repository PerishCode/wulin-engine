import {
    assertObjectCopies,
    type Coord,
    event,
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
    objectSourceNamespace,
    queryObject,
    queryObjectSamples,
    rejectedObjectQuery,
    sameObjectQueries,
    sameObjectQueryContent,
} from "./query.ts";

export async function rejectStaleObjectIdentity(
    query: Json,
    label: string,
): Promise<Json> {
    const identity = object(object(query, "object"), "identity");
    const region = object(identity, "region");
    return await rejectedObjectQuery(
        canonicalObjectSource(object(query, "object")),
        [number(region, "x"), number(region, "z")],
        number(identity, "authoredLocalId"),
        label,
    );
}

export async function differentObjectSourceGates(
    objects: string,
    base: Coord,
    localIds: number[],
    referenceQueries: Json[],
    nearestSamples: ObjectNearestSample[],
    referenceNearest: Json[],
): Promise<Json> {
    const staleIdentity = await rejectStaleObjectIdentity(
        referenceQueries[0],
        "prior identity after replacement publication",
    );
    const queries = await queryObjectSamples(objects, base, localIds);
    sameObjectQueryContent(queries, referenceQueries, "physical object order A/B query content");
    const nearest = await queryObjectNearestSamples(objects, nearestSamples);
    sameObjectNearestContent(
        nearest,
        referenceNearest,
        "physical object order A/B nearest content",
    );
    return { staleIdentity, queries, nearest };
}

export async function adjacentObjectGates(
    objects: string,
    base: Coord,
    nearestSamples: ObjectNearestSample[],
    referenceNearest: Json[],
): Promise<Json> {
    const retiredRegion: Coord = [base[0] - 2, base[1]];
    const admittedRegion: Coord = [base[0] + 3, base[1]];
    const oldBefore = await queryObject(objects, retiredRegion, 0);
    const publication = await publish(target([base[0] + 1, base[1]]));
    assertObjectCopies(publication, 5, "adjacent publication");
    const oldAfter = await rejectedObjectQuery(
        await objectSourceNamespace(objects),
        retiredRegion,
        0,
        "retired adjacent-window object query",
    );
    const admitted = await queryObject(objects, admittedRegion, 1_023);
    const nearest = await queryObjectNearestSamples(
        objects,
        nearestSamples,
        [base[0] + 1, base[1]],
    );
    sameObjectNearestQueries(nearest, referenceNearest, "adjacent-window nearest query");
    return {
        publication,
        query: {
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
    const objectBefore = await queryObject(objects, publishedRegion, 1_023);
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
    const objectAfterObject = await queryObject(objects, publishedRegion, 1_023);
    const nearestAfterObject = await queryObjectNearest(objects, nearestSample, publishedRegion);
    sameObjectQueries([objectAfterObject], [objectBefore], "object-corrupt rollback query");
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
    const objectAfterTerrain = await queryObject(objects, publishedRegion, 1_023);
    const nearestAfterTerrain = await queryObjectNearest(objects, nearestSample, publishedRegion);
    sameObjectQueries(
        [objectAfterTerrain],
        [objectBefore],
        "terrain-corrupt rollback object query",
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
        objectQuery: {
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
