import {
    halfReports,
    probe as compositionProbe,
    publishPair,
    validatePair,
    waitPair,
} from "./global-composition.ts";
import { stableLodCompositionProbe } from "./composition.ts";
import { projection as terrainProjection, stableProjection } from "./camera-relative-terrain.ts";
import { canonicalEvidence } from "./signed-terrain-storage.ts";
import { event, type GlobalConfig, sleep } from "./global-terrain.ts";
import { array, captureEvidence, fail, field, object, same } from "./terrain.ts";

const REVISION = "canonical-generated-object-v1";
const OBJECT_ID_BASE = 65_536;

export function canonicalObjects(
    probe: Record<string, unknown>,
): Record<string, unknown> {
    const value = object(probe, "canonicalObjects");
    if (
        value.revision !== REVISION || value.entryCount !== 25 ||
        value.semanticCollisionCount !== 0 || value.stableSeedCollisionCount !== 0 ||
        value.mismatchCount !== 0
    ) fail("canonical generated-object aggregate failed");
    field<string>(value, "sourceNamespace", "string");
    field<string>(value, "contentSha256", "string");
    field<string>(value, "stableSeedSha256", "string");
    if (value.payloadAuthority !== undefined) {
        const authority = object(value, "payloadAuthority");
        if (
            authority.revision !== "cooked-object-payload-authority-v1" ||
            authority.regionCount !== 25 || authority.recordCount !== 25_600 ||
            authority.copyCount !== 25 || authority.readbackBytes !== 512_000 ||
            authority.allocationBytes !== 524_288 || authority.chunkMismatchCount !== 0 ||
            authority.expectedIndexSha256 !== authority.observedIndexSha256
        ) fail("cooked object payload authority failed");
        field<string>(authority, "payloadSha256", "string");
        field<number>(authority, "probeCount", "number");
        field<number>(authority, "totalCopyCount", "number");
    }

    const terrain = object(probe, "terrain");
    const terrainEntries = array(terrainProjection(terrain), "entries");
    const semanticIds = new Set<number>();
    const stableSeeds = new Set<number>();
    for (const [index, raw] of array(value, "entries").entries()) {
        if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
            fail("canonical generated-object entry is invalid");
        }
        const entry = raw as Record<string, unknown>;
        const semantic = field<number>(entry, "semanticRegionId", "number");
        const stableSeed = field<number>(entry, "stableSeed", "number");
        if (
            entry.activeIndex !== index ||
            entry.objectId !== OBJECT_ID_BASE + semantic + 1
        ) fail("canonical generated-object semantic handle does not invert");
        const terrainRaw = terrainEntries[index];
        if (!terrainRaw || typeof terrainRaw !== "object" || Array.isArray(terrainRaw)) {
            fail("canonical terrain projection entry is invalid");
        }
        const terrainEntry = terrainRaw as Record<string, unknown>;
        same(entry.globalRegion, terrainEntry.globalRegion, "canonical object/terrain region");
        same(
            entry.renderOffsetRegions,
            terrainEntry.renderOffsetRegions,
            "canonical object/terrain projection",
        );
        if (entry.semanticRegionId !== terrainEntry.semanticRegionId) {
            fail("canonical object/terrain semantic regions diverged");
        }
        semanticIds.add(semantic);
        stableSeeds.add(stableSeed);
    }
    if (semanticIds.size !== 25 || stableSeeds.size !== 25) {
        fail("canonical generated-object active identities collide");
    }
    return value;
}

export function stableObjectEvidence(
    probe: Record<string, unknown>,
): Record<string, unknown> {
    const value = canonicalObjects(probe);
    return {
        revision: value.revision,
        sourceNamespace: value.sourceNamespace,
        contentSha256: value.contentSha256,
        stableSeedSha256: value.stableSeedSha256,
        entries: value.entries,
    };
}

export function stableFrame(
    probe: Record<string, unknown>,
    capture: Record<string, unknown>,
): Record<string, unknown> {
    const stable = stableLodCompositionProbe(probe);
    const terrain = object(probe, "terrain");
    const skeletal = object(stable, "skeletal");
    const { config: _localAlias, ...canonicalSkeletal } = skeletal;
    return {
        revision: stable.revision,
        order: probe.order,
        canonicalObjects: stableObjectEvidence(probe),
        grounding: stable.grounding,
        contact: stable.contact,
        terrain: {
            projection: stableProjection(terrain),
            content: canonicalEvidence(terrain),
            lod: terrain.lod,
        },
        skeletal: canonicalSkeletal,
        clearCount: stable.clearCount,
        fixedTerrainDispatches: stable.fixedTerrainDispatches,
        fixedSkeletalDispatches: stable.fixedSkeletalDispatches,
        capture,
    };
}

export async function probe(
    config: GlobalConfig,
    requireVisible = true,
): Promise<Record<string, unknown>> {
    const value = await compositionProbe(config, requireVisible);
    canonicalObjects(value);
    return value;
}

export async function capture(
    id: string,
    collection: string,
    probeValue: Record<string, unknown>,
): Promise<Record<string, unknown>> {
    const terrainById = semanticMap(
        array(terrainProjection(object(probeValue, "terrain")), "entries"),
        "terrain projection",
    );
    const objectsById = semanticMap(
        array(canonicalObjects(probeValue), "entries"),
        "canonical objects",
    );
    const raw = await event("perception.capture", {
        id,
        collection,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const evidence = captureEvidence(raw);
    const visible = array(
        object(object(object(raw, "perception"), "evidence"), "fullFrame"),
        "objects",
    );
    const terrainJoins = [];
    const objectJoins = [];
    for (const rawObject of visible) {
        if (!rawObject || typeof rawObject !== "object" || Array.isArray(rawObject)) {
            fail("canonical composition semantic object is invalid");
        }
        const semantic = rawObject as Record<string, unknown>;
        const id = field<number>(semantic, "id", "number");
        if (semantic.kind === "terrain-region") {
            const globalRegion = terrainById.get(id);
            if (!globalRegion) fail(`terrain semantic ID ${id} has no canonical inverse`);
            terrainJoins.push({ id, globalRegion });
        } else if (semantic.kind === "region-proxy") {
            const globalRegion = objectsById.get(id);
            if (!globalRegion) fail(`object semantic ID ${id} has no canonical inverse`);
            objectJoins.push({ id, globalRegion });
        }
    }
    if (terrainJoins.length === 0 || objectJoins.length === 0) {
        fail("canonical composition capture omitted a semantic class");
    }
    return {
        ...evidence,
        png: field<string>(object(raw, "image"), "pngSha256", "string"),
        terrainJoins,
        objectJoins,
    };
}

function semanticMap(
    entries: unknown[],
    label: string,
): Map<number, Record<string, unknown>> {
    const result = new Map<number, Record<string, unknown>>();
    for (const raw of entries) {
        if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
            fail(`${label} entry is invalid`);
        }
        const entry = raw as Record<string, unknown>;
        const id = field<number>(entry, "objectId", "number");
        if (result.has(id)) fail(`${label} object IDs collide`);
        result.set(id, object(entry, "globalRegion"));
    }
    return result;
}

export function expectHalfCounts(
    publication: Record<string, unknown>,
    half: "terrain" | "instance",
    expected: Record<string, number>,
): void {
    const report = object(object(publication, "halves"), half);
    for (const [name, count] of Object.entries(expected)) {
        if (field<number>(report, name, "number") !== count) {
            fail(`${half} transaction ${name} mismatch`);
        }
    }
    if (
        field<number>(report, "residentRegionCount", "number") > 50 ||
        field<number>(report, "protectedRegionCount", "number") > 25
    ) fail(`${half} transaction exceeded bounded ownership`);
}

export function expectPairMovement(
    publication: Record<string, unknown>,
    retained: number,
    uploaded: number,
    terrainBytes: number,
    objectBytes: number,
): void {
    for (const half of ["terrain", "instance"] as const) {
        expectHalfCounts(publication, half, {
            retainedRegionCount: retained,
            uploadedRegionCount: uploaded,
        });
    }
    const halves = object(publication, "halves");
    const terrain = object(halves, "terrain");
    const instance = object(halves, "instance");
    if (
        terrain.payloadBytes !== terrainBytes ||
        object(terrain, "io").payloadBytes !== terrainBytes ||
        instance.instanceBytes !== objectBytes
    ) fail("canonical pair transfer byte counts diverged");
}

export function retainedObjectEvidence(
    before: Record<string, unknown>,
    after: Record<string, unknown>,
): Record<string, unknown> {
    const previous = new Map<string, number>();
    for (const raw of array(canonicalObjects(before), "entries")) {
        const entry = raw as Record<string, unknown>;
        previous.set(JSON.stringify(entry.globalRegion), Number(entry.stableSeed));
    }
    let retainedCount = 0;
    let mismatchCount = 0;
    for (const raw of array(canonicalObjects(after), "entries")) {
        const entry = raw as Record<string, unknown>;
        const seed = previous.get(JSON.stringify(entry.globalRegion));
        if (seed === undefined) continue;
        retainedCount += 1;
        if (seed !== entry.stableSeed) mismatchCount += 1;
    }
    if (retainedCount !== 20 || mismatchCount !== 0) {
        fail("retained canonical object identities changed");
    }
    return { retainedCount, mismatchCount };
}

export async function hold(
    kind: "terrain-io" | "terrain-copy" | "object-copy",
    target: GlobalConfig,
    collection: string,
): Promise<Record<string, unknown>> {
    const beforeProbe = await probeFromPublished();
    const beforeCapture = await capture(`${kind}-before`, collection, beforeProbe);
    const gate = kind === "object-copy"
        ? "async.gate"
        : kind === "terrain-io"
        ? "terrain.io_gate"
        : "terrain.copy_gate";
    await event(`${gate}.arm`);
    const scheduled = await event("composition.global.schedule", target);
    const token = field<number>(scheduled, "token", "number");
    await event("workbench.resume");
    const deadline = Date.now() + 10_000;
    let heldStatus: Record<string, unknown> | undefined;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        if (status.pending) {
            const pending = object(status, "pending");
            const reached = kind === "object-copy"
                ? pending.terrainStage === "staged" && pending.instanceStage === "in-flight"
                : pending.instanceStage === "staged" && pending.terrainStage === "in-flight";
            if (reached) {
                heldStatus = status;
                break;
            }
        }
        await sleep(10);
    }
    if (!heldStatus) fail(`${kind} did not reach one-half-ready state`);
    await event("workbench.pause");
    const heldProbe = await probeFromPublished();
    const heldCapture = await capture(`${kind}-held`, collection, heldProbe);
    same(
        stableFrame(heldProbe, heldCapture),
        stableFrame(beforeProbe, beforeCapture),
        `${kind} complete old frame`,
    );
    await event(`${gate}.release`);
    await event("workbench.resume");
    const status = await waitPair(token);
    await event("workbench.pause");
    const published = object(status, "published");
    validatePair(published, target);
    return {
        scheduled,
        heldStatus,
        beforeProbe,
        beforeCapture,
        heldProbe,
        heldCapture,
        published,
        halves: await halfReports(published, target),
    };
}

export async function waitPairFailure(token: number): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        if (status.pending === null && status.lastFailure) {
            const failure = object(status, "lastFailure");
            if (failure.token === token) return status;
        }
        await sleep(10);
    }
    fail(`canonical composition token ${token} failure timed out`);
}

export async function probeFromPublished(): Promise<Record<string, unknown>> {
    const status = await event("composition.status");
    const pair = object(status, "published");
    const config = object(pair, "globalConfig");
    return await probe({
        origin_x: field<number>(object(config, "globalOrigin"), "x", "number"),
        origin_z: field<number>(object(config, "globalOrigin"), "z", "number"),
        center_x: field<number>(object(config, "globalCenter"), "x", "number"),
        center_z: field<number>(object(config, "globalCenter"), "z", "number"),
        active_radius: field<number>(config, "activeRadius", "number"),
    });
}

export { publishPair };
