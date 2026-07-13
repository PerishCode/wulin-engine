import { array, fail, field, object, same } from "./surface-resolve.ts";

export {
    array,
    collectEnvironment,
    distribution,
    fail,
    field,
    object,
    same,
} from "./surface-resolve.ts";

export const TERRAIN_REVISION = "gpu-streamed-terrain-v1";
export const TERRAIN_LOD_REVISION = "gpu-terrain-lod-v1";
export const CANONICAL_MAPPING = "42adea7d457e6094829661910fb22122b8069ff56570f22d94129970df47c449";
export const CANONICAL_PAYLOAD = "5353840d77c05d7e7e0e17232e06a5cc2bc2461b86b25ba32c3f2e9c5774c460";

export function loadConfig(x = 64, z = 64, radius = 2): Record<string, number> {
    return {
        world_region_side: 128,
        active_center_x: x,
        active_center_z: z,
        active_radius: radius,
    };
}

export function stableProbe(probe: Record<string, unknown>): Record<string, unknown> {
    return {
        revision: probe.revision,
        config: probe.config,
        activeMapping: probe.activeMapping,
        activeMappingSha256: probe.activeMappingSha256,
        payloadSha256: probe.payloadSha256,
        cpuEdges: probe.cpuEdges,
        gpuEdges: probe.gpuEdges,
        geometry: probe.geometry,
        submission: probe.submission,
        resources: probe.resources,
    };
}

export function validateProbe(probe: Record<string, unknown>): void {
    if (field<string>(probe, "revision", "string") !== TERRAIN_REVISION) {
        fail("terrain probe revision mismatch");
    }
    const config = object(probe, "config");
    const radius = field<number>(config, "activeRadius", "number");
    const side = radius * 2 + 1;
    const active = side * side;
    const patches = active * 16;
    const edges = 2 * side * (side - 1);
    const cpu = object(probe, "cpuEdges");
    const gpu = object(probe, "gpuEdges");
    for (const value of [cpu, gpu]) {
        if (
            field<number>(value, "neighborEdges", "number") !== edges ||
            field<number>(value, "sampleComparisons", "number") !== edges * 33 ||
            field<number>(value, "mismatchCount", "number") !== 0 ||
            value.firstMismatch !== null
        ) fail("terrain shared-edge oracle mismatch");
    }
    const geometry = object(probe, "geometry");
    if (
        field<number>(geometry, "fixedPatchGroups", "number") !== 400 ||
        field<number>(geometry, "emittedPatches", "number") !== patches ||
        field<number>(geometry, "inactiveGroups", "number") !== 400 - patches ||
        field<number>(geometry, "vertices", "number") !== patches * 81 ||
        field<number>(geometry, "triangles", "number") !== patches * 128 ||
        geometry.vertices !== geometry.oracleVertices ||
        geometry.triangles !== geometry.oracleTriangles ||
        geometry.emittedPatches !== geometry.oraclePatches
    ) fail("terrain GPU geometry aggregates diverged from the oracle");
    same(object(probe, "submission"), {
        meshDispatchCount: 1,
        meshDispatchGroups: [400, 1, 1],
        seamDispatchCount: 1,
        seamDispatchGroups: [25, 2, 1],
    }, "terrain fixed submission");
    same(object(probe, "resources"), {
        cacheCapacity: 50,
        activeCapacity: 25,
        payloadBytes: 4096,
        statsBytes: 32,
        seamBytes: 32,
    }, "terrain bounded resources");
    if (array(probe, "activeMapping").length !== active) {
        fail("terrain active mapping length mismatch");
    }
}

export function captureEvidence(capture: Record<string, unknown>): Record<string, unknown> {
    if (capture.lastError !== null || object(capture, "renderer").deviceRemovedReason !== null) {
        fail("terrain capture reported a renderer failure");
    }
    const perception = object(capture, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) {
        fail("terrain perception contains unknown semantic IDs");
    }
    return {
        color: field<string>(object(capture, "image"), "pixelSha256", "string"),
        objectId: field<string>(perception, "rawSha256", "string"),
        diagnostic: field<string>(perception, "diagnosticPngSha256", "string"),
        centerSample: array(evidence, "samples")[1],
        visibleObjects: array(object(evidence, "fullFrame"), "objects").length,
    };
}

export function validateLodProbe(probe: Record<string, unknown>, enabled: boolean): void {
    if (field<string>(probe, "revision", "string") !== TERRAIN_REVISION) {
        fail("terrain LOD parent revision mismatch");
    }
    const lod = object(probe, "lod");
    const oracle = object(lod, "oracle");
    const gpu = object(lod, "gpu");
    if (field<string>(oracle, "revision", "string") !== TERRAIN_LOD_REVISION) {
        fail("terrain LOD oracle revision mismatch");
    }
    const settings = object(oracle, "settings");
    if (field<boolean>(settings, "enabled", "boolean") !== enabled) {
        fail("terrain LOD probe enable state mismatch");
    }
    same(array(gpu, "lodCounts"), array(oracle, "lodCounts"), "terrain LOD counts");
    const geometry = object(probe, "geometry");
    const oracleGeometry = object(oracle, "geometry");
    for (
        const [actual, expected] of [
            [geometry.emittedPatches, oracleGeometry.patches],
            [geometry.vertices, oracleGeometry.vertices],
            [geometry.triangles, oracleGeometry.triangles],
        ]
    ) {
        if (actual !== expected) fail("terrain LOD geometry diverged from CPU oracle");
    }
    if (array(gpu, "lodCounts").map(Number).reduce((sum, value) => sum + value, 0) !== 400) {
        fail("terrain LOD did not classify all patches");
    }
    same(object(probe, "submission"), {
        meshDispatchCount: 1,
        meshDispatchGroups: [400, 1, 1],
        seamDispatchCount: 1,
        seamDispatchGroups: [25, 2, 1],
    }, "terrain parent submission");
    same(object(probe, "resources"), {
        cacheCapacity: 50,
        activeCapacity: 25,
        payloadBytes: 4096,
        statsBytes: 32,
        seamBytes: 32,
    }, "terrain parent resources");
    same(object(lod, "submission"), {
        dispatchCount: enabled ? 1 : 0,
        dispatchGroups: [400, 2, 1],
    }, "terrain LOD submission");
    same(object(lod, "resources"), { statsBytes: 64 }, "terrain LOD resources");
    if (!enabled) return;
    const edges = object(oracle, "edges");
    for (
        const name of [
            "patchEdges",
            "sameLodEdges",
            "transitionEdges",
            "adjustedVertices",
            "sampleComparisons",
            "maxLodDelta",
            "mismatchCount",
        ]
    ) {
        if (gpu[name] !== edges[name]) fail(`terrain LOD edge field ${name} diverged`);
    }
    if (
        edges.patchEdges !== 760 || edges.sampleComparisons !== 6_840 ||
        field<number>(edges, "maxLodDelta", "number") > 1 ||
        edges.mismatchCount !== 0 || edges.firstMismatch !== null || gpu.firstMismatch !== null
    ) fail("terrain LOD edge oracle failed");
}

export function stableLodProbe(probe: Record<string, unknown>): Record<string, unknown> {
    return {
        ...stableProbe(probe),
        lod: probe.lod,
    };
}
