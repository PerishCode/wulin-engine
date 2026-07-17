import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { presentationInvariant } from "../presentation.ts";

function nativeDiagonalWalkInvariant(launch: Json): Json {
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
        JSON.stringify(sequence.keys) !== JSON.stringify([
                { key: "W", virtualKey: 0x57, down: true },
                { key: "A", virtualKey: 0x41, down: true },
                { key: "W", virtualKey: 0x57, down: false },
            ]) ||
        JSON.stringify(sequence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:A",
                "WM_KEYUP:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 250]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        typeof intervals[0] !== "number" ||
        intervals[0] < 0 ||
        intervals[0] > 50 ||
        typeof intervals[1] !== "number" ||
        intervals[1] < 250 ||
        intervals[1] > 750 ||
        sequence.atomicBatch !== false ||
        number(sequence, "atomicPrefixLength") !== 2 ||
        typeof sequence.batchThreadId !== "number" ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        sequence.batchThreadId <= 0 ||
        typeof sequence.batchSpanMilliseconds !== "number" ||
        sequence.batchSpanMilliseconds < 0 ||
        sequence.batchSpanMilliseconds > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 250 ||
        exitInterval < 250 ||
        exitInterval > 750
    ) fail("prototype native diagonal Walk input evidence diverged");
    same(sequence, object(launch, "exitInput"), "prototype diagonal Walk exit input");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        diagonalKeyPostIntervalMilliseconds: intervals[0],
        diagonalHoldMilliseconds: intervals[1],
        orderedMessages: sequence.messages,
        leftWalkHoldMilliseconds: exitInterval,
        actionAfterReadiness: true,
    };
}

export function diagonalWalkSessionInvariant(launch: Json, session: Json): Json {
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
        fail("prototype diagonal Walk readiness moved before action");
    }
    const readyPresentation = presentationInvariant(
        object(readyActor, "presentation"),
        0,
        0,
        "prototype diagonal Walk readiness",
    );
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype diagonal Walk actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype diagonal Walk actor region",
    );
    if (
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype diagonal Walk vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") - readyXQ9;
    const deltaZQ9 = number(finalPosition, "localZQ9") - readyZQ9;
    if (
        deltaZQ9 >= 0 ||
        deltaZQ9 % 23 !== 0 ||
        deltaXQ9 >= deltaZQ9 ||
        (deltaZQ9 - deltaXQ9) % 32 !== 0
    ) fail("prototype diagonal-to-left Walk decomposition diverged");
    const diagonalStepCount = -deltaZQ9 / 23;
    const leftStepCount = (deltaZQ9 - deltaXQ9) / 32;
    if (
        diagonalStepCount < 1 || diagonalStepCount > 512 ||
        leftStepCount < 1 || leftStepCount > 512
    ) {
        fail("prototype diagonal-to-left Walk phase bounds diverged");
    }
    const finalPresentation = presentationInvariant(
        object(finalActor, "presentation"),
        1,
        32_768,
        "prototype diagonal Walk completion",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype diagonal Walk did not commit its presentation transition");

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
    ) fail("prototype diagonal Walk clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeDiagonalWalkInvariant(launch),
        readinessCamera: camera,
        atomicDiagonalInput: true,
        nativeLeftInput: true,
        exactWalkNormalization: true,
        forwardInputReleased: true,
        retainedLeftWalk: true,
        exactTwoPhaseDisplacement: true,
        diagonalStepCount,
        leftStepCount,
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
