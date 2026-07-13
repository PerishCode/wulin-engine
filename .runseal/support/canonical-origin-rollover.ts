import { aliasConfig } from "./camera-relative-terrain.ts";
import { type GlobalConfig, halfReports, localConfig } from "./global-composition.ts";
import { event, lifecycle, sleep, useSidecar } from "./global-terrain.ts";
import { fail, field, object, same } from "./terrain.ts";
import { traversal as traversalState } from "./traversal.ts";

export type Coord = [number, number];

const REVISION = "canonical-origin-rollover-v1";
const REGION_METERS = 16;

export async function startClean(config: string): Promise<void> {
    useSidecar(config);
    await lifecycle("stop");
    await lifecycle("start");
}

export function target(
    center: Coord,
    localCenterX = 64,
    localCenterZ = 64,
): GlobalConfig {
    return aliasConfig(center, localCenterX, localCenterZ);
}

export function targetMatches(value: unknown, expected: GlobalConfig): boolean {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    const owner = value as Record<string, unknown>;
    if (!configMatches(owner.config, localConfig(expected))) return false;
    if (!owner.globalConfig || typeof owner.globalConfig !== "object") return false;
    const global = owner.globalConfig as Record<string, unknown>;
    const origin = object(global, "globalOrigin");
    const center = object(global, "globalCenter");
    return origin.x === expected.origin_x && origin.z === expected.origin_z &&
        center.x === expected.center_x && center.z === expected.center_z &&
        global.activeRadius === expected.active_radius;
}

export function traversal(status: Record<string, unknown>): Record<string, unknown> {
    const value = traversalState(status);
    const rollover = object(value, "rollover");
    if (
        rollover.revision !== REVISION || rollover.minimumLocalCenter !== 32 ||
        rollover.maximumLocalCenter !== 96 || rollover.recenterLocalCenter !== 64
    ) fail("canonical rollover policy changed");
    const basis = object(value, "basis");
    const policy = object(basis, "rollover");
    if (
        policy.minimumLocalCenter !== 32 || policy.maximumLocalCenter !== 96 ||
        policy.recenterLocalCenter !== 64 || basis.worldRegionSide !== 128 ||
        basis.activeRadius !== 2
    ) fail("canonical traversal basis changed");
    const published = object(status, "published");
    const publishedOrigin = object(object(published, "globalConfig"), "globalOrigin");
    same(object(basis, "globalOrigin"), publishedOrigin, "rollover basis/published origin");
    return value;
}

export function rollover(status: Record<string, unknown>): Record<string, unknown> {
    return object(traversal(status), "rollover");
}

export async function waitStatus(
    label: string,
    predicate: (status: Record<string, unknown>) => boolean,
): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        traversal(status);
        if (predicate(status)) return status;
        await sleep(10);
    }
    fail(`${label} timed out`);
}

export async function waitPublished(
    expected: GlobalConfig,
    minimumPublications?: number,
): Promise<Record<string, unknown>> {
    return await waitStatus("canonical rollover publication", (status) => {
        if (status.pending !== null || !targetMatches(status.published, expected)) return false;
        return minimumPublications === undefined ||
            field<number>(traversal(status), "automaticPublicationCount", "number") >=
                minimumPublications;
    });
}

export async function enable(): Promise<Record<string, unknown>> {
    await event("composition.traversal.enable");
    await event("workbench.resume");
    return await waitStatus(
        "canonical rollover enable",
        (status) => traversal(status).enabled === true,
    );
}

export async function setCamera(localCenterX: number, localCenterZ = 64): Promise<void> {
    const offsetX = (localCenterX - 64) * REGION_METERS;
    const offsetZ = (localCenterZ - 64) * REGION_METERS;
    await event("camera.set_pose", {
        position: [offsetX, 6, offsetZ],
        target: [offsetX, 1, offsetZ - 3],
        vertical_fov_degrees: 60,
    });
}

export async function camera(): Promise<Record<string, unknown>> {
    return await event("camera.status");
}

export function expectCamera(
    value: Record<string, unknown>,
    localCenterX: number,
    localCenterZ = 64,
): void {
    const offsetX = (localCenterX - 64) * REGION_METERS;
    const offsetZ = (localCenterZ - 64) * REGION_METERS;
    same(value.position, [offsetX, 6, offsetZ], "rollover camera position");
    same(value.target, [offsetX, 1, offsetZ - 3], "rollover camera target");
    const near = field<number>(value, "nearPlaneMeters", "number");
    if (value.verticalFovDegrees !== 60 || Math.abs(near - 0.1) > 0.000001) {
        fail("rollover camera projection changed");
    }
}

export function expectEvent(
    status: Record<string, unknown>,
    expected: {
        oldOrigin: Coord;
        newOrigin: Coord;
        center: Coord;
        localCenter: Coord;
        delta: Coord;
        count: number;
    },
): void {
    const state = rollover(status);
    if (field<number>(state, "count", "number") !== expected.count) {
        fail("rollover count mismatch");
    }
    const last = object(state, "last");
    same(last.oldOrigin, coord(expected.oldOrigin), "rollover old origin");
    same(last.newOrigin, coord(expected.newOrigin), "rollover new origin");
    same(last.globalCenter, coord(expected.center), "rollover global center");
    same(last.localCenter, expected.localCenter, "rollover local center");
    same(last.cameraDeltaRegions, expected.delta, "rollover camera delta regions");
    same(
        last.cameraDeltaMeters,
        expected.delta.map((value) => value * REGION_METERS),
        "rollover camera delta meters",
    );
}

export async function publication(
    status: Record<string, unknown>,
    expected: GlobalConfig,
): Promise<Record<string, unknown>> {
    const published = object(status, "published");
    return { published, halves: await halfReports(published, expected) };
}

function configMatches(value: unknown, expected: Record<string, number>): boolean {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    const actual = value as Record<string, unknown>;
    return Object.entries(expected).every(([name, expectedValue]) =>
        actual[name] === expectedValue
    );
}

function coord(value: Coord): Record<string, number> {
    return { x: value[0], z: value[1] };
}
