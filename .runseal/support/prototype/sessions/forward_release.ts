import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { presentationInvariant } from "../presentation.ts";

function nativeForwardReleaseInvariant(launch: Json): Json {
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
                { key: "W", virtualKey: 0x57, down: true },
                { key: "W", virtualKey: 0x57, down: false },
            ]) ||
        JSON.stringify(sequence.messages) !==
            JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:W",
                "WM_KEYUP:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 250]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        typeof intervals[0] !== "number" ||
        intervals[0] < 250 ||
        intervals[0] > 750 ||
        sequence.atomicBatch !== false ||
        number(sequence, "atomicPrefixLength") !== 1 ||
        typeof sequence.batchThreadId !== "number" ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        sequence.batchThreadId <= 0 ||
        number(sequence, "batchSpanMilliseconds") !== 0 ||
        number(sequence, "exitAfterLastMilliseconds") !== 250 ||
        exitInterval < 250 ||
        exitInterval > 750
    ) fail("prototype native forward-release evidence diverged");
    same(sequence, object(launch, "exitInput"), "prototype forward-release exit input");
    return {
        exactProcessWindow: true,
        atomicInitialPress: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        orderedMessages: sequence.messages,
        walkHoldIntervalMilliseconds: intervals[0],
        stationaryHoldIntervalMilliseconds: exitInterval,
    };
}

export function forwardReleaseSessionInvariant(launch: Json, session: Json): Json {
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
        "prototype forward-release readiness",
    );
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype forward-release actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype forward-release actor region",
    );
    if (
        number(readyPosition, "localXQ9") !== 0 ||
        number(readyPosition, "localZQ9") !== 0 ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype forward-release vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 !== 0 || deltaZQ9 >= 0 || deltaZQ9 % 32 !== 0) {
        fail("prototype forward-release did not commit exact Walk movement");
    }
    const forwardStepCount = -deltaZQ9 / 32;
    if (forwardStepCount < 1 || forwardStepCount > 512) {
        fail("prototype forward-release step bound diverged");
    }
    const finalPresentation = presentationInvariant(
        object(finalActor, "presentation"),
        0,
        49_152,
        "prototype forward-release final Survey",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype forward-release did not commit its presentation transitions");

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
    ) fail("prototype forward-release clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeForwardReleaseInvariant(launch),
        readinessCamera: camera,
        normalForwardReleased: true,
        movedThenStopped: true,
        transitionedToSurvey: true,
        retainedForwardYaw: true,
        deltaXQ9,
        deltaZQ9,
        forwardStepCount,
        readyPresentation,
        finalPresentation,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}
