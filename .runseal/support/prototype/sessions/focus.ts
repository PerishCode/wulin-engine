import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { presentationInvariant } from "../presentation.ts";

function nativeFocusInvariant(
    suspended: Json,
    resumed: Json,
    readmission: Json,
    processId: number,
): Json {
    const readmissionIntervals = readmission.keyPostIntervalsMilliseconds;
    const exitInterval = number(readmission, "exitIntervalMilliseconds");
    if (
        suspended.schema !== "prototype-native-window-action-v4" ||
        suspended.action !== "suspend" ||
        suspended.processId !== processId ||
        suspended.activated !== true ||
        suspended.closeRequested !== false ||
        suspended.requiredVisible !== true ||
        suspended.windowWasVisible !== true ||
        JSON.stringify(suspended.keys) !==
            JSON.stringify([
                { key: "Space", virtualKey: 32, down: true },
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: true },
                { key: "W", virtualKey: 87, down: true },
            ]) ||
        JSON.stringify(suspended.messages) !==
            JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Space",
                "WM_KEYDOWN:F",
                "WM_KEYDOWN:Enter",
                "WM_KEYDOWN:W",
                "WM_KILLFOCUS",
            ]) ||
        JSON.stringify(suspended.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 0, 0]) ||
        !Array.isArray(suspended.keyPostIntervalsMilliseconds) ||
        suspended.keyPostIntervalsMilliseconds.length !== 3 ||
        suspended.keyPostIntervalsMilliseconds.some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        suspended.atomicBatch !== true ||
        number(suspended, "atomicPrefixLength") !== 4 ||
        number(suspended, "batchThreadId") <= 0 ||
        number(suspended, "batchSpanMilliseconds") < 0 ||
        number(suspended, "batchSpanMilliseconds") > 50 ||
        resumed.schema !== "prototype-native-window-action-v4" ||
        resumed.action !== "resume" ||
        resumed.processId !== processId ||
        resumed.windowHandle !== suspended.windowHandle ||
        resumed.activated !== true ||
        resumed.closeRequested !== false ||
        resumed.requiredVisible !== true ||
        resumed.windowWasVisible !== true ||
        !Array.isArray(resumed.keys) ||
        resumed.keys.length !== 0 ||
        JSON.stringify(resumed.messages) !== JSON.stringify(["WM_SETFOCUS"]) ||
        readmission.schema !== "prototype-native-window-action-v4" ||
        readmission.action !== "input" ||
        readmission.processId !== processId ||
        readmission.windowHandle !== suspended.windowHandle ||
        readmission.windowHandle !== resumed.windowHandle ||
        readmission.activated !== true ||
        readmission.closeRequested !== false ||
        readmission.requiredVisible !== true ||
        readmission.windowWasVisible !== true ||
        JSON.stringify(readmission.keys) !==
            JSON.stringify([
                { key: "A", virtualKey: 65, down: true },
                { key: "A", virtualKey: 65, down: false },
            ]) ||
        JSON.stringify(readmission.messages) !==
            JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:A",
                "WM_KEYUP:A",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(readmission.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 250]) ||
        !Array.isArray(readmissionIntervals) ||
        readmissionIntervals.length !== 1 ||
        typeof readmissionIntervals[0] !== "number" ||
        readmissionIntervals[0] < 250 ||
        readmissionIntervals[0] > 750 ||
        readmission.atomicBatch !== false ||
        number(readmission, "atomicPrefixLength") !== 1 ||
        typeof readmission.batchThreadId !== "number" ||
        !Number.isSafeInteger(readmission.batchThreadId) ||
        readmission.batchThreadId <= 0 ||
        number(readmission, "batchSpanMilliseconds") !== 0 ||
        number(readmission, "exitAfterLastMilliseconds") !== 250 ||
        exitInterval < 250 ||
        exitInterval > 750
    ) fail("prototype native focus-discontinuity evidence diverged");
    return {
        exactProcessWindow: true,
        exactSuspendedMessageOrder: true,
        exactResumedMessageOrder: true,
        exactReadmissionMessageOrder: true,
        atomicWindowThreadBatch: {
            threadId: suspended.batchThreadId,
            spanMilliseconds: suspended.batchSpanMilliseconds,
        },
        atomicFreshLocomotionPress: {
            threadId: readmission.batchThreadId,
            spanMilliseconds: readmission.batchSpanMilliseconds,
        },
        leftHoldIntervalMilliseconds: readmissionIntervals[0],
        stationaryHoldIntervalMilliseconds: exitInterval,
        actionPressBeforeFocusLoss: true,
        observationPressBeforeFocusLoss: true,
        activationPressBeforeFocusLoss: true,
        locomotionPressBeforeFocusLoss: true,
        freshLocomotionAfterResume: true,
        synthesizedFocusState: false,
    };
}

export function focusSessionInvariant(launch: Json, session: Json): Json {
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
        "prototype focus-discontinuity readiness",
    );
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype focus-discontinuity actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype focus-discontinuity actor region",
    );
    if (
        number(readyPosition, "localXQ9") !== 0 ||
        number(readyPosition, "localZQ9") !== 0 ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype focus-discontinuity actor body diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 >= 0 || deltaXQ9 % 32 !== 0 || deltaZQ9 !== 0) {
        fail("prototype focus-discontinuity locomotion readmission diverged");
    }
    const leftStepCount = -deltaXQ9 / 32;
    if (leftStepCount < 1 || leftStepCount > 512) {
        fail("prototype focus-discontinuity left step bound diverged");
    }
    const finalPresentation = presentationInvariant(
        object(finalActor, "presentation"),
        0,
        32_768,
        "prototype focus-discontinuity final Survey",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype focus-discontinuity did not commit its presentation transitions");

    const jump = object(object(readiness, "jump_driver"), "status");
    if (jump.pending !== false || jump.grounded !== true) {
        fail("prototype focus-discontinuity did not begin from grounded idle Jump state");
    }
    const readyObservation = object(object(readiness, "object_observation_driver"), "status");
    const readyInteraction = object(object(readiness, "object_interaction_driver"), "status");
    if (
        readyObservation.pending !== false ||
        readyObservation.target !== null ||
        readyInteraction.pending !== false ||
        readyInteraction.acknowledgement !== null ||
        number(readyInteraction, "committedCount") !== 0 ||
        number(readyInteraction, "ineligibleCount") !== 0 ||
        readyInteraction.consumed !== null
    ) fail("prototype focus-discontinuity did not begin from idle object policies");

    const readyClock = object(object(readiness, "simulation_driver"), "clock");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") + 1 ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") + 1 ||
        number(finalClock, "suspendedSampleCount") <=
            number(readyClock, "suspendedSampleCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") + 1 ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype focus-discontinuity clock recovery diverged");

    const nativeInput = object(launch, "nativeInput");
    const readmission = object(nativeInput, "readmission");
    return {
        ...session,
        readinessCamera: camera,
        postFocusLocomotionReadmitted: true,
        staleForwardDidNotReachSimulation: true,
        freshLeftInputReadmitted: true,
        movedThenStopped: true,
        retainedLeftYaw: true,
        sameBatchJumpDidNotReachResumedSimulation: true,
        sameBatchObservationDidNotReachResumedSimulation: true,
        sameBatchActivationDidNotReachResumedSimulation: true,
        sameBatchForwardDidNotReachResumedSimulation: true,
        objectPoliciesIdleAcrossDiscontinuity: true,
        resumedReadyProgress: true,
        deltaXQ9,
        deltaZQ9,
        leftStepCount,
        readyPresentation,
        finalPresentation,
        clock: {
            continuityValidated: true,
            exactSuspendResumeCount: 1,
            postResumeResetCount: 1,
            elapsedBacklog: false,
        },
        nativeFocus: nativeFocusInvariant(
            object(nativeInput, "suspended"),
            object(nativeInput, "resumed"),
            readmission,
            number(launch, "processId"),
        ),
    };
}
