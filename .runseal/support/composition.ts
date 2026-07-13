import {
    distribution,
    fail,
    field,
    object,
    stableProbe as stableTerrainProbe,
    validateProbe as validateTerrainProbe,
} from "./terrain.ts";
import { validateProbe as validateSkeletalProbe } from "./skeletal-crowds.ts";

export const REVISION = "atomic-terrain-object-composition-v1";

export function sleep(milliseconds: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export function stableCompositionProbe(
    probe: Record<string, unknown>,
): Record<string, unknown> {
    const grounding = object(probe, "grounding");
    const skeletal = object(probe, "skeletal");
    const terrain = object(probe, "terrain");
    return {
        revision: probe.revision,
        grounding: {
            candidateCount: grounding.candidateCount,
            gpuSha256: grounding.gpuSha256,
            cpuSha256: grounding.cpuSha256,
            minimumNumerator: grounding.minimumNumerator,
            maximumNumerator: grounding.maximumNumerator,
            mismatchCount: grounding.mismatchCount,
            firstMismatch: grounding.firstMismatch,
            readbackBytes: grounding.readbackBytes,
            allocationBytes: grounding.allocationBytes,
            cullWriteCount: grounding.cullWriteCount,
            meshReadCount: grounding.meshReadCount,
        },
        terrain: stableTerrainProbe(terrain),
        skeletal: {
            config: skeletal.config,
            gpu: skeletal.gpu,
            cpuOracle: skeletal.cpuOracle,
            settings: skeletal.settings,
            meshletCatalogSha256: skeletal.meshletCatalogSha256,
            animationCatalogSha256: skeletal.animationCatalogSha256,
        },
        clearCount: probe.clearCount,
        fixedTerrainDispatches: probe.fixedTerrainDispatches,
        fixedSkeletalDispatches: probe.fixedSkeletalDispatches,
    };
}

export function validateCompositionProbe(
    probe: Record<string, unknown>,
    requireVisible = true,
): void {
    if (field<string>(probe, "revision", "string") !== REVISION) {
        fail("composition probe revision mismatch");
    }
    const grounding = object(probe, "grounding");
    if (
        grounding.candidateCount !== 25_600 || grounding.gpuSha256 !== grounding.cpuSha256 ||
        grounding.mismatchCount !== 0 || grounding.firstMismatch !== null ||
        grounding.readbackBytes !== 102_400 || grounding.allocationBytes !== 102_400 ||
        grounding.cullWriteCount !== 25_600 ||
        field<number>(grounding, "meshReadCount", "number") > 25_600 ||
        (requireVisible && field<number>(grounding, "meshReadCount", "number") <= 0)
    ) fail("composition grounding contract failed");
    if (
        probe.clearCount !== 1 || probe.fixedTerrainDispatches !== 3 ||
        probe.fixedSkeletalDispatches !== 5
    ) fail("composition fixed submission or clear ownership changed");
    validateTerrainProbe(object(probe, "terrain"));
    validateSkeletalProbe(object(probe, "skeletal"));
    const pair = object(probe, "pair");
    const published = object(pair, "published");
    if (
        field<number>(published, "physicalSlotDivergenceCount", "number") <= 0 ||
        !Array.isArray(published.logicalRegionIds) || published.logicalRegionIds.length !== 25 ||
        !Array.isArray(published.instanceSlots) || published.instanceSlots.length !== 25 ||
        !Array.isArray(published.terrainSlots) || published.terrainSlots.length !== 25 ||
        published.instanceMappingSha256 === published.terrainMappingSha256
    ) fail("composition did not prove independent physical mappings");
}

export function compositionTimings(
    samples: Record<string, unknown>[],
): Record<string, unknown> {
    const value = (sample: Record<string, unknown>, name: string) =>
        field<number>(object(sample, "timing"), name, "number");
    const dist = (name: string) => distribution(samples.map((sample) => value(sample, name)));
    return {
        sampleCount: samples.length,
        terrainTotalMs: dist("terrainTotalMs"),
        groundAndCullClassifyMs: dist("groundAndCullClassifyMs"),
        poseCompactMs: dist("poseCompactMs"),
        poseEvaluateMs: dist("poseEvaluateMs"),
        meshSkinMs: dist("meshSkinMs"),
        skeletalTotalMs: dist("skeletalTotalMs"),
        combinedGpuMs: dist("combinedGpuMs"),
    };
}
