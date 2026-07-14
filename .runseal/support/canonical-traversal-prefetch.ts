import { expectPairMovement } from "./canonical-object-composition.ts";
import {
    type Coord,
    startClean,
    targetMatches,
    traversal,
    waitStatus,
} from "./canonical-origin-rollover.ts";
import { type GlobalConfig, prepare } from "./global-composition.ts";
import { event, sleep } from "./global-terrain.ts";
import { fail, field, object } from "./terrain.ts";

export type PrefetchContext = {
    collection: string;
    pack: string;
    missingPack: string;
    corruptPack: string;
    objectPack?: string;
    base: Coord;
};

export async function setupPrefetch(
    pack: string,
    config: GlobalConfig,
    position: Coord,
    sidecar = "sidecar.toml",
    objectPack?: string,
): Promise<Record<string, unknown>> {
    await startClean(sidecar);
    const initial = await prepare(pack, config, objectPack);
    await setPosition(position);
    await event("composition.traversal.enable");
    const enabled = await event("composition.prefetch.enable");
    await event("workbench.resume");
    await sleep(30);
    if (prefetch(enabled).enabled !== true) fail("canonical prefetch did not enable");
    return { initial, enabled };
}

export async function setPosition(position: Coord): Promise<void> {
    await event("camera.set_pose", {
        position: [position[0], 6, position[1]],
        target: [position[0], 1, position[1] - 3],
        vertical_fov_degrees: 60,
    });
}

export function prefetch(status: Record<string, unknown>): Record<string, unknown> {
    return object(traversal(status), "prefetch");
}

export async function waitPrefetchPending(
    expected: GlobalConfig,
    label: string,
): Promise<Record<string, unknown>> {
    return await waitStatus(label, (status) => {
        if (!status.pending || !targetMatches(status.pending, expected)) return false;
        return object(status, "pending").prefetch === true;
    });
}

export async function waitPrefetchCompletion(
    expected: GlobalConfig,
    count: number,
    label: string,
): Promise<Record<string, unknown>> {
    return await waitStatus(label, (status) => {
        const state = prefetch(status);
        return status.pending === null && state.completionCount === count &&
            targetMatches(state.lastCompleted, expected);
    });
}

export async function waitPrefetchFailure(
    expected: GlobalConfig,
    count: number,
): Promise<Record<string, unknown>> {
    return await waitStatus("prefetch failure", (status) => {
        const state = prefetch(status);
        if (status.pending !== null || state.failureCount !== count) return false;
        const failure = object(state, "lastFailure");
        return targetMatches(object(failure, "target"), expected);
    });
}

export function completionReports(status: Record<string, unknown>): {
    completion: Record<string, unknown>;
    terrain: Record<string, unknown>;
    objects: Record<string, unknown>;
} {
    const completion = object(prefetch(status), "lastCompleted");
    return {
        completion,
        terrain: object(completion, "terrain"),
        objects: object(completion, "objects"),
    };
}

export function expectPreparedCounts(
    status: Record<string, unknown>,
    retained: number,
    uploaded: number,
): ReturnType<typeof completionReports> {
    const reports = completionReports(status);
    expectHalf(reports.terrain, "terrain", retained, uploaded, 4_096);
    expectHalf(reports.objects, "objects", retained, uploaded, 20_480);
    return reports;
}

export function expectDemandCounts(
    report: Record<string, unknown>,
    retained: number,
    uploaded: number,
): void {
    expectPairMovement(report, retained, uploaded, uploaded * 4_096, uploaded * 20_480);
}

export function publicationToken(status: Record<string, unknown>): number {
    return field<number>(object(status, "published"), "token", "number");
}

export function localCenterPosition(local: Coord): Coord {
    return [(local[0] - 64) * 16, (local[1] - 64) * 16];
}

function expectHalf(
    report: Record<string, unknown>,
    name: string,
    retained: number,
    uploaded: number,
    bytesPerRegion: number,
): void {
    if (
        field<number>(report, "retainedRegionCount", "number") !== retained ||
        field<number>(report, "uploadedRegionCount", "number") !== uploaded
    ) fail(`${name} prepared counts diverged`);
    const bytes = name === "terrain"
        ? field<number>(report, "payloadBytes", "number")
        : field<number>(report, "instanceBytes", "number");
    if (bytes !== uploaded * bytesPerRegion) fail(`${name} prepared bytes diverged`);
}
