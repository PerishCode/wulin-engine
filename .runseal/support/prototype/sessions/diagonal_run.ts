import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { presentationInvariant } from "../presentation.ts";

function nativeDiagonalRunInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const sequence = object(object(launch, "nativeInput"), "sequence");
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
                { key: "W", virtualKey: 0x57, down: false },
                { key: "A", virtualKey: 0x41, down: false },
            ]) ||
        JSON.stringify(sequence.messages) !==
            JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Shift",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:A",
                "WM_KEYUP:W",
                "WM_KEYUP:A",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 0, 250, 250]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 4 ||
        intervals.slice(0, 2).some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        typeof intervals[2] !== "number" ||
        intervals[2] < 250 ||
        intervals[2] > 750 ||
        typeof intervals[3] !== "number" ||
        intervals[3] < 250 ||
        intervals[3] > 750 ||
        sequence.atomicBatch !== false ||
        number(sequence, "atomicPrefixLength") !== 3 ||
        typeof sequence.batchThreadId !== "number" ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        sequence.batchThreadId <= 0 ||
        typeof sequence.batchSpanMilliseconds !== "number" ||
        sequence.batchSpanMilliseconds < 0 ||
        sequence.batchSpanMilliseconds > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 250 ||
        exitInterval < 250 ||
        exitInterval > 750
    ) fail("prototype native diagonal Run input evidence diverged");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        diagonalKeyPostIntervalCount: 2,
        diagonalHoldMilliseconds: intervals[2],
        exactMessageOrder: true,
        leftRunHoldMilliseconds: intervals[3],
        stationaryHoldMilliseconds: exitInterval,
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
        deltaZQ9 >= 0 ||
        deltaZQ9 % 45 !== 0 ||
        deltaXQ9 >= deltaZQ9 ||
        (deltaZQ9 - deltaXQ9) % 64 !== 0
    ) fail("prototype diagonal-to-left Run decomposition diverged");
    const diagonalRunStepCount = -deltaZQ9 / 45;
    const leftRunStepCount = (deltaZQ9 - deltaXQ9) / 64;
    if (
        diagonalRunStepCount < 1 || diagonalRunStepCount > 512 ||
        leftRunStepCount < 1 || leftRunStepCount > 512
    ) {
        fail("prototype diagonal-to-left Run phase bounds diverged");
    }
    const finalPresentation = presentationInvariant(
        object(finalActor, "presentation"),
        0,
        32_768,
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
        forwardInputReleased: true,
        retainedLeftRun: true,
        exactTwoPhaseDisplacement: true,
        leftInputReleased: true,
        movedThenStopped: true,
        transitionedToSurvey: true,
        retainedLeftYaw: true,
        diagonalRunStepCount,
        leftRunStepCount,
        deltaXQ9,
        deltaZQ9,
        readyPresentation,
        finalPresentation,
        animationEpochTransitioned: true,
        clock: {
            continuityValidated: true,
            discontinuity: false,
        },
    };
}
