import {
    event,
    fail,
    type Json,
    number,
    object,
    rejectedEvent,
    root,
    same,
} from "../canonical-runtime.ts";

const RECORD_COUNT = 1_024;
const RECORD_BYTES = 20;
const IDENTITY_BYTES = 4;
const PRESENTATION_BYTES = 16;
const HEADER_BYTES = 96;
const INDEX_ENTRY_BYTES = 64;
const REGION_BYTES = RECORD_COUNT * (RECORD_BYTES + IDENTITY_BYTES + PRESENTATION_BYTES);
const ZERO_SOURCE_NAMESPACE = "00".repeat(32);
export const OBJECT_RESOLUTION_SAMPLE_IDS = [0, 31, 511, 992, 1_023];

type SourceIndex = {
    bytes: Uint8Array;
    view: DataView;
    sourceNamespace: string;
};

const sourceIndexes = new Map<string, SourceIndex>();

function requireResolutionFailure(value: Json, label: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("canonical_object_resolution_failed: ")
    ) fail(`${label} returned the wrong canonical-object-resolution failure`);
}

function requireZeroResolutionWork(value: Json, label: string): void {
    if (
        value.perResolutionAllocationBytes !== 0 || value.sourceReadCount !== 0 ||
        value.gpuCopyCount !== 0 || value.gpuReadbackCount !== 0 ||
        value.fenceWaitCount !== 0 || value.synchronizationCount !== 0
    ) fail(`${label} performed work outside the committed CPU object snapshot`);
}

function requireResolutionOutcome(value: Json, expected: string, label: string): void {
    const resolution = object(value, "resolution");
    if (resolution.outcome !== expected) {
        fail(`${label} returned resolution outcome ${JSON.stringify(resolution.outcome)}`);
    }
    if (expected !== "resolved" && value.terrainPosition !== null) {
        fail(`${label} exposed a terrain position for a non-resolved identity`);
    }
}

export function resolvedObject(value: Json): Json {
    requireResolutionOutcome(value, "resolved", "resolved canonical object");
    return object(object(value, "resolution"), "object");
}

export function canonicalObjectSnapshot(value: Json): Json {
    const snapshot = object(value, "snapshot");
    const publicationToken = number(snapshot, "publicationToken");
    if (!Number.isSafeInteger(publicationToken) || publicationToken < 1) {
        fail("canonical object snapshot has an invalid live publication token");
    }
    if (
        typeof snapshot.sourceNamespace !== "string" ||
        !/^[0-9a-f]{64}$/.test(snapshot.sourceNamespace)
    ) fail("canonical object snapshot has an invalid source namespace");
    return snapshot;
}

export async function unavailableObjectResolutionGate(base: [number, number]): Promise<Json> {
    return await failedObjectResolution(
        ZERO_SOURCE_NAMESPACE,
        base,
        0,
        "pre-publication object resolution",
    );
}

export async function failedObjectResolution(
    sourceNamespace: string,
    region: [number, number],
    localId: number,
    label: string,
): Promise<Json> {
    const rejected = await rejectedEvent("canonical.objects.resolve", {
        source_namespace: sourceNamespace,
        region_x: region[0],
        region_z: region[1],
        authored_local_id: localId,
    });
    requireResolutionFailure(rejected, label);
    return rejected;
}

export async function resolveObjectIdentity(
    sourceNamespace: string,
    region: [number, number],
    localId: number,
): Promise<Json> {
    const value = await event("canonical.objects.resolve", {
        source_namespace: sourceNamespace,
        region_x: region[0],
        region_z: region[1],
        authored_local_id: localId,
    });
    if (value.revision !== "versioned-canonical-object-resolution-v2") {
        fail(`object identity resolution ${region[0]},${region[1]}:${localId} revision diverged`);
    }
    canonicalObjectSnapshot(value);
    requireZeroResolutionWork(
        value,
        `object identity resolution ${region[0]},${region[1]}:${localId}`,
    );
    return value;
}

export async function objectResolutionGates(
    source: string,
    base: [number, number],
    unavailable: Json,
    publication: Json,
): Promise<Json> {
    const sourceNamespace = await objectSourceNamespace(source);
    const invalidLocalId = await rejectedEvent("canonical.objects.resolve", {
        source_namespace: sourceNamespace,
        region_x: base[0],
        region_z: base[1],
        authored_local_id: RECORD_COUNT,
    });
    const outside = await resolveObjectIdentity(sourceNamespace, [base[0] + 3, base[1]], 0);
    requireResolutionFailure(invalidLocalId, "out-of-range authored local ID");
    requireResolutionOutcome(
        outside,
        "outside-published-window",
        "outside-window object resolution",
    );
    const sourceMismatch = await resolveObjectIdentity(
        ZERO_SOURCE_NAMESPACE,
        base,
        0,
    );
    requireResolutionOutcome(sourceMismatch, "source-replaced", "object source replacement");
    if (canonicalObjectSnapshot(sourceMismatch).sourceNamespace === ZERO_SOURCE_NAMESPACE) {
        fail("source-replaced resolution exposed the stale source as the current snapshot");
    }
    const unqualified = await rejectedEvent("canonical.objects.resolve", {
        region_x: base[0],
        region_z: base[1],
        authored_local_id: 0,
    });
    if (
        typeof unqualified.error !== "string" ||
        !unqualified.error.startsWith("invalid_payload: ")
    ) {
        fail("unqualified canonical object resolution was not rejected at the current schema");
    }
    const samples = await resolveObjectSamples(source, base, OBJECT_RESOLUTION_SAMPLE_IDS);
    const published = object(publication, "published");
    const snapshot = canonicalObjectSnapshot(samples[0]);
    if (
        number(snapshot, "publicationToken") !== number(published, "token") ||
        snapshot.sourceNamespace !== published.objectSourceNamespace
    ) fail("canonical object snapshot diverged from the published composition pair");
    return {
        unavailable,
        invalidLocalId,
        outside,
        sourceMismatch,
        unqualified,
        samples,
        snapshot,
    };
}

export async function resolveObjectSamples(
    source: string,
    region: [number, number],
    localIds: number[],
): Promise<Json[]> {
    const samples: Json[] = [];
    for (const localId of localIds) {
        samples.push(await resolveObject(source, region, localId));
    }
    return samples;
}

export async function resolveObject(
    source: string,
    region: [number, number],
    localId: number,
): Promise<Json> {
    const sourceNamespace = await objectSourceNamespace(source);
    const value = await resolveObjectIdentity(sourceNamespace, region, localId);
    requireResolutionOutcome(
        value,
        "resolved",
        `object resolution ${region[0]},${region[1]}:${localId}`,
    );
    const expected = await readObjectOracle(source, region, localId);
    if (canonicalObjectSnapshot(value).sourceNamespace !== sourceNamespace) {
        fail(`object resolution ${region[0]},${region[1]}:${localId} snapshot source diverged`);
    }
    same(
        resolvedObject(value),
        object(expected, "object"),
        `object resolution ${region[0]},${region[1]}:${localId}`,
    );
    same(
        object(value, "terrainPosition"),
        object(expected, "terrainPosition"),
        `object terrain position ${region[0]},${region[1]}:${localId}`,
    );
    return value;
}

export function sameObjectResolutions(actual: Json[], expected: Json[], label: string): void {
    same(
        actual.map(resolvedObject),
        expected.map(resolvedObject),
        label,
    );
}

export function sameObjectResolutionContent(
    actual: Json[],
    expected: Json[],
    label: string,
): void {
    same(
        actual.map((value) => canonicalObjectContent(resolvedObject(value))),
        expected.map((value) => canonicalObjectContent(resolvedObject(value))),
        label,
    );
    const actualSources = actual.map((value) => canonicalObjectSource(resolvedObject(value)));
    const expectedSources = expected.map((value) => canonicalObjectSource(resolvedObject(value)));
    if (actualSources.some((source, index) => source === expectedSources[index])) {
        fail(`${label} did not change every source-qualified identity`);
    }
}

export function canonicalObjectContent(value: Json): Json {
    const identity = object(value, "identity");
    return {
        identity: {
            region: object(identity, "region"),
            authoredLocalId: identity.authoredLocalId,
        },
        position: value.position,
        height: value.height,
        presentation: value.presentation,
    };
}

export function canonicalObjectSource(value: Json): string {
    const source = object(value, "identity").sourceNamespace;
    if (typeof source !== "string") fail("canonical object identity has no source namespace");
    return source;
}

export async function objectSourceNamespace(source: string): Promise<string> {
    return (await sourceIndex(source)).sourceNamespace;
}

async function sourceIndex(source: string): Promise<SourceIndex> {
    const cached = sourceIndexes.get(source);
    if (cached) return cached;
    const bytes = await Deno.readFile(`${root}/${source}`);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (
        new TextDecoder().decode(bytes.subarray(0, 8)) !== "WLRGN003" ||
        view.getUint32(8, true) !== 3 || view.getUint32(12, true) !== HEADER_BYTES ||
        view.getUint32(20, true) !== INDEX_ENTRY_BYTES ||
        view.getUint32(24, true) !== RECORD_COUNT || view.getUint32(28, true) !== RECORD_BYTES ||
        view.getUint32(56, true) !== 3
    ) fail("independent object query oracle rejected the schema-3 header");
    const regionCount = view.getUint32(16, true);
    const namespaceBytes = bytes.slice(0, HEADER_BYTES + regionCount * INDEX_ENTRY_BYTES);
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", namespaceBytes));
    const sourceNamespace = Array.from(digest, (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
    const parsed = { bytes, view, sourceNamespace };
    sourceIndexes.set(source, parsed);
    return parsed;
}

async function readObjectOracle(
    source: string,
    region: [number, number],
    localId: number,
): Promise<Json> {
    const { view, sourceNamespace } = await sourceIndex(source);
    const regionCount = view.getUint32(16, true);
    let payloadOffset: number | undefined;
    for (let index = 0; index < regionCount; index += 1) {
        const entry = HEADER_BYTES + index * INDEX_ENTRY_BYTES;
        const x = Number(view.getBigInt64(entry, true));
        const z = Number(view.getBigInt64(entry + 8, true));
        if (x !== region[0] || z !== region[1]) continue;
        if (view.getUint32(entry + 24, true) !== REGION_BYTES) {
            fail("independent object query oracle found a noncanonical region size");
        }
        payloadOffset = Number(view.getBigUint64(entry + 16, true));
        break;
    }
    if (payloadOffset === undefined) {
        fail(`independent object query oracle did not find region ${region[0]},${region[1]}`);
    }
    const identityOffset = payloadOffset + RECORD_COUNT * RECORD_BYTES;
    let physicalIndex: number | undefined;
    for (let index = 0; index < RECORD_COUNT; index += 1) {
        if (view.getUint32(identityOffset + index * IDENTITY_BYTES, true) === localId) {
            if (physicalIndex !== undefined) {
                fail("independent object query oracle found a duplicate authored local ID");
            }
            physicalIndex = index;
        }
    }
    if (physicalIndex === undefined) {
        fail(`independent object query oracle did not find local ID ${localId}`);
    }
    const record = payloadOffset + physicalIndex * RECORD_BYTES;
    const presentation = identityOffset + RECORD_COUNT * IDENTITY_BYTES +
        physicalIndex * PRESENTATION_BYTES;
    const authored = {
        height: view.getFloat32(record + 12, true),
        identity: {
            authoredLocalId: localId,
            region: { x: region[0], z: region[1] },
            sourceNamespace,
        },
        position: [
            view.getFloat32(record, true),
            view.getFloat32(record + 4, true),
            view.getFloat32(record + 8, true),
        ],
        presentation: {
            animation: view.getUint32(presentation + 12, true),
            archetype: view.getUint32(presentation, true),
            material: view.getUint32(presentation + 4, true),
            yawQ16: view.getUint32(presentation + 8, true),
        },
    };
    return {
        object: authored,
        terrainPosition: exactTerrainPosition(region, authored.position),
    };
}

function exactTerrainPosition(region: [number, number], position: number[]): Json {
    const axis = (value: number, name: string): { regionDelta: number; localQ9: number } => {
        const scaled = value * 512;
        if (!Number.isFinite(scaled) || !Number.isInteger(scaled)) {
            fail(`independent object position oracle rejected non-Q9 ${name}`);
        }
        if (scaled < -4_096 || scaled > 4_096) {
            fail(`independent object position oracle rejected out-of-region ${name}`);
        }
        return scaled === 4_096
            ? { regionDelta: 1, localQ9: -4_096 }
            : { regionDelta: 0, localQ9: scaled };
    };
    const x = axis(position[0], "X");
    const z = axis(position[2], "Z");
    return {
        localXQ9: x.localQ9,
        localZQ9: z.localQ9,
        region: { x: region[0] + x.regionDelta, z: region[1] + z.regionDelta },
    };
}
