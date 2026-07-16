import { event, fail, type Json, object, rejectedEvent, root, same } from "./canonical-runtime.ts";

const RECORD_COUNT = 1_024;
const RECORD_BYTES = 20;
const IDENTITY_BYTES = 4;
const PRESENTATION_BYTES = 16;
const HEADER_BYTES = 96;
const INDEX_ENTRY_BYTES = 64;
const REGION_BYTES = RECORD_COUNT * (RECORD_BYTES + IDENTITY_BYTES + PRESENTATION_BYTES);

function requireQueryRejection(value: Json, label: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("canonical_object_query_failed: ")
    ) fail(`${label} returned the wrong canonical-object-query rejection`);
}

function requireZeroRuntimeWork(value: Json, label: string): void {
    if (
        value.perQueryAllocationBytes !== 0 || value.sourceReadCount !== 0 ||
        value.gpuCopyCount !== 0 || value.gpuReadbackCount !== 0 ||
        value.fenceWaitCount !== 0 || value.synchronizationCount !== 0
    ) fail(`${label} performed work outside the committed CPU object snapshot`);
}

export async function unavailableObjectQueryGate(base: [number, number]): Promise<Json> {
    return await rejectedObjectQuery(base, 0, "pre-publication object query");
}

export async function rejectedObjectQuery(
    region: [number, number],
    localId: number,
    label: string,
): Promise<Json> {
    const rejected = await rejectedEvent("canonical.objects.query", {
        region_x: region[0],
        region_z: region[1],
        authored_local_id: localId,
    });
    requireQueryRejection(rejected, label);
    return rejected;
}

export async function objectQueryGates(
    source: string,
    base: [number, number],
    unavailable: Json,
): Promise<Json> {
    const invalidLocalId = await rejectedEvent("canonical.objects.query", {
        region_x: base[0],
        region_z: base[1],
        authored_local_id: RECORD_COUNT,
    });
    const outside = await rejectedEvent("canonical.objects.query", {
        region_x: base[0] + 3,
        region_z: base[1],
        authored_local_id: 0,
    });
    requireQueryRejection(invalidLocalId, "out-of-range authored local ID");
    requireQueryRejection(outside, "outside-window object query");
    const samples = await queryObjectSamples(source, base, [0, 511, 1_023]);
    return { unavailable, invalidLocalId, outside, samples };
}

export async function queryObjectSamples(
    source: string,
    region: [number, number],
    localIds: number[],
): Promise<Json[]> {
    const samples: Json[] = [];
    for (const localId of localIds) {
        samples.push(await queryObject(source, region, localId));
    }
    return samples;
}

export async function queryObject(
    source: string,
    region: [number, number],
    localId: number,
): Promise<Json> {
    const value = await event("canonical.objects.query", {
        region_x: region[0],
        region_z: region[1],
        authored_local_id: localId,
    });
    if (value.revision !== "exact-canonical-object-query-v1") {
        fail(`object query ${region[0]},${region[1]}:${localId} revision diverged`);
    }
    requireZeroRuntimeWork(value, `object query ${region[0]},${region[1]}:${localId}`);
    const expected = await readObjectOracle(source, region, localId);
    same(object(value, "object"), expected, `object query ${region[0]},${region[1]}:${localId}`);
    return value;
}

export function sameObjectQueries(actual: Json[], expected: Json[], label: string): void {
    same(
        actual.map((value) => object(value, "object")),
        expected.map((value) => object(value, "object")),
        label,
    );
}

async function readObjectOracle(
    source: string,
    region: [number, number],
    localId: number,
): Promise<Json> {
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
    return {
        authoredLocalId: localId,
        height: view.getFloat32(record + 12, true),
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
        region: { x: region[0], z: region[1] },
    };
}
