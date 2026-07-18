import { fail, type Json, number, object, same } from "../canonical-runtime.ts";
import {
    BOUNDARY_SLIDE_HOLD_MILLISECONDS,
    BOUNDARY_STATIONARY_HOLD_MILLISECONDS,
} from "./input/actions.ts";
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
    const nativeInput = object(launch, "nativeInput");
    const sequence = object(nativeInput, "sequence");
    const tangentialRun = object(nativeInput, "tangentialRun");
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const tangentialIntervals = tangentialRun.keyPostIntervalsMilliseconds;
    const heldMilliseconds = number(nativeInput, "heldMilliseconds");
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
        number(nativeInput, "minimumHoldMilliseconds") !==
            BOUNDARY_RUN_HOLD_MILLISECONDS ||
        heldMilliseconds < BOUNDARY_RUN_HOLD_MILLISECONDS ||
        heldMilliseconds > BOUNDARY_RUN_HOLD_MILLISECONDS + 15_000 ||
        tangentialRun.schema !== "prototype-native-window-action-v4" ||
        tangentialRun.action !== "input" ||
        number(tangentialRun, "processId") !== processId ||
        tangentialRun.windowHandle !== sequence.windowHandle ||
        tangentialRun.activated !== true ||
        tangentialRun.closeRequested !== false ||
        tangentialRun.requiredVisible !== true ||
        tangentialRun.windowWasVisible !== true ||
        JSON.stringify(tangentialRun.keys) !== JSON.stringify([
                { key: "Shift", virtualKey: 0x10, down: true },
                { key: "W", virtualKey: 0x57, down: true },
                { key: "A", virtualKey: 0x41, down: true },
                { key: "A", virtualKey: 0x41, down: false },
                { key: "W", virtualKey: 0x57, down: false },
                { key: "Shift", virtualKey: 0x10, down: false },
            ]) ||
        JSON.stringify(tangentialRun.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Shift",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:A",
                "WM_KEYUP:A",
                "WM_KEYUP:W",
                "WM_KEYUP:Shift",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(tangentialRun.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 0, BOUNDARY_SLIDE_HOLD_MILLISECONDS, 0, 0]) ||
        !Array.isArray(tangentialIntervals) ||
        tangentialIntervals.length !== 5 ||
        tangentialIntervals.slice(0, 2).some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        typeof tangentialIntervals[2] !== "number" ||
        tangentialIntervals[2] < BOUNDARY_SLIDE_HOLD_MILLISECONDS ||
        tangentialIntervals[2] > BOUNDARY_SLIDE_HOLD_MILLISECONDS + 500 ||
        tangentialIntervals.slice(3).some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        number(tangentialRun, "exitAfterLastMilliseconds") !==
            BOUNDARY_STATIONARY_HOLD_MILLISECONDS ||
        typeof tangentialRun.exitIntervalMilliseconds !== "number" ||
        tangentialRun.exitIntervalMilliseconds < BOUNDARY_STATIONARY_HOLD_MILLISECONDS ||
        tangentialRun.exitIntervalMilliseconds >
            BOUNDARY_STATIONARY_HOLD_MILLISECONDS + 500 ||
        tangentialRun.atomicBatch !== false ||
        number(tangentialRun, "atomicPrefixLength") !== 3 ||
        !Number.isSafeInteger(tangentialRun.batchThreadId) ||
        number(tangentialRun, "batchThreadId") <= 0 ||
        number(tangentialRun, "batchSpanMilliseconds") < 0 ||
        number(tangentialRun, "batchSpanMilliseconds") > 50
    ) fail("prototype native finite-boundary Run input evidence diverged");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        initialKeyPostIntervalMilliseconds: intervals[0],
        exactInitialMessageOrder: true,
        runModifierHeld: true,
        forwardHeld: true,
        minimumHoldMilliseconds: BOUNDARY_RUN_HOLD_MILLISECONDS,
        heldMilliseconds,
        tangentialRun: {
            atomicPrefixLength: 3,
            batchThreadId: tangentialRun.batchThreadId,
            batchSpanMilliseconds: tangentialRun.batchSpanMilliseconds,
            heldStateReasserted: true,
            holdMilliseconds: tangentialIntervals[2],
            exactMessageOrder: true,
            releaseIntervalCount: 2,
        },
        forwardAndRunReleased: true,
        stationaryHoldMilliseconds: tangentialRun.exitIntervalMilliseconds,
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
    const finalLocalXQ9 = number(finalPosition, "localXQ9");
    const tangentialRunStepCount = -finalLocalXQ9 / 45;
    if (
        finalLocalXQ9 >= 0 ||
        finalLocalXQ9 % 45 !== 0 ||
        tangentialRunStepCount < 16 ||
        tangentialRunStepCount > 48 ||
        finalLocalZQ9 < -4096 ||
        finalLocalZQ9 > -3648 ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) {
        fail(
            `prototype finite-boundary final motion diverged: ${
                JSON.stringify({ finalLocalXQ9, finalLocalZQ9, tangentialRunStepCount })
            }`,
        );
    }
    const presentation = presentationInvariant(
        object(finalActor, "presentation"),
        0,
        32_768,
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
            finalLocalZQ9,
            minimumLocalZQ9: -4096,
            maximumLocalZQ9: -3648,
        },
        exactTangentialRun: {
            finalLocalXQ9,
            componentQ9: 45,
            committedRunStepCount: tangentialRunStepCount,
            maximumCoupledStepCount: 9,
            minimumTangentialOnlyStepCount: tangentialRunStepCount - 9,
        },
        blockedForwardAxisRetainedBoundaryBand: true,
        tangentialRunAdmitted: true,
        stationaryAfterTangentialRun: true,
        finalPresentation: presentation,
        surveyPresentationRetained: true,
        retainedTangentialYawQ16: 32_768,
        presentationLifetimeAdvanced: true,
        liveFramesAdvanced: true,
        renderBlockCount: 0,
        clockContinuous: true,
    };
}
