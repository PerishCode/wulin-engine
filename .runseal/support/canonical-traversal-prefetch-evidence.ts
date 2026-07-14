import { capture, probe } from "./canonical-object-composition.ts";
import { capability, objectTimings } from "./canonical-origin-rollover-evidence.ts";
import {
    publication,
    startClean,
    target,
    traversal,
    waitPublished,
} from "./canonical-origin-rollover.ts";
import {
    completionReports,
    expectDemandCounts,
    expectPreparedCounts,
    prefetch,
    type PrefetchContext,
    setPosition,
    setupPrefetch,
    waitPrefetchCompletion,
} from "./canonical-traversal-prefetch.ts";
import { compositionTimings } from "./composition.ts";
import { prepare } from "./global-composition.ts";
import { event, lifecycle, sleep, transactionDistributions } from "./global-terrain.ts";
import { distribution, fail, field, object } from "./terrain.ts";

export async function unpreparedSweep(
    context: PrefetchContext,
): Promise<Record<string, unknown>> {
    await startClean("sidecar.benchmark.toml");
    const status = await event("workbench.status");
    const processId = field<number>(status, "processId", "number");
    const renderer = capability(status, false);
    await prepare(context.pack, target(context.base), context.objectPack);
    await setPosition([0, 0]);
    await event("composition.traversal.enable");
    await event("workbench.resume");
    await sleep(30);
    const terrain: Record<string, unknown>[] = [];
    const objects: Record<string, unknown>[] = [];
    const pairPublication: number[] = [];
    const probes: Record<string, unknown>[] = [];
    const objectIo: Record<string, unknown>[] = [];
    let finalStatus: Record<string, unknown> | undefined;
    for (let offset = 1; offset <= 32; offset++) {
        const centerMeters = (offset - 1) * 16;
        await setPosition([centerMeters + 5, 0]);
        await sleep(2);
        await setPosition([centerMeters + 9, 0]);
        const expected = target([context.base[0] + offset, context.base[1]], 64 + offset);
        finalStatus = await waitPublished(expected, offset);
        const report = await publication(finalStatus, expected);
        expectDemandCounts(report, 20, 5);
        const halves = object(report, "halves");
        terrain.push(object(halves, "terrain"));
        objects.push(object(halves, "instance"));
        pairPublication.push(
            field<number>(object(report, "published"), "publicationMs", "number"),
        );
        probes.push(await probe(expected));
        if (context.objectPack) {
            objectIo.push(object(await event("objects.status"), "lastCompleted"));
        }
    }
    await event("workbench.pause");
    if (!finalStatus || "prefetch" in traversal(finalStatus)) {
        fail("control sweep exposed prefetch status");
    }
    const frame = await capture("release-control", context.collection, probes.at(-1)!);
    const payloadReadback = object(await event("async.status"), "payloadReadback");
    await lifecycle("stop");
    return {
        processId,
        renderer,
        sampleCount: 32,
        pairPublicationMs: distribution(pairPublication),
        terrain: transactionDistributions(terrain),
        objects: objectTimings(objects),
        composition: compositionTimings(probes),
        cookedObjectIo: context.objectPack ? objectIoDistributions(objectIo) : undefined,
        payloadReadback,
        frame,
    };
}

export async function preparedSweep(
    context: PrefetchContext,
): Promise<Record<string, unknown>> {
    await setupPrefetch(
        context.pack,
        target(context.base),
        [0, 0],
        "sidecar.benchmark.toml",
        context.objectPack,
    );
    const status = await event("workbench.status");
    const processId = field<number>(status, "processId", "number");
    const renderer = capability(status, false);
    const preparedTerrain: Record<string, unknown>[] = [];
    const preparedObjects: Record<string, unknown>[] = [];
    const preparation: number[] = [];
    const demandTerrain: Record<string, unknown>[] = [];
    const demandObjects: Record<string, unknown>[] = [];
    const pairPublication: number[] = [];
    const probes: Record<string, unknown>[] = [];
    const objectIo: Record<string, unknown>[] = [];
    let finalStatus: Record<string, unknown> | undefined;
    for (let offset = 1; offset <= 32; offset++) {
        const centerMeters = (offset - 1) * 16;
        const expected = target([context.base[0] + offset, context.base[1]], 64 + offset);
        await setPosition([centerMeters + 5, 0]);
        const completed = await waitPrefetchCompletion(expected, offset, "release prefetch");
        const prepared = expectPreparedCounts(completed, 20, 5);
        if (context.objectPack) {
            objectIo.push(object(await event("objects.status"), "lastCompleted"));
        }
        preparedTerrain.push(prepared.terrain);
        preparedObjects.push(prepared.objects);
        preparation.push(field<number>(prepared.completion, "preparationMs", "number"));
        await setPosition([centerMeters + 9, 0]);
        finalStatus = await waitPublished(expected, offset);
        const report = await publication(finalStatus, expected);
        expectDemandCounts(report, 25, 0);
        const halves = object(report, "halves");
        demandTerrain.push(object(halves, "terrain"));
        demandObjects.push(object(halves, "instance"));
        pairPublication.push(
            field<number>(object(report, "published"), "publicationMs", "number"),
        );
        probes.push(await probe(expected));
    }
    await event("workbench.pause");
    if (!finalStatus || prefetch(finalStatus).completionCount !== 32) {
        fail("prepared sweep completion count diverged");
    }
    const frame = await capture("release-prepared", context.collection, probes.at(-1)!);
    const completion = completionReports(finalStatus);
    const payloadReadback = object(await event("async.status"), "payloadReadback");
    await lifecycle("stop");
    return {
        processId,
        renderer,
        sampleCount: 32,
        preparationMs: distribution(preparation),
        prepared: {
            terrain: transactionDistributions(preparedTerrain),
            objects: objectTimings(preparedObjects),
        },
        demand: {
            pairPublicationMs: distribution(pairPublication),
            terrain: transactionDistributions(demandTerrain),
            objects: objectTimings(demandObjects),
        },
        composition: compositionTimings(probes),
        cookedObjectIo: context.objectPack ? objectIoDistributions(objectIo) : undefined,
        payloadReadback,
        finalCompletion: completion,
        frame,
    };
}

function objectIoDistributions(samples: Record<string, unknown>[]) {
    const values = (name: string) =>
        samples.map((sample) => field<number>(object(sample, "io"), name, "number"));
    return {
        sampleCount: samples.length,
        chunkCount: distribution(values("chunkCount"), "object chunk count", true),
        payloadBytes: distribution(values("payloadBytes"), "object payload bytes", true),
        totalMs: distribution(values("totalMs"), "object I/O time", true),
        readMs: distribution(values("readMs"), "object read time", true),
        verifyMs: distribution(values("verifyMs"), "object verify time", true),
    };
}
