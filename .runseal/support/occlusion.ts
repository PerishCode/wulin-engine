import {
    array,
    fail,
    field,
    object,
    same,
    stableProbeEvidence,
    validateCatalogs,
    validateProbe as validateSurfaceProbe,
} from "./surface-resolve.ts";

export {
    ANIMATION_SHA256,
    array,
    baselineSkeletalSettings,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    MESH_SHA256,
    object,
    same,
    stableProbeEvidence,
    SURFACE_SHA256,
    surfaceSettings,
    validateCatalogs,
} from "./surface-resolve.ts";
export type { SkeletalSettings, SurfaceSettings } from "./surface-resolve.ts";

export const OCCLUSION_REVISION = "gpu-conservative-occlusion-v1";
export const HIGH_OCCLUSION_CAMERA = {
    position: [0, 2, 20],
    target: [0, 1, -20],
    vertical_fov_degrees: 60,
};

export function assertStableQueried(
    evidence: Record<string, unknown>,
    minimumMeshletPercent: number,
): void {
    if (!field<boolean>(evidence, "historyQueried", "boolean")) {
        fail("optimized occlusion sample did not query compatible history");
    }
    const source = field<number>(evidence, "sourceMeshlets", "number");
    const submitted = field<number>(evidence, "submittedMeshlets", "number");
    const eliminatedPercent = (source - submitted) * 100 / source;
    if (eliminatedPercent < minimumMeshletPercent) {
        fail(
            `optimized occlusion eliminated ${eliminatedPercent}% meshlets, expected ${minimumMeshletPercent}%`,
        );
    }
}

export function validateProbe(
    probe: Record<string, unknown>,
    requireFullMaterialCoverage = true,
): void {
    validateSurfaceProbe(probe, requireFullMaterialCoverage);
    validateCatalogs(probe);
    const occlusion = object(probe, "occlusion");
    if (
        field<number>(occlusion, "hierarchyMismatchCount", "number") !== 0 ||
        field<number>(occlusion, "invalidQueries", "number") !== 0 ||
        field<number>(occlusion, "overflow", "number") !== 0 ||
        field<number>(occlusion, "queryDispatchCount", "number") !== 1 ||
        field<number>(occlusion, "queryGroups", "number") !== 100 ||
        field<number>(occlusion, "prefixDispatchCount", "number") !== 1 ||
        field<number>(occlusion, "prefixGroups", "number") !== 1 ||
        field<number>(occlusion, "scatterDispatchCount", "number") !== 1 ||
        field<number>(occlusion, "scatterGroups", "number") !== 100 ||
        field<number>(occlusion, "compactionDispatchCount", "number") !== 3 ||
        field<number>(occlusion, "hierarchyDispatchCount", "number") !== 11 ||
        field<string>(occlusion, "hierarchyFormat", "string") !== "R32_UINT" ||
        field<number>(occlusion, "hierarchyBytes", "number") !== 4_915_052
    ) fail("occlusion hierarchy or fixed submission contract mismatch");
    if (field<number>(occlusion, "stableCompactionMismatchCount", "number") !== 0) {
        fail("occlusion compaction changed source-visible order");
    }
    same(array(occlusion, "hierarchyMipDimensions"), [
        [1280, 720],
        [640, 360],
        [320, 180],
        [160, 90],
        [80, 45],
        [40, 22],
        [20, 11],
        [10, 5],
        [5, 2],
        [2, 1],
        [1, 1],
    ], "occlusion hierarchy dimensions");
    const source = field<number>(occlusion, "sourceVisible", "number");
    const survivors = field<number>(occlusion, "survivors", "number");
    const occluded = field<number>(occlusion, "occluded", "number");
    if (survivors + occluded !== source) fail("occlusion object accounting diverged");
    const oracle = object(occlusion, "cpuOracle");
    for (
        const name of [
            "sourceVisible",
            "survivors",
            "occluded",
            "sourceMeshlets",
            "submittedMeshlets",
            "sourceVertices",
            "submittedVertices",
            "sourceTriangles",
            "submittedTriangles",
            "sourceSkinInfluences",
            "submittedSkinInfluences",
        ]
    ) {
        if (field<number>(occlusion, name, "number") !== field<number>(oracle, name, "number")) {
            fail(`occlusion GPU and CPU ${name} differ`);
        }
    }
    const proof = object(occlusion, "boundProof");
    if (
        field<number>(proof, "testedVertexPoses", "number") !== 7_667_712 ||
        field<number>(proof, "minimumRadialSlack", "number") < 0 ||
        field<number>(proof, "minimumVerticalPad", "number") >
            field<number>(proof, "verticalPad", "number") ||
        field<number>(proof, "maximumVerticalPad", "number") >
            field<number>(proof, "verticalPad", "number")
    ) fail("occlusion fixture bound proof failed");
}

export function stableOcclusionEvidence(probe: Record<string, unknown>): Record<string, unknown> {
    const occlusion = object(probe, "occlusion");
    return {
        surface: stableProbeEvidence(probe),
        historyQueried: occlusion.historyQueried,
        bypassReason: occlusion.bypassReason,
        sourceVisible: occlusion.sourceVisible,
        survivors: occlusion.survivors,
        occluded: occlusion.occluded,
        sourceMeshlets: occlusion.sourceMeshlets,
        submittedMeshlets: occlusion.submittedMeshlets,
        sourceVertices: occlusion.sourceVertices,
        submittedVertices: occlusion.submittedVertices,
        sourceTriangles: occlusion.sourceTriangles,
        submittedTriangles: occlusion.submittedTriangles,
        sourceSkinInfluences: occlusion.sourceSkinInfluences,
        submittedSkinInfluences: occlusion.submittedSkinInfluences,
        candidateMaskSha256: occlusion.candidateMaskSha256,
        stableCompactionMismatchCount: occlusion.stableCompactionMismatchCount,
        hierarchySha256: occlusion.hierarchySha256,
        hierarchyMismatchCount: occlusion.hierarchyMismatchCount,
        cpuOracle: occlusion.cpuOracle,
        boundProof: occlusion.boundProof,
    };
}
