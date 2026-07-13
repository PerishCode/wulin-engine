import {
    ANIMATION_SHA256,
    array,
    fail,
    field,
    MESH_SHA256,
    object,
    type Settings as SkeletalSettings,
    settings as skeletalSettings,
    validateProbe as validateSkeletalProbe,
} from "./skeletal-crowds.ts";

export {
    ANIMATION_SHA256,
    array,
    collectEnvironment,
    distribution,
    fail,
    field,
    MESH_SHA256,
    object,
    same,
} from "./skeletal-crowds.ts";
export type { SkeletalSettings };
export { skeletalSettings };

export const REVISION = "gpu-surface-resolve-v1";
export const SURFACE_SHA256 = "e9715635b9e9f2a7dd0089c35db3cb3ccd6ae87fc2119cc548ed2f37a4996989";

export type SurfaceSettings = {
    material_count: number;
    mip_level: number;
};

export function surfaceSettings(overrides: Partial<SurfaceSettings> = {}): SurfaceSettings {
    return { material_count: 64, mip_level: 0, ...overrides };
}

export function loadConfig(
    x = 64,
    z = 64,
    radius = 2,
): Record<string, number> {
    return {
        world_region_side: 128,
        active_center_x: x,
        active_center_z: z,
        active_radius: radius,
    };
}

export function validateProbe(probe: Record<string, unknown>): void {
    if (
        field<string>(probe, "revision", "string") !== REVISION ||
        field<string>(probe, "surfaceCatalogSha256", "string") !== SURFACE_SHA256 ||
        field<number>(probe, "invalidPayloadCount", "number") !== 0 ||
        field<number>(probe, "visibilityDispatchCount", "number") !== 1 ||
        field<number>(probe, "resolveDispatchCount", "number") !== 1
    ) fail("surface probe identity or fixed submission mismatch");
    validateSkeletalProbe(object(probe, "skeletal"));
    const stats = object(probe, "stats");
    const total = field<number>(stats, "resolvedPixels", "number");
    const visible = field<number>(stats, "visiblePixels", "number");
    const background = field<number>(stats, "backgroundPixels", "number");
    if (total !== 921_600 || visible + background !== total || visible === 0) {
        fail("surface pixel coverage invariant failed");
    }
    const settings = object(probe, "settings");
    const materialCount = field<number>(settings, "materialCount", "number");
    if (field<number>(stats, "observedMaterialCount", "number") !== materialCount) {
        fail("surface material coverage differs from the configured count");
    }
    if (
        field<number>(probe, "maximumSampleChannelDelta", "number") >
            field<number>(probe, "sampleChannelTolerance", "number")
    ) fail("surface shade oracle exceeded tolerance");
    const samples = array(probe, "samples") as Record<string, unknown>[];
    if (samples.length !== 6) fail("surface probe did not return six samples");
    for (const sample of samples) {
        if (
            field<number>(sample, "rgba8", "number") !==
                field<number>(sample, "expectedRgba8", "number") ||
            field<number>(sample, "maximumChannelDelta", "number") > 2
        ) fail("surface sample differs from the CPU shade oracle");
    }
    const groups = array(probe, "resolveGroups");
    if (JSON.stringify(groups) !== JSON.stringify([160, 90, 1])) {
        fail("surface resolve dispatch dimensions changed");
    }
}

export function stableProbeEvidence(probe: Record<string, unknown>): Record<string, unknown> {
    const samples = (array(probe, "samples") as Record<string, unknown>[]).map((sample) => ({
        pixel: sample.pixel,
        payload: sample.payload,
        candidateIndex: sample.candidateIndex,
        primitiveIndex: sample.primitiveIndex,
        barycentrics: sample.barycentrics,
        stableKey: sample.stableKey,
        materialIndex: sample.materialIndex,
        mipLevel: sample.mipLevel,
        rgba8: sample.rgba8,
        expectedRgba8: sample.expectedRgba8,
        texel: sample.texel,
        expectedTexel: sample.expectedTexel,
        maximumChannelDelta: sample.maximumChannelDelta,
    }));
    return {
        visibilitySha256: probe.visibilitySha256,
        stats: probe.stats,
        samples,
        skeletalGpu: object(object(probe, "skeletal"), "gpu"),
    };
}

export function baselineSkeletalSettings(
    overrides: Partial<SkeletalSettings> = {},
): SkeletalSettings {
    return skeletalSettings(overrides);
}

export function validateCatalogs(probe: Record<string, unknown>): void {
    const skeletal = object(probe, "skeletal");
    if (
        field<string>(skeletal, "meshletCatalogSha256", "string") !== MESH_SHA256 ||
        field<string>(skeletal, "animationCatalogSha256", "string") !== ANIMATION_SHA256
    ) fail("accepted catalog hashes changed");
}
