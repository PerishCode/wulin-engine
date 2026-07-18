import { fail, type Json, number, object, same } from "../canonical-runtime.ts";
import { presentationInvariant } from "./presentation.ts";
import {
    BOUNDARY_RUN_HOLD_MILLISECONDS,
    gracefulExit,
    idleCompletionInvariant,
} from "./sessions/mod.ts";

export async function boundaryCompletionSession(
    executable: string,
    config: string,
): Promise<Json> {
    return await gracefulExit(
        executable,
        config,
        "prototype finite-boundary Run completion",
        "boundary-run",
    );
}

export function boundaryRunInputInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const postReadiness = object(launch, "postReadinessInput");
    const sequence = object(postReadiness, "sequence");
    const exit = object(postReadiness, "exit");
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const heldMilliseconds = number(postReadiness, "heldMilliseconds");
    if (
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.activated !== true ||
        sequence.closeRequested !== false ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !==
            JSON.stringify([
                { key: "Shift", virtualKey: 0x10, down: true },
                { key: "W", virtualKey: 0x57, down: true },
            ]) ||
        JSON.stringify(sequence.messages) !==
            JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Shift",
                "WM_KEYDOWN:W",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        intervals.some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        sequence.atomicBatch !== true ||
        number(sequence, "atomicPrefixLength") !== 2 ||
        typeof sequence.batchThreadId !== "number" ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        sequence.batchThreadId <= 0 ||
        typeof sequence.batchSpanMilliseconds !== "number" ||
        sequence.batchSpanMilliseconds < 0 ||
        sequence.batchSpanMilliseconds > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 0 ||
        sequence.exitIntervalMilliseconds !== null ||
        number(postReadiness, "minimumHoldMilliseconds") !==
            BOUNDARY_RUN_HOLD_MILLISECONDS ||
        heldMilliseconds < BOUNDARY_RUN_HOLD_MILLISECONDS ||
        heldMilliseconds > BOUNDARY_RUN_HOLD_MILLISECONDS + 15_000 ||
        exit.schema !== "prototype-native-window-action-v4" ||
        exit.action !== "input" ||
        number(exit, "processId") !== processId ||
        exit.windowHandle !== sequence.windowHandle ||
        exit.activated !== true ||
        exit.closeRequested !== false ||
        exit.requiredVisible !== false ||
        exit.windowWasVisible !== true ||
        JSON.stringify(exit.keys) !== JSON.stringify([
                { key: "Escape", virtualKey: 0x1B, down: true },
            ]) ||
        JSON.stringify(exit.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(exit.delaysBeforeKeysMilliseconds) !== JSON.stringify([0]) ||
        !Array.isArray(exit.keyPostIntervalsMilliseconds) ||
        exit.keyPostIntervalsMilliseconds.length !== 0 ||
        number(exit, "exitAfterLastMilliseconds") !== 0 ||
        exit.exitIntervalMilliseconds !== null ||
        exit.atomicBatch !== false ||
        number(exit, "atomicPrefixLength") !== 0 ||
        exit.batchThreadId !== null ||
        exit.batchSpanMilliseconds !== null
    ) fail("prototype native finite-boundary Run input evidence diverged");
    same(exit, object(launch, "exitInput"), "prototype boundary exit input");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        keyPostIntervalsMilliseconds: intervals,
        orderedMessages: sequence.messages,
        runModifierHeld: true,
        forwardHeld: true,
        minimumHoldMilliseconds: BOUNDARY_RUN_HOLD_MILLISECONDS,
        heldMilliseconds,
        delayedEscape: true,
    };
}

export function boundarySessionInvariant(launch: Json): Json {
    const completion = object(launch, "completion");
    const session = idleCompletionInvariant(launch);
    const readiness = object(launch, "readiness");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype finite-boundary actor handle",
    );
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");
    const readyPosition = object(readyBody, "position");
    const finalPosition = object(finalBody, "position");
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype finite-boundary actor region",
    );
    const finalLocalZQ9 = number(finalPosition, "localZQ9");
    if (
        number(finalPosition, "localXQ9") !== 0 ||
        finalLocalZQ9 < -4096 ||
        finalLocalZQ9 > -3648 ||
        (-finalLocalZQ9) % 64 !== 0 ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype finite-boundary final motion diverged");
    const presentation = presentationInvariant(
        object(finalActor, "presentation"),
        0,
        49_152,
        "prototype finite-boundary Survey",
    );
    const readyEpoch = number(readyActor, "animationEpochTick");
    const finalEpoch = number(finalActor, "animationEpochTick");
    if (finalEpoch <= readyEpoch || finalEpoch >= 31_002_560) {
        fail("prototype finite-boundary presentation lifetime diverged");
    }

    const readyDriver = object(readiness, "simulation_driver");
    const readyClock = object(readyDriver, "clock");
    const finalClock = object(completion, "clock");
    const frames = object(completion, "frames");
    if (
        number(launch, "exitCode") !== 0 ||
        number(frames, "liveFrameCount") <= number(readyDriver, "liveFrameCount") ||
        number(frames, "renderBlockCount") !== 0 ||
        number(frames, "activatedFrameCount") !== 0 ||
        number(frames, "rejectedFrameCount") !== 0 ||
        number(frames, "objectTargetFrameCount") !== 0 ||
        number(frames, "suppressionProjectedFrameCount") !== 0 ||
        finalClock.suspended !== false ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") ||
        number(finalClock, "suspendedSampleCount") !==
            number(readyClock, "suspendedSampleCount")
    ) fail("prototype finite-boundary completion diverged");

    return {
        ...session,
        nativeInput: boundaryRunInputInvariant(launch),
        actorIdentityStable: true,
        exactFinalBoundaryBand: {
            localXQ9: 0,
            minimumLocalZQ9: -4096,
            maximumLocalZQ9: -3648,
            stepQuantumQ9: 64,
            committedRunStepCount: -finalLocalZQ9 / 64,
        },
        stationaryAtFiniteBoundary: true,
        finalPresentation: presentation,
        surveyPresentationRetained: true,
        retainedForwardYawQ16: 49_152,
        presentationLifetimeAdvanced: true,
        liveFramesAdvanced: true,
        renderBlockCount: 0,
        clockContinuous: true,
    };
}
