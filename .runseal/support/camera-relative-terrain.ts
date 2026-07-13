import {
    type Coord,
    event,
    type GlobalConfig,
    globalConfig,
    sleep,
    validateGlobalProbe,
    waitPublished,
} from "./global-terrain.ts";
import { canonicalEvidence, captureJoined } from "./signed-terrain-storage.ts";
import { array, fail, field, object, same, validateLodProbe, validateProbe } from "./terrain.ts";

const CENTER = 64;
const REGION_METERS = 16;
const TERRAIN_OBJECT_ID_BASE = 32_768;

export function aliasConfig(
    globalCenter: Coord,
    localCenterX: number,
    localCenterZ = CENTER,
): GlobalConfig {
    return globalConfig(
        [
            globalCenter[0] - (localCenterX - CENTER),
            globalCenter[1] - (localCenterZ - CENTER),
        ],
        globalCenter,
    );
}

export async function setAliasCamera(centerX: number, centerZ = CENTER): Promise<void> {
    const offsetX = (centerX - CENTER) * REGION_METERS;
    const offsetZ = (centerZ - CENTER) * REGION_METERS;
    await event("camera.set_pose", {
        position: [9 + offsetX, 6, 12 + offsetZ],
        target: [offsetX, 1, -3 + offsetZ],
        vertical_fov_degrees: 60,
    });
}

export function projection(probe: Record<string, unknown>): Record<string, unknown> {
    const value = object(probe, "canonicalProjection");
    if (
        value.revision !== "camera-relative-terrain-v1" ||
        value.entryCount !== 25 ||
        value.semanticCollisionCount !== 0 ||
        value.mismatchCount !== 0 ||
        array(value, "localRegionIds").length !== 25
    ) fail("canonical terrain projection aggregate failed");
    const semanticIds = new Set<number>();
    for (const [index, raw] of array(value, "entries").entries()) {
        if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
            fail("canonical terrain projection entry is invalid");
        }
        const entry = raw as Record<string, unknown>;
        const semantic = field<number>(entry, "semanticRegionId", "number");
        const objectId = field<number>(entry, "objectId", "number");
        const offset = array(entry, "renderOffsetRegions").map(Number);
        if (
            entry.activeIndex !== index ||
            objectId !== TERRAIN_OBJECT_ID_BASE + semantic + 1 ||
            semantic % 128 - CENTER !== offset[0] ||
            Math.floor(semantic / 128) - CENTER !== offset[1]
        ) fail("canonical terrain projection entry does not invert");
        object(entry, "globalRegion");
        semanticIds.add(semantic);
    }
    if (semanticIds.size !== 25) fail("canonical terrain semantic IDs collide");
    field<string>(value, "viewProjectionSha256", "string");
    field<string>(value, "projectionSha256", "string");
    return value;
}

export function stableProjection(probe: Record<string, unknown>): Record<string, unknown> {
    const value = projection(probe);
    return {
        revision: value.revision,
        projectedCamera: value.projectedCamera,
        viewProjectionSha256: value.viewProjectionSha256,
        projectionSha256: value.projectionSha256,
        entries: value.entries,
    };
}

export function projectionShape(probe: Record<string, unknown>): Record<string, unknown> {
    const value = projection(probe);
    return {
        projectedCamera: value.projectedCamera,
        viewProjectionSha256: value.viewProjectionSha256,
        semantics: array(value, "entries").map((raw) => {
            const entry = raw as Record<string, unknown>;
            return {
                activeIndex: entry.activeIndex,
                semanticRegionId: entry.semanticRegionId,
                objectId: entry.objectId,
                renderOffsetRegions: entry.renderOffsetRegions,
            };
        }),
    };
}

export function aliasEvidence(probe: Record<string, unknown>): Record<string, unknown> {
    const value = projection(probe);
    return {
        aliasCenter: value.aliasCenter,
        aliasOffsetRegions: value.aliasOffsetRegions,
        aliasOffsetMeters: value.aliasOffsetMeters,
        localRegionIds: value.localRegionIds,
    };
}

export async function projectionProbe(
    config: GlobalConfig,
    lodEnabled = false,
): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    if (lodEnabled) validateLodProbe(value, true);
    else validateProbe(value);
    validateGlobalProbe(value, config);
    projection(value);
    return value;
}

export function stableFrame(
    probe: Record<string, unknown>,
    capture: Record<string, unknown>,
): Record<string, unknown> {
    return {
        projection: stableProjection(probe),
        terrain: canonicalEvidence(probe),
        lod: probe.lod,
        capture,
    };
}

export async function projectionHold(
    kind: "io" | "copy",
    current: GlobalConfig,
    target: GlobalConfig,
    collection: string,
): Promise<Record<string, unknown>> {
    const beforeProbe = await projectionProbe(current);
    const beforeCapture = await captureJoined(`${kind}-before`, collection, beforeProbe);
    await event(`terrain.${kind}_gate.arm`);
    const scheduled = await event("terrain.global.schedule", target);
    const transactionId = field<number>(scheduled, "transactionId", "number");
    await event("workbench.resume");
    const deadline = Date.now() + 10_000;
    let heldStatus: Record<string, unknown> | undefined;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const reached = kind === "io"
            ? object(status, "stream").pending !== null
            : object(object(status, "renderer"), "transfer").copyPending !== null;
        if (reached) {
            heldStatus = status;
            break;
        }
        await sleep(10);
    }
    if (!heldStatus) fail(`${kind} projection gate did not hold`);
    await event("workbench.pause");
    const heldProbe = await projectionProbe(current);
    const heldCapture = await captureJoined(`${kind}-held`, collection, heldProbe);
    same(
        stableFrame(heldProbe, heldCapture),
        stableFrame(beforeProbe, beforeCapture),
        `${kind} held frame`,
    );
    await event(`terrain.${kind}_gate.release`);
    await event("workbench.resume");
    const published = await waitPublished(transactionId);
    await event("workbench.pause");
    return { scheduled, beforeProbe, beforeCapture, heldProbe, heldCapture, heldStatus, published };
}
