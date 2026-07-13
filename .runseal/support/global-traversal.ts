import { type GlobalConfig, globalConfig, halfReports } from "./global-composition.ts";
import { event, sleep } from "./global-terrain.ts";
import { configMatches, traversal as traversalState, worldCenter } from "./traversal.ts";
import { fail, field, object } from "./terrain.ts";

export type Coord = [number, number];

export function target(origin: Coord, x: number, z: number): GlobalConfig {
    return globalConfig(origin, [origin[0] + x - 64, origin[1] + z - 64]);
}

export function targetMatches(
    value: unknown,
    x: number,
    z: number,
    origin: Coord,
): boolean {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    const owner = value as Record<string, unknown>;
    if (!configMatches(owner.config, x, z)) return false;
    if (!owner.globalConfig || typeof owner.globalConfig !== "object") return false;
    const global = owner.globalConfig as Record<string, unknown>;
    const actualOrigin = object(global, "globalOrigin");
    const center = object(global, "globalCenter");
    return actualOrigin.x === origin[0] && actualOrigin.z === origin[1] &&
        center.x === origin[0] + x - 64 && center.z === origin[1] + z - 64 &&
        global.activeRadius === 2;
}

export function traversal(
    status: Record<string, unknown>,
    origin?: Coord,
): Record<string, unknown> {
    const value = traversalState(status);
    if (origin && value.basis) {
        const basis = object(value, "basis");
        const actual = object(basis, "globalOrigin");
        if (
            actual.x !== origin[0] || actual.z !== origin[1] ||
            basis.worldRegionSide !== 128 || basis.activeRadius !== 2
        ) fail("signed traversal basis changed");
    }
    return value;
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
    origin: Coord,
    x: number,
    z: number,
    minimumPublications?: number,
): Promise<Record<string, unknown>> {
    return await waitStatus(`global publication ${x},${z}`, (status) => {
        if (status.pending !== null || !targetMatches(status.published, x, z, origin)) {
            return false;
        }
        return minimumPublications === undefined ||
            field<number>(traversal(status, origin), "automaticPublicationCount", "number") >=
                minimumPublications;
    });
}

export async function waitDesired(
    origin: Coord,
    x: number,
    z: number,
): Promise<Record<string, unknown>> {
    return await waitStatus(
        `global desired ${x},${z}`,
        (status) => targetMatches(traversal(status, origin).desired, x, z, origin),
    );
}

export async function setPosition(x: number, z: number): Promise<void> {
    await event("camera.set_pose", {
        position: [x, 6, z],
        target: [x, 1, z - 3],
        vertical_fov_degrees: 60,
    });
}

export async function setCenter(x: number, z: number): Promise<void> {
    await setPosition(worldCenter(x), worldCenter(z));
}

export async function moveAndPublish(
    origin: Coord,
    x: number,
    z: number,
): Promise<Record<string, unknown>> {
    const before = traversal(await event("composition.status"), origin);
    const publication = field<number>(before, "automaticPublicationCount", "number") + 1;
    await setCenter(x, z);
    return await waitPublished(origin, x, z, publication);
}

export async function enable(origin: Coord, x = 64, z = 64): Promise<Record<string, unknown>> {
    await event("composition.traversal.enable");
    await event("workbench.resume");
    const status = await waitDesired(origin, x, z);
    const state = traversal(status, origin);
    if (state.enabled !== true) fail("signed traversal did not enable");
    return status;
}

export async function publication(
    status: Record<string, unknown>,
    config: GlobalConfig,
): Promise<Record<string, unknown>> {
    const published = object(status, "published");
    return {
        published,
        halves: await halfReports(published, config),
    };
}
