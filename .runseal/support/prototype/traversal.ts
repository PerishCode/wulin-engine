import {
    type Coord,
    fail,
    type Json,
    number,
    object,
    target,
    targetMatches,
} from "../canonical-runtime.ts";

function targetIsExact(value: unknown, expected: ReturnType<typeof target>): boolean {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    const config = (value as Json).config;
    if (!config || typeof config !== "object" || Array.isArray(config)) return false;
    const local = config as Json;
    return targetMatches(value, expected) && local.activeCenterX === 65 &&
        local.activeCenterZ === 65 && local.activeRadius === 2 && local.worldRegionSide === 128;
}

export function traversalInvariant(launch: Json, base: Coord): Json {
    const traversal = object(object(launch, "readiness"), "traversal");
    const expected = target([base[0] + 1, base[1] + 1], 65, 65);
    if (
        traversal.revision !== "camera-region-traversal-v1" ||
        traversal.enabled !== true ||
        number(traversal, "sessionCount") !== 1 ||
        number(traversal, "desiredChangeCount") !== 1 ||
        number(traversal, "automaticAttemptCount") !== 1 ||
        number(traversal, "automaticScheduleCount") !== 1 ||
        number(traversal, "coalescedReplacementCount") !== 0 ||
        number(traversal, "maxQueuedDepth") !== 0 ||
        traversal.blocked !== null || traversal.queued !== null || traversal.lastFailure !== null
    ) fail("prototype traversal activation counters diverged");
    if (Object.hasOwn(traversal, "prefetch")) {
        fail("prototype unexpectedly configured composition prefetch");
    }
    if (!targetIsExact(traversal.desired, expected)) {
        fail("prototype traversal desired target diverged");
    }
    const scheduled = object(traversal, "lastScheduled");
    if (number(scheduled, "token") !== 2 || !targetIsExact(scheduled, expected)) {
        fail("prototype traversal scheduled target diverged");
    }
    const rollover = object(traversal, "rollover");
    if (number(rollover, "count") !== 0 || rollover.pendingCameraDeltaRegions !== null) {
        fail("prototype traversal rollover state diverged");
    }
    const publicationCount = number(traversal, "automaticPublicationCount");
    if (publicationCount === 0) {
        if (traversal.lastPublished !== null) {
            fail("prototype traversal reported an uncounted publication");
        }
    } else if (publicationCount === 1) {
        const published = object(traversal, "lastPublished");
        if (number(published, "token") !== 2 || !targetIsExact(published, expected)) {
            fail("prototype traversal published target diverged");
        }
    } else {
        fail("prototype traversal publication count exceeded readiness bounds");
    }
    return {
        revision: traversal.revision,
        enabled: true,
        target: expected,
        token: 2,
        exactSingleSchedule: true,
        publicationBounded: true,
        noQueueBlockFailure: true,
        noPrefetch: true,
        noRollover: true,
    };
}
