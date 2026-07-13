import { canonicalObjects } from "./canonical-object-composition.ts";
import type { Coord } from "./canonical-origin-rollover.ts";
import { distribution, fail, field, object } from "./terrain.ts";

export function capability(status: Record<string, unknown>, debug: boolean) {
    const renderer = object(status, "renderer");
    if (renderer.debugLayer !== debug || renderer.deviceRemovedReason !== null) {
        fail(`${debug ? "debug" : "release"} rollover capability gate failed`);
    }
    return renderer;
}

export function packCenters(anchors: Coord[]): Coord[] {
    const unique = new Map<string, Coord>();
    const add = (value: Coord) => unique.set(`${value[0]},${value[1]}`, value);
    for (const anchor of anchors) {
        for (let offset = -40; offset <= 80; offset += 1) {
            add([anchor[0] + offset, anchor[1]]);
            add([anchor[0], anchor[1] + offset]);
        }
        for (const offset of [-33, -32, 32, 33]) {
            add([anchor[0] + offset, anchor[1] + offset]);
        }
    }
    return [...unique.values()];
}

export function retainedOverlap(
    before: Record<string, unknown>,
    after: Record<string, unknown>,
    expected: number,
): Record<string, number> {
    const previous = new Map<string, number>();
    for (const raw of canonicalObjects(before).entries as Record<string, unknown>[]) {
        previous.set(JSON.stringify(raw.globalRegion), Number(raw.stableSeed));
    }
    let retainedCount = 0;
    let mismatchCount = 0;
    for (const raw of canonicalObjects(after).entries as Record<string, unknown>[]) {
        const seed = previous.get(JSON.stringify(raw.globalRegion));
        if (seed === undefined) continue;
        retainedCount += 1;
        if (seed !== raw.stableSeed) mismatchCount += 1;
    }
    if (retainedCount !== expected || mismatchCount !== 0) {
        fail(`rollover retained identity mismatch ${retainedCount}/${expected}/${mismatchCount}`);
    }
    return { retainedCount, mismatchCount };
}

export function objectTimings(samples: Record<string, unknown>[]) {
    const values = (name: string) => samples.map((sample) => field<number>(sample, name, "number"));
    return {
        sampleCount: samples.length,
        generationMs: distribution(values("generationMs"), "object generation", true),
        scheduleMs: distribution(values("scheduleMs")),
        pendingMs: distribution(values("pendingMs")),
    };
}
