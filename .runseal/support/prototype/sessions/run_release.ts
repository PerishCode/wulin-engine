import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { presentationInvariant } from "../presentation.ts";

function nativeRunReleaseInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const sequence = object(object(launch, "nativeInput"), "sequence");
    const expectedKeys = [
        { key: "Shift", virtualKey: 0x10, down: true },
        { key: "W", virtualKey: 0x57, down: true },
        { key: "Shift", virtualKey: 0x10, down: false },
    ];
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const exitInterval = number(sequence, "exitIntervalMilliseconds");
    if (
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !== JSON.stringify(expectedKeys) ||
        JSON.stringify(sequence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Shift",
                "WM_KEYDOWN:W",
                "WM_KEYUP:Shift",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0, 500]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        typeof intervals[0] !== "number" ||
        intervals[0] < 0 ||
        intervals[0] > 50 ||
        typeof intervals[1] !== "number" ||
        intervals[1] < 500 ||
        intervals[1] > 1_000 ||
        sequence.atomicBatch !== false ||
        number(sequence, "atomicPrefixLength") !== 2 ||
        typeof sequence.batchThreadId !== "number" ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        sequence.batchThreadId <= 0 ||
        typeof sequence.batchSpanMilliseconds !== "number" ||
        sequence.batchSpanMilliseconds < 0 ||
        sequence.batchSpanMilliseconds > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700
    ) fail("prototype native Run modifier release evidence diverged");
    return {
        exactProcessWindow: true,
        atomicInitialPrefix: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        orderedMessages: sequence.messages,
        runHoldIntervalMilliseconds: intervals[1],
        exitIntervalMilliseconds: exitInterval,
    };
}

export function runReleaseSessionInvariant(launch: Json, session: Json): Json {
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
    const readyPresentation = presentationInvariant(
        object(readyActor, "presentation"),
        0,
        0,
        "prototype Run modifier release readiness",
    );
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype Run modifier release actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype Run modifier release actor region",
    );
    if (
        number(readyPosition, "localXQ9") !== 0 ||
        number(readyPosition, "localZQ9") !== 0 ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype Run modifier release vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 !== 0 || deltaZQ9 >= 0 || deltaZQ9 % 32 !== 0) {
        fail("prototype Run modifier release did not retain forward locomotion");
    }
    const forwardDisplacementUnits32Q9 = -deltaZQ9 / 32;
    if (forwardDisplacementUnits32Q9 < 1 || forwardDisplacementUnits32Q9 > 1_024) {
        fail("prototype Run modifier release displacement bound diverged");
    }
    const finalPresentation = presentationInvariant(
        object(finalActor, "presentation"),
        1,
        49_152,
        "prototype Run modifier released Walk",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype Run modifier release did not commit the Walk transition");

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
    ) fail("prototype Run modifier release clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeRunReleaseInvariant(launch),
        readinessCamera: camera,
        runModifierReleased: true,
        retainedForwardInput: true,
        transitionedToWalk: true,
        deltaXQ9,
        deltaZQ9,
        forwardDisplacementUnits32Q9,
        readyPresentation,
        finalPresentation,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}
