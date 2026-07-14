import {
    canonicalObjects,
    capture,
    expectHalfCounts,
    probe,
    stableFrame,
} from "../canonical-object-composition.ts";
import { publication, target, traversal, waitPublished } from "../canonical-origin-rollover.ts";
import {
    prefetch,
    type PrefetchContext,
    setPosition,
    setupPrefetch,
    waitPrefetchFailure,
} from "../canonical-traversal-prefetch.ts";
import { cookObjects, objectStatus } from "../cooked-canonical-objects/mod.ts";
import { type GlobalConfig, halfReports, prepare } from "../global-composition.ts";
import { event, sleep } from "../global-terrain.ts";
import { fail, field, object } from "../terrain.ts";

const AUTHORITY_REVISION = "canonical-authored-object-authority-q9-v1";

export async function cookAuthority(
    path: string,
    centers: [number, number][],
): Promise<Record<string, unknown>> {
    const report = await cookObjects(path, centers, true);
    if (report.sourceRevision !== AUTHORITY_REVISION) {
        fail("authority cooker revision diverged");
    }
    return report;
}

export function payloadAuthority(
    probeValue: Record<string, unknown>,
): Record<string, unknown> {
    const authority = object(canonicalObjects(probeValue), "payloadAuthority");
    const grounding = object(probeValue, "grounding");
    const probeCount = field<number>(authority, "probeCount", "number");
    const totalCopyCount = field<number>(authority, "totalCopyCount", "number");
    if (
        grounding.instanceReadbackBytes !== 512_000 ||
        grounding.instanceReadbackAllocationBytes !== 524_288 ||
        grounding.instanceReadbackCopyCount !== 25 ||
        probeCount * 25 !== totalCopyCount
    ) fail("authority probe readback accounting diverged");
    return authority;
}

export function requireNoPayloadAuthority(probeValue: Record<string, unknown>): void {
    if (canonicalObjects(probeValue).payloadAuthority !== undefined) {
        fail("generated object probe exposed cooked payload authority");
    }
}

export async function authorityProbe(
    config: GlobalConfig,
    requireVisible = true,
): Promise<Record<string, unknown>> {
    const value = await probe(config, requireVisible);
    payloadAuthority(value);
    return value;
}

export function expectReadback(
    status: Record<string, unknown>,
    probeCount: number,
): Record<string, unknown> {
    const value = object(status, "payloadReadback");
    if (
        value.resourceCount !== 1 || value.capacityPages !== 25 ||
        value.capacityBytes !== 512_000 || value.allocationBytes !== 524_288 ||
        value.probeCount !== probeCount || value.copyCount !== probeCount * 25
    ) fail(`active payload readback accounting diverged at probe ${probeCount}`);
    return value;
}

export async function asyncStatus(): Promise<Record<string, unknown>> {
    return await event("async.status");
}

export async function sourceEvidence(
    terrainPack: string,
    config: GlobalConfig,
    collection: string,
    label: string,
    objectPack?: string,
): Promise<Record<string, unknown>> {
    const publication = await prepare(terrainPack, config, objectPack);
    const beforeProbe = expectReadback(await asyncStatus(), 0);
    const probeValue = await probe(config);
    if (objectPack) payloadAuthority(probeValue);
    else requireNoPayloadAuthority(probeValue);
    const afterProbe = expectReadback(await asyncStatus(), 1);
    const captureValue = await capture(label, collection, probeValue);
    const afterCapture = expectReadback(await asyncStatus(), 1);
    const pair = object(publication, "published");
    return {
        publication,
        probe: probeValue,
        capture: captureValue,
        halves: await halfReports(pair, config),
        frame: stableFrame(probeValue, captureValue),
        readback: { beforeProbe, afterProbe, afterCapture },
    };
}

export function expectSweepReadback(
    sweep: Record<string, unknown>,
    probeCount: number,
): Record<string, unknown> {
    return expectReadback({ payloadReadback: object(sweep, "payloadReadback") }, probeCount);
}

export async function objectFailure(
    label: string,
    failingPack: string,
    context: PrefetchContext,
): Promise<Record<string, unknown>> {
    const objectPack = context.objectPack;
    if (!objectPack) fail(`${label} authority failure has no object source`);
    const start: [number, number] = [context.base[0] + 70, context.base[1]];
    const config = target(start);
    await setupPrefetch(context.pack, config, [0, 0], "sidecar.toml", objectPack);
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
    ) fail(`${label} authority failure mutated demand state`);
    const heldProbe = await authorityProbe(config);
    const failedObjects = await objectStatus();
    if (!failedObjects.lastFailure) fail(`${label} authority failure omitted diagnostics`);
    await sleep(100);
    const stable = await event("composition.status");
    if (prefetch(stable).failureCount !== 1) {
        fail(`${label} authority failure retried without motion`);
    }
    await event("objects.open", { path: objectPack });
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
    const recoveredProbe = await authorityProbe(expected);
    return { failed, failedObjects, heldProbe, stable, published, report, recoveredProbe };
}
