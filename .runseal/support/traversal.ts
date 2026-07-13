import { stableLodCompositionProbe } from "./composition.ts";
import { fail, field, object } from "./terrain.ts";

export const TRAVERSAL_REVISION = "camera-region-traversal-v1";

export function configMatches(value: unknown, x: number, z: number): boolean {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    const config = value as Record<string, unknown>;
    return config.worldRegionSide === 128 && config.activeCenterX === x &&
        config.activeCenterZ === z && config.activeRadius === 2;
}

export function traversal(status: Record<string, unknown>): Record<string, unknown> {
    const value = object(status, "traversal");
    if (
        value.revision !== TRAVERSAL_REVISION ||
        field<number>(value, "maxQueuedDepth", "number") > 1
    ) fail("camera traversal status violated its bounded contract");
    return value;
}

export function worldCenter(region: number): number {
    return (region - 64) * 16;
}

export function logicalEvidence(probe: Record<string, unknown>): Record<string, unknown> {
    const stable = stableLodCompositionProbe(probe);
    const terrain = object(stable, "terrain");
    const { activeMapping: _mapping, activeMappingSha256: _hash, ...logicalTerrain } = terrain;
    return { ...stable, terrain: logicalTerrain };
}
