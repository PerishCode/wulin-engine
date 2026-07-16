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
import {
    canonicalObjectContent,
    canonicalObjectSnapshot,
    canonicalObjectSource,
    objectSourceNamespace,
} from "./query.ts";

const RECORD_COUNT = 1_024;
const RECORD_BYTES = 20;
const IDENTITY_BYTES = 4;
const PRESENTATION_BYTES = 16;
const HEADER_BYTES = 96;
const INDEX_ENTRY_BYTES = 64;
const REGION_BYTES = RECORD_COUNT * (RECORD_BYTES + IDENTITY_BYTES + PRESENTATION_BYTES);
const ACTIVE_RADIUS = 2;
const MAXIMUM_CANDIDATE_COUNT = 25 * RECORD_COUNT;

export type ObjectNearestSample = {
    region: [number, number];
    localXQ9: number;
    localZQ9: number;
    maxDistanceQ9: number;
};

type SourceIndex = {
    bytes: Uint8Array;
    view: DataView;
    regions: Map<string, number>;
};

const sourceIndexes = new Map<string, SourceIndex>();

function request(sample: ObjectNearestSample): Json {
    return {
        region_x: sample.region[0],
        region_z: sample.region[1],
        local_x_q9: sample.localXQ9,
        local_z_q9: sample.localZQ9,
        max_distance_q9: sample.maxDistanceQ9,
    };
}

function requireNearestRejection(value: Json, label: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("canonical_object_nearest_failed: ")
    ) fail(`${label} returned the wrong canonical-object-nearest rejection`);
}

function requireZeroRuntimeWork(value: Json, label: string): void {
    if (
        number(value, "maximumCandidateCount") !== MAXIMUM_CANDIDATE_COUNT ||
        value.perQueryAllocationBytes !== 0 || value.sourceReadCount !== 0 ||
        value.gpuCopyCount !== 0 || value.gpuReadbackCount !== 0 ||
        value.fenceWaitCount !== 0 || value.synchronizationCount !== 0
    ) fail(`${label} performed work outside the committed CPU object snapshot`);
}

export function objectNearestSamples(base: [number, number]): ObjectNearestSample[] {
    return [
        { region: base, localXQ9: -4_096, localZQ9: -4_096, maxDistanceQ9: 0 },
        { region: base, localXQ9: -4_095, localZQ9: -4_096, maxDistanceQ9: 1 },
        { region: base, localXQ9: -4_095, localZQ9: -4_096, maxDistanceQ9: 0 },
        { region: base, localXQ9: 0, localZQ9: 0, maxDistanceQ9: 512 },
        { region: base, localXQ9: 0, localZQ9: 0, maxDistanceQ9: 4_294_967_295 },
    ];
}

export async function unavailableObjectNearestGate(base: [number, number]): Promise<Json> {
    const rejected = await rejectedEvent(
        "canonical.objects.nearest",
        request(objectNearestSamples(base)[0]),
    );
    requireNearestRejection(rejected, "pre-publication object nearest");
    return rejected;
}

export async function objectNearestGates(
    source: string,
    base: [number, number],
    unavailable: Json,
): Promise<Json> {
    const malformedOrigin = await rejectedEvent("canonical.objects.nearest", {
        ...request(objectNearestSamples(base)[0]),
        local_x_q9: 4_096,
    });
    const outsideOrigin = await rejectedEvent("canonical.objects.nearest", {
        ...request(objectNearestSamples(base)[0]),
        region_x: base[0] + ACTIVE_RADIUS + 1,
    });
    requireNearestRejection(malformedOrigin, "malformed object-nearest origin");
    requireNearestRejection(outsideOrigin, "outside-window object-nearest origin");
    const samples = await queryObjectNearestSamples(source, objectNearestSamples(base));

    const seam = object(object(samples[0], "query"), "nearest");
    const seamIdentity = object(object(seam, "object"), "identity");
    if (
        number(object(seamIdentity, "region"), "x") !== base[0] - 1 ||
        number(object(seamIdentity, "region"), "z") !== base[1] - 1 ||
        number(seamIdentity, "authoredLocalId") !== 1_023 ||
        number(seam, "distanceSquaredQ18") !== 0
    ) fail("zero-radius shared-seam tie order diverged");
    if (object(samples[2], "query").nearest !== null) {
        fail("zero-radius displaced object-nearest query returned a candidate");
    }
    return { unavailable, malformedOrigin, outsideOrigin, samples };
}

export async function queryObjectNearestSamples(
    source: string,
    samples: ObjectNearestSample[],
    windowCenter?: [number, number],
): Promise<Json[]> {
    const values: Json[] = [];
    for (const sample of samples) {
        values.push(await queryObjectNearest(source, sample, windowCenter));
    }
    return values;
}

export async function queryObjectNearest(
    source: string,
    sample: ObjectNearestSample,
    windowCenter: [number, number] = sample.region,
): Promise<Json> {
    const value = await event("canonical.objects.nearest", request(sample));
    if (value.revision !== "versioned-canonical-object-nearest-v2") {
        fail(`object nearest ${sample.region.join(",")} revision diverged`);
    }
    requireZeroRuntimeWork(value, `object nearest ${sample.region.join(",")}`);
    const sourceNamespace = await objectSourceNamespace(source);
    if (canonicalObjectSnapshot(value).sourceNamespace !== sourceNamespace) {
        fail(`object nearest ${sample.region.join(",")} snapshot source diverged`);
    }
    const query = object(value, "query");
    if (number(query, "candidateCount") !== MAXIMUM_CANDIDATE_COUNT) {
        fail("object nearest did not scan the exact committed candidate bound");
    }
    same(
        query,
        await objectNearestOracle(source, sample, windowCenter),
        `object nearest ${sample.region.join(",")}:${sample.localXQ9},${sample.localZQ9}`,
    );
    return value;
}

export function sameObjectNearestQueries(actual: Json[], expected: Json[], label: string): void {
    same(
        actual.map((value) => object(value, "query")),
        expected.map((value) => object(value, "query")),
        label,
    );
}

export function sameObjectNearestContent(
    actual: Json[],
    expected: Json[],
    label: string,
): void {
    same(
        actual.map(nearestContent),
        expected.map(nearestContent),
        label,
    );
    const actualSources = actual.map(nearestSource);
    const expectedSources = expected.map(nearestSource);
    if (
        actualSources.some((source, index) => source !== null && source === expectedSources[index])
    ) {
        fail(`${label} did not change every nearest source-qualified identity`);
    }
}

function nearestContent(value: Json): Json {
    const query = object(value, "query");
    if (query.nearest === null) return query;
    const nearest = object(query, "nearest");
    return {
        candidateCount: query.candidateCount,
        nearest: {
            deltaXQ9: nearest.deltaXQ9,
            deltaZQ9: nearest.deltaZQ9,
            distanceSquaredQ18: nearest.distanceSquaredQ18,
            object: canonicalObjectContent(object(nearest, "object")),
            terrainPosition: nearest.terrainPosition,
        },
    };
}

function nearestSource(value: Json): string | null {
    const query = object(value, "query");
    return query.nearest === null
        ? null
        : canonicalObjectSource(object(object(query, "nearest"), "object"));
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
    ) fail("independent object-nearest oracle rejected the schema-3 header");
    const regions = new Map<string, number>();
    const regionCount = view.getUint32(16, true);
    for (let index = 0; index < regionCount; index += 1) {
        const entry = HEADER_BYTES + index * INDEX_ENTRY_BYTES;
        const x = Number(view.getBigInt64(entry, true));
        const z = Number(view.getBigInt64(entry + 8, true));
        if (!Number.isSafeInteger(x) || !Number.isSafeInteger(z)) {
            fail("independent object-nearest oracle requires exact JSON-safe signed regions");
        }
        if (view.getUint32(entry + 24, true) !== REGION_BYTES) {
            fail("independent object-nearest oracle found a noncanonical region size");
        }
        const key = `${x},${z}`;
        if (regions.has(key)) fail("independent object-nearest oracle found a duplicate region");
        regions.set(key, Number(view.getBigUint64(entry + 16, true)));
    }
    const parsed = { bytes, view, regions };
    sourceIndexes.set(source, parsed);
    return parsed;
}

export async function objectNearestOracle(
    source: string,
    sample: ObjectNearestSample,
    windowCenter: [number, number],
): Promise<Json> {
    const { view, regions } = await sourceIndex(source);
    const sourceNamespace = await objectSourceNamespace(source);
    const radius = BigInt(sample.maxDistanceQ9);
    const radiusSquared = radius * radius;
    let candidateCount = 0;
    let nearest:
        | { key: [bigint, number, number, number]; value: Json }
        | undefined;

    for (
        let regionZ = windowCenter[1] - ACTIVE_RADIUS;
        regionZ <= windowCenter[1] + ACTIVE_RADIUS;
        regionZ++
    ) {
        for (
            let regionX = windowCenter[0] - ACTIVE_RADIUS;
            regionX <= windowCenter[0] + ACTIVE_RADIUS;
            regionX++
        ) {
            const payloadOffset = regions.get(`${regionX},${regionZ}`);
            if (payloadOffset === undefined) {
                fail(`independent object-nearest oracle did not find region ${regionX},${regionZ}`);
            }
            const identityOffset = payloadOffset + RECORD_COUNT * RECORD_BYTES;
            const presentationOffset = identityOffset + RECORD_COUNT * IDENTITY_BYTES;
            const seen = new Set<number>();
            for (let physical = 0; physical < RECORD_COUNT; physical++) {
                const authoredLocalId = view.getUint32(
                    identityOffset + physical * IDENTITY_BYTES,
                    true,
                );
                if (authoredLocalId >= RECORD_COUNT || seen.has(authoredLocalId)) {
                    fail("independent object-nearest oracle found an invalid authored ID plane");
                }
                seen.add(authoredLocalId);
                candidateCount += 1;
                const record = payloadOffset + physical * RECORD_BYTES;
                const presentation = presentationOffset + physical * PRESENTATION_BYTES;
                const position = [
                    view.getFloat32(record, true),
                    view.getFloat32(record + 4, true),
                    view.getFloat32(record + 8, true),
                ];
                const terrainPosition = exactTerrainPosition([regionX, regionZ], position);
                const deltaX = axisDelta(
                    sample.region[0],
                    sample.localXQ9,
                    number(object(terrainPosition, "region"), "x"),
                    number(terrainPosition, "localXQ9"),
                );
                const deltaZ = axisDelta(
                    sample.region[1],
                    sample.localZQ9,
                    number(object(terrainPosition, "region"), "z"),
                    number(terrainPosition, "localZQ9"),
                );
                if (deltaX < -radius || deltaX > radius || deltaZ < -radius || deltaZ > radius) {
                    continue;
                }
                const distanceSquared = deltaX * deltaX + deltaZ * deltaZ;
                if (distanceSquared > radiusSquared) continue;
                const key: [bigint, number, number, number] = [
                    distanceSquared,
                    regionX,
                    regionZ,
                    authoredLocalId,
                ];
                if (nearest && compareKey(key, nearest.key) >= 0) continue;
                const distance = Number(distanceSquared);
                if (!Number.isSafeInteger(distance)) {
                    fail("independent object-nearest oracle produced an unsafe JSON distance");
                }
                nearest = {
                    key,
                    value: {
                        deltaXQ9: Number(deltaX),
                        deltaZQ9: Number(deltaZ),
                        distanceSquaredQ18: distance,
                        object: {
                            height: view.getFloat32(record + 12, true),
                            identity: {
                                authoredLocalId,
                                region: { x: regionX, z: regionZ },
                                sourceNamespace,
                            },
                            position,
                            presentation: {
                                animation: view.getUint32(presentation + 12, true),
                                archetype: view.getUint32(presentation, true),
                                material: view.getUint32(presentation + 4, true),
                                yawQ16: view.getUint32(presentation + 8, true),
                            },
                        },
                        terrainPosition,
                    },
                };
            }
            if (seen.size !== RECORD_COUNT) {
                fail("independent object-nearest oracle found an incomplete authored ID plane");
            }
        }
    }
    if (candidateCount !== MAXIMUM_CANDIDATE_COUNT) {
        fail("independent object-nearest oracle scanned the wrong candidate count");
    }
    return { candidateCount, nearest: nearest?.value ?? null };
}

function axisDelta(
    originRegion: number,
    originLocalQ9: number,
    candidateRegion: number,
    candidateLocalQ9: number,
): bigint {
    return (BigInt(candidateRegion) - BigInt(originRegion)) * 8_192n +
        BigInt(candidateLocalQ9 - originLocalQ9);
}

function compareKey(
    left: [bigint, number, number, number],
    right: [bigint, number, number, number],
): number {
    if (left[0] !== right[0]) return left[0] < right[0] ? -1 : 1;
    for (let index = 1; index < left.length; index++) {
        if (left[index] !== right[index]) return left[index] < right[index] ? -1 : 1;
    }
    return 0;
}

function exactTerrainPosition(region: [number, number], position: number[]): Json {
    const axis = (value: number, name: string): { regionDelta: number; localQ9: number } => {
        const scaled = value * 512;
        if (!Number.isFinite(scaled) || !Number.isInteger(scaled)) {
            fail(`independent object-nearest oracle rejected non-Q9 ${name}`);
        }
        if (scaled < -4_096 || scaled > 4_096) {
            fail(`independent object-nearest oracle rejected out-of-region ${name}`);
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
