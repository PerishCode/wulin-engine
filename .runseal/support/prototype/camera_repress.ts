import { fail, type Json, number, object, same } from "../canonical-runtime.ts";
import { cameraDriverInvariant } from "./camera.ts";
import { presentationInvariant } from "./presentation.ts";

function nativeCameraRepressInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const nativeInput = object(launch, "nativeInput");
    const initialPress = object(nativeInput, "initialPress");
    const sequence = object(nativeInput, "sequence");
    const expectedInitialKeys = [{ key: "E", virtualKey: 0x45, down: true }];
    const expectedSequenceKeys = [
        { key: "E", virtualKey: 0x45, down: false },
        { key: "E", virtualKey: 0x45, down: true },
        { key: "W", virtualKey: 0x57, down: true },
    ];
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const exitInterval = number(sequence, "exitIntervalMilliseconds");
    if (
        initialPress.schema !== "prototype-native-window-action-v4" ||
        initialPress.action !== "input" ||
        initialPress.processId !== processId ||
        initialPress.requiredVisible !== true ||
        initialPress.windowWasVisible !== true ||
        JSON.stringify(initialPress.keys) !== JSON.stringify(expectedInitialKeys) ||
        JSON.stringify(initialPress.messages) !==
            JSON.stringify(["WM_SETFOCUS", "WM_KEYDOWN:E"]) ||
        initialPress.atomicBatch !== true ||
        number(initialPress, "atomicPrefixLength") !== 1 ||
        number(nativeInput, "requestedInitialHoldMilliseconds") !== 250 ||
        number(nativeInput, "initialHoldMilliseconds") < 250 ||
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.windowHandle !== initialPress.windowHandle ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !== JSON.stringify(expectedSequenceKeys) ||
        JSON.stringify(sequence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYUP:E",
                "WM_KEYDOWN:E",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        intervals.some((interval) => typeof interval !== "number" || interval < 0) ||
        sequence.atomicBatch !== true ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        number(sequence, "batchThreadId") <= 0 ||
        number(sequence, "batchSpanMilliseconds") < 0 ||
        number(sequence, "batchSpanMilliseconds") > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700
    ) fail("prototype native camera re-press evidence diverged");
    return {
        exactProcessWindow: true,
        initialPress: initialPress.messages,
        releaseAndRepress: sequence.messages,
        initialHoldMilliseconds: nativeInput.initialHoldMilliseconds,
        atomicBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        keyPostIntervalsMilliseconds: intervals,
        exitIntervalMilliseconds: exitInterval,
    };
}

export function cameraRepressSessionInvariant(launch: Json, session: Json): Json {
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
        "prototype camera re-press actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype camera re-press actor region",
    );
    if (
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype camera re-press vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 !== 0 || deltaZQ9 <= 0 || deltaZQ9 % 32 !== 0) {
        fail("prototype camera re-press did not advance to orbit-two locomotion");
    }
    const horizontalSteps = deltaZQ9 / 32;
    if (horizontalSteps < 1 || horizontalSteps > 43) {
        fail("prototype camera re-press horizontal step count diverged");
    }
    const presentation = presentationInvariant(
        object(finalActor, "presentation"),
        1,
        16_384,
        "prototype camera re-press Walk",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype camera re-press did not commit the Walk transition");

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
    ) fail("prototype camera re-press clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeCameraRepressInvariant(launch),
        readinessCamera: camera,
        heldKeyReleased: true,
        freshPressEdgeReadmitted: true,
        committedOrbitIndex: 2,
        horizontalSteps,
        deltaXQ9,
        deltaZQ9,
        presentation,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}
