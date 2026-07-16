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
    const expectedLocalX = 64 + expected.center_x - expected.origin_x;
    const expectedLocalZ = 64 + expected.center_z - expected.origin_z;
    return targetMatches(value, expected) && local.activeCenterX === expectedLocalX &&
        local.activeCenterZ === expectedLocalZ &&
        local.activeRadius === 2 && local.worldRegionSide === 128;
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

export function cameraOrbitTraversalInvariant(launch: Json, base: Coord): Json {
    const traversal = object(object(launch, "readiness"), "traversal");
    const initial = target([base[0] + 1, base[1] + 1], 65, 65);
    const rotated = target([base[0] + 1, base[1] - 1], 65, 63);
    const desiredChangeCount = number(traversal, "desiredChangeCount");
    const attemptCount = number(traversal, "automaticAttemptCount");
    const scheduleCount = number(traversal, "automaticScheduleCount");
    const publicationCount = number(traversal, "automaticPublicationCount");
    const maxQueuedDepth = number(traversal, "maxQueuedDepth");
    if (
        traversal.revision !== "camera-region-traversal-v1" ||
        traversal.enabled !== true ||
        number(traversal, "sessionCount") !== 1 ||
        (desiredChangeCount !== 1 && desiredChangeCount !== 2) ||
        attemptCount !== scheduleCount ||
        scheduleCount < 1 || scheduleCount > desiredChangeCount ||
        publicationCount < 0 || publicationCount > scheduleCount ||
        number(traversal, "coalescedReplacementCount") !== 0 ||
        maxQueuedDepth < 0 || maxQueuedDepth > 1 ||
        (desiredChangeCount === 1 && maxQueuedDepth !== 0) ||
        (desiredChangeCount === 2 && scheduleCount === 1 && maxQueuedDepth !== 1) ||
        traversal.blocked !== null || traversal.lastFailure !== null
    ) fail("prototype camera-orbit traversal counters diverged");
    if (Object.hasOwn(traversal, "prefetch")) {
        fail("prototype camera orbit unexpectedly configured composition prefetch");
    }
    if (!targetIsExact(traversal.desired, rotated)) {
        fail("prototype camera-orbit traversal desired target diverged");
    }

    const scheduled = object(traversal, "lastScheduled");
    const expectedScheduled = scheduleCount === 2 || desiredChangeCount === 1 ? rotated : initial;
    if (
        number(scheduled, "token") !== scheduleCount + 1 ||
        !targetIsExact(scheduled, expectedScheduled)
    ) fail("prototype camera-orbit traversal scheduled target diverged");

    if (scheduleCount === 1 && desiredChangeCount === 2) {
        if (!targetIsExact(traversal.queued, rotated)) {
            fail("prototype camera-orbit traversal queued target diverged");
        }
    } else if (traversal.queued !== null) {
        fail("prototype camera-orbit traversal retained an unexpected queue");
    }
    if (scheduleCount === 2 && publicationCount < 1) {
        fail("prototype camera-orbit traversal rescheduled before publication");
    }

    if (publicationCount === 0) {
        if (traversal.lastPublished !== null) {
            fail("prototype camera orbit reported an uncounted publication");
        }
    } else {
        const published = object(traversal, "lastPublished");
        const expectedPublished = publicationCount === 1 && desiredChangeCount === 2
            ? initial
            : rotated;
        if (
            number(published, "token") !== publicationCount + 1 ||
            !targetIsExact(published, expectedPublished)
        ) fail("prototype camera-orbit traversal published target diverged");
    }

    const rollover = object(traversal, "rollover");
    if (number(rollover, "count") !== 0 || rollover.pendingCameraDeltaRegions !== null) {
        fail("prototype camera-orbit traversal rollover state diverged");
    }
    return {
        revision: traversal.revision,
        enabled: true,
        target: rotated,
        desiredChangeCount,
        scheduleCount,
        publicationCount,
        latestWinsDepth: maxQueuedDepth,
        noBlockFailure: true,
        noPrefetch: true,
        noRollover: true,
    };
}
