import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { presentationInvariant } from "../presentation.ts";

function nativeDiagonalRunInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const sequence = object(object(launch, "postReadinessInput"), "sequence");
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const exitInterval = number(sequence, "exitIntervalMilliseconds");
    if (
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !==
            JSON.stringify([
                { key: "Shift", virtualKey: 0x10, down: true },
                { key: "W", virtualKey: 0x57, down: true },
                { key: "A", virtualKey: 0x41, down: true },
            ]) ||
        JSON.stringify(sequence.messages) !==
            JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Shift",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:A",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        intervals.some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        sequence.atomicBatch !== true ||
        number(sequence, "atomicPrefixLength") !== 3 ||
        typeof sequence.batchThreadId !== "number" ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        sequence.batchThreadId <= 0 ||
        typeof sequence.batchSpanMilliseconds !== "number" ||
        sequence.batchSpanMilliseconds < 0 ||
        sequence.batchSpanMilliseconds > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700 ||
        Object.hasOwn(launch, "startupNativeInput")
    ) fail("prototype native diagonal Run input evidence diverged");
    same(sequence, object(launch, "exitInput"), "prototype diagonal Run exit input");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        keyPostIntervalsMilliseconds: intervals,
        orderedMessages: sequence.messages,
        exitIntervalMilliseconds: exitInterval,
        actionAfterReadiness: true,
    };
}

export function diagonalRunSessionInvariant(launch: Json, session: Json): Json {
    const camera = cameraDriverInvariant(launch, 0);
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");
    const readyPosition = object(readyBody, "position");
    const finalPosition = object(finalBody, "position");
    const readyXQ9 = number(readyPosition, "localXQ9");
    const readyZQ9 = number(readyPosition, "localZQ9");
    if (readyXQ9 !== 0 || readyZQ9 !== 0) {
        fail("prototype diagonal Run readiness moved before action");
    }
    const readyPresentation = presentationInvariant(
        object(readyActor, "presentation"),
        0,
        0,
        "prototype diagonal Run readiness",
    );
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype diagonal Run actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype diagonal Run actor region",
    );
    if (
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype diagonal Run vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") - readyXQ9;
    const deltaZQ9 = number(finalPosition, "localZQ9") - readyZQ9;
    if (
        deltaXQ9 >= 0 ||
        deltaXQ9 !== deltaZQ9 ||
        deltaXQ9 % 45 !== 0
    ) fail("prototype diagonal Run completion normalization diverged");
    const diagonalRunStepCount = -deltaXQ9 / 45;
    if (diagonalRunStepCount < 1 || diagonalRunStepCount > 512) {
        fail("prototype diagonal Run completion step bound diverged");
    }
    const finalPresentation = presentationInvariant(
        object(finalActor, "presentation"),
        2,
        40_960,
        "prototype diagonal Run completion",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype diagonal Run did not commit its presentation transition");

    const readyClock = object(object(readiness, "simulation_driver"), "clock");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype diagonal Run clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeDiagonalRunInvariant(launch),
        readinessCamera: camera,
        atomicDiagonalRunInput: true,
        nativeLeftInput: true,
        exactRunNormalization: true,
        diagonalRunStepCount,
        deltaXQ9,
        deltaZQ9,
        readyPresentation,
        finalPresentation,
        actionAfterReadiness: true,
        animationEpochTransitioned: true,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}
