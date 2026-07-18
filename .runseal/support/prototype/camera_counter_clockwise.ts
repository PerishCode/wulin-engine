import { fail, type Json, number, object, same } from "../canonical-runtime.ts";
import { cameraDriverInvariant } from "./camera.ts";
import { presentationInvariant } from "./presentation.ts";

function nativeCounterClockwiseInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const sequence = object(object(launch, "nativeInput"), "sequence");
    const expectedKeys = [
        { key: "Q", virtualKey: 0x51, down: true },
        { key: "W", virtualKey: 0x57, down: true },
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
                "WM_KEYDOWN:Q",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        typeof intervals[0] !== "number" ||
        intervals[0] < 0 ||
        sequence.atomicBatch !== true ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        number(sequence, "batchThreadId") <= 0 ||
        number(sequence, "batchSpanMilliseconds") < 0 ||
        number(sequence, "batchSpanMilliseconds") > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700
    ) fail("prototype native counter-clockwise camera evidence diverged");
    return {
        exactProcessWindow: true,
        exactMessageOrder: true,
        atomicBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        keyPostIntervalMilliseconds: intervals[0],
        exitIntervalMilliseconds: exitInterval,
    };
}

export function counterClockwiseSessionInvariant(launch: Json, session: Json): Json {
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
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype counter-clockwise camera actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype counter-clockwise camera actor region",
    );
    if (
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype counter-clockwise camera vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 <= 0 || deltaXQ9 % 32 !== 0 || deltaZQ9 !== 0) {
        fail("prototype counter-clockwise camera did not wrap to orbit-three locomotion");
    }
    const horizontalSteps = deltaXQ9 / 32;
    if (horizontalSteps < 1 || horizontalSteps > 43) {
        fail("prototype counter-clockwise camera horizontal step count diverged");
    }
    const presentation = presentationInvariant(
        object(finalActor, "presentation"),
        1,
        0,
        "prototype counter-clockwise camera Walk",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype counter-clockwise camera did not commit the Walk transition");

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
    ) fail("prototype counter-clockwise camera clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeCounterClockwiseInvariant(launch),
        readinessCamera: camera,
        counterClockwisePressEdgeRetained: true,
        wrappedOrbitIndex: 3,
        horizontalSteps,
        deltaXQ9,
        deltaZQ9,
        presentation,
        clock: {
            continuityValidated: true,
            discontinuity: false,
        },
    };
}
