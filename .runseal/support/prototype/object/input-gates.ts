import { fail, type Json, number, object } from "../../canonical-runtime.ts";

export function nativeSelectionInvariant(
    evidence: Json,
    processId: number,
): Json {
    const intervals = evidence.keyPostIntervalsMilliseconds;
    if (
        evidence.schema !== "prototype-native-window-action-v4" ||
        evidence.action !== "input" ||
        number(evidence, "processId") !== processId ||
        evidence.activated !== true ||
        evidence.closeRequested !== false ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        JSON.stringify(evidence.keys) !== JSON.stringify([
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: true },
            ]) ||
        JSON.stringify(evidence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:F",
                "WM_KEYDOWN:Enter",
            ]) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        typeof intervals[0] !== "number" ||
        intervals[0] < 0 ||
        intervals[0] > 50 ||
        evidence.atomicBatch !== true ||
        number(evidence, "atomicPrefixLength") !== 2 ||
        !Number.isSafeInteger(evidence.batchThreadId) ||
        number(evidence, "batchThreadId") <= 0 ||
        number(evidence, "batchSpanMilliseconds") < 0 ||
        number(evidence, "batchSpanMilliseconds") > 50 ||
        number(evidence, "exitAfterLastMilliseconds") !== 0 ||
        evidence.exitIntervalMilliseconds !== null
    ) fail("prototype post-ready native object-selection action evidence diverged");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: evidence.batchThreadId,
        batchSpanMilliseconds: evidence.batchSpanMilliseconds,
        keyPostIntervalMilliseconds: intervals[0],
        exactMessageOrder: true,
        exitIntervalMilliseconds: evidence.exitIntervalMilliseconds,
    };
}

export function missingTargetInputInvariant(
    evidence: Json,
    processId: number,
    windowHandle: unknown,
): Json {
    if (
        evidence.schema !== "prototype-native-window-action-v4" ||
        evidence.action !== "input" ||
        number(evidence, "processId") !== processId ||
        evidence.windowHandle !== windowHandle ||
        evidence.activated !== true ||
        evidence.closeRequested !== false ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        JSON.stringify(evidence.keys) !== JSON.stringify([
                { key: "Enter", virtualKey: 13, down: true },
            ]) ||
        JSON.stringify(evidence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Enter",
            ]) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0]) ||
        !Array.isArray(evidence.keyPostIntervalsMilliseconds) ||
        evidence.keyPostIntervalsMilliseconds.length !== 0 ||
        number(evidence, "exitAfterLastMilliseconds") !== 0 ||
        evidence.exitIntervalMilliseconds !== null ||
        evidence.atomicBatch !== false ||
        number(evidence, "atomicPrefixLength") !== 0 ||
        evidence.batchThreadId !== null ||
        evidence.batchSpanMilliseconds !== null
    ) fail("prototype missing-target native input evidence diverged");
    return {
        exactProcessWindow: true,
        exactMessageOrder: true,
        noFeedbackCandidate: true,
    };
}

export function objectRecoveryInputInvariant(
    evidence: Json,
    processId: number,
    windowHandle: unknown,
): Json {
    const input = object(evidence, "input");
    const frameCompletion = object(evidence, "frameCompletion");
    const intervals = input.keyPostIntervalsMilliseconds;
    if (frameCompletion.completionObserved !== true) {
        fail(
            `prototype object recovery frame completion was not observed: ${
                JSON.stringify(frameCompletion)
            }`,
        );
    }
    if (
        evidence.revision !== "prototype-object-recovery-frame-completion-v1" ||
        input.schema !== "prototype-native-window-action-v4" ||
        input.action !== "input" ||
        number(input, "processId") !== processId ||
        input.windowHandle !== windowHandle ||
        input.activated !== true ||
        input.closeRequested !== false ||
        input.requiredVisible !== true ||
        input.windowWasVisible !== true ||
        JSON.stringify(input.keys) !== JSON.stringify([
                { key: "Enter", virtualKey: 13, down: false },
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: true },
            ]) ||
        JSON.stringify(input.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYUP:Enter",
                "WM_KEYDOWN:F",
                "WM_KEYDOWN:Enter",
            ]) ||
        JSON.stringify(input.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        intervals.some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        input.atomicBatch !== true ||
        number(input, "atomicPrefixLength") !== 3 ||
        !Number.isSafeInteger(input.batchThreadId) ||
        number(input, "batchThreadId") <= 0 ||
        number(input, "batchSpanMilliseconds") < 0 ||
        number(input, "batchSpanMilliseconds") > 50 ||
        number(input, "exitAfterLastMilliseconds") !== 0 ||
        input.exitIntervalMilliseconds !== null ||
        frameCompletion.schema !== "prototype-activated-frame-completion-v1" ||
        number(frameCompletion, "processId") !== processId ||
        frameCompletion.windowHandle !== windowHandle ||
        frameCompletion.requiredVisible !== true ||
        frameCompletion.windowWasVisible !== true ||
        frameCompletion.captureVisibility !== "temporary-topmost-noactivate" ||
        frameCompletion.captureMethod !== "print-window-client-full-content-v1" ||
        frameCompletion.colorRule !== "activated-green-v1" ||
        frameCompletion.captureOwner !== null ||
        number(frameCompletion, "minimumActivatedPixelDelta") !== 64 ||
        number(frameCompletion, "completionTolerancePixels") !== 16 ||
        number(frameCompletion, "activatedPixelPeak") <
            number(frameCompletion, "baselineActivatedPixelCount") + 64 ||
        number(frameCompletion, "activatedSampleCount") < 1 ||
        number(frameCompletion, "completionPixelCount") >
            number(frameCompletion, "baselineActivatedPixelCount") + 16 ||
        number(frameCompletion, "completionClearSampleCount") !== 2 ||
        number(frameCompletion, "sampleCount") <
            number(frameCompletion, "activatedSampleCount") + 2 ||
        number(frameCompletion, "elapsedMilliseconds") <= 0 ||
        number(frameCompletion, "elapsedMilliseconds") >= 10_000 ||
        number(frameCompletion, "timeoutMilliseconds") !== 10_000 ||
        JSON.stringify(frameCompletion.messages) !== JSON.stringify(["WM_KEYDOWN:Escape"])
    ) fail("prototype object recovery native input evidence diverged");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: input.batchThreadId,
        batchSpanMilliseconds: input.batchSpanMilliseconds,
        keyPostIntervalCount: intervals.length,
        exactMessageOrder: true,
        releasedMissingTargetKey: true,
        frameCompletion: {
            colorRule: frameCompletion.colorRule,
            captureMethod: frameCompletion.captureMethod,
            completionObserved: frameCompletion.completionObserved,
            baselineActivatedPixelCount: frameCompletion.baselineActivatedPixelCount,
            activatedPixelPeak: frameCompletion.activatedPixelPeak,
            activatedSampleCount: frameCompletion.activatedSampleCount,
            completionPixelCount: frameCompletion.completionPixelCount,
            completionClearSampleCount: frameCompletion.completionClearSampleCount,
            sampleCount: frameCompletion.sampleCount,
            elapsedMilliseconds: frameCompletion.elapsedMilliseconds,
            boundedTimeoutMilliseconds: frameCompletion.timeoutMilliseconds,
        },
    };
}

export function outsideRadiusInputInvariant(
    evidence: Json,
    processId: number,
    windowHandle: unknown,
): Json {
    const initialRejection = object(evidence, "initialRejection");
    const motion = object(evidence, "motion");
    const outsideRadius = object(evidence, "outsideRadius");
    const intervals = motion.keyPostIntervalsMilliseconds;
    if (
        evidence.revision !== "prototype-object-outside-radius-input-v1" ||
        number(evidence, "requestedRejectionHoldMilliseconds") !== 250 ||
        number(evidence, "rejectionHoldMilliseconds") < 250 ||
        number(evidence, "requestedMotionHoldMilliseconds") !== 500 ||
        initialRejection.windowHandle !== windowHandle ||
        motion.schema !== "prototype-native-window-action-v4" ||
        motion.action !== "input" ||
        number(motion, "processId") !== processId ||
        motion.windowHandle !== windowHandle ||
        motion.activated !== true ||
        motion.closeRequested !== false ||
        motion.requiredVisible !== true ||
        motion.windowWasVisible !== true ||
        JSON.stringify(motion.keys) !== JSON.stringify([
                { key: "F", virtualKey: 70, down: false },
                { key: "Enter", virtualKey: 13, down: false },
                { key: "D", virtualKey: 68, down: true },
                { key: "D", virtualKey: 68, down: false },
            ]) ||
        JSON.stringify(motion.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYUP:F",
                "WM_KEYUP:Enter",
                "WM_KEYDOWN:D",
                "WM_KEYUP:D",
            ]) ||
        JSON.stringify(motion.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 0, 500]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 3 ||
        intervals.slice(0, 2).some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        typeof intervals[2] !== "number" ||
        intervals[2] < 500 ||
        intervals[2] > 1_000 ||
        motion.atomicBatch !== false ||
        number(motion, "atomicPrefixLength") !== 3 ||
        !Number.isSafeInteger(motion.batchThreadId) ||
        number(motion, "batchThreadId") <= 0 ||
        number(motion, "batchSpanMilliseconds") < 0 ||
        number(motion, "batchSpanMilliseconds") > 50 ||
        number(motion, "exitAfterLastMilliseconds") !== 0 ||
        motion.exitIntervalMilliseconds !== null ||
        outsideRadius.schema !== "prototype-native-window-action-v4" ||
        outsideRadius.action !== "input" ||
        number(outsideRadius, "processId") !== processId ||
        outsideRadius.windowHandle !== windowHandle ||
        outsideRadius.activated !== true ||
        outsideRadius.closeRequested !== false ||
        outsideRadius.requiredVisible !== true ||
        outsideRadius.windowWasVisible !== true ||
        JSON.stringify(outsideRadius.keys) !== JSON.stringify([
                { key: "Enter", virtualKey: 13, down: true },
            ]) ||
        JSON.stringify(outsideRadius.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Enter",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(outsideRadius.delaysBeforeKeysMilliseconds) !== JSON.stringify([0]) ||
        !Array.isArray(outsideRadius.keyPostIntervalsMilliseconds) ||
        outsideRadius.keyPostIntervalsMilliseconds.length !== 0 ||
        outsideRadius.atomicBatch !== true ||
        number(outsideRadius, "atomicPrefixLength") !== 1 ||
        !Number.isSafeInteger(outsideRadius.batchThreadId) ||
        number(outsideRadius, "batchThreadId") <= 0 ||
        number(outsideRadius, "batchSpanMilliseconds") < 0 ||
        number(outsideRadius, "batchSpanMilliseconds") > 50 ||
        number(outsideRadius, "exitAfterLastMilliseconds") !== 250 ||
        number(outsideRadius, "exitIntervalMilliseconds") < 250 ||
        number(outsideRadius, "exitIntervalMilliseconds") > 750 ||
        number(initialRejection, "batchThreadId") !==
            number(motion, "batchThreadId") ||
        number(initialRejection, "batchThreadId") !==
            number(outsideRadius, "batchThreadId")
    ) fail("prototype native object outside-radius input evidence diverged");
    return {
        revision: evidence.revision,
        rejectionHoldMilliseconds: evidence.rejectionHoldMilliseconds,
        motionHoldMilliseconds: intervals[2],
        batchThreadId: motion.batchThreadId,
        batchSpanMilliseconds: motion.batchSpanMilliseconds,
        keyPostIntervalCount: intervals.length,
        exactMotionMessageOrder: true,
        outsideRadiusBatchSpanMilliseconds: outsideRadius.batchSpanMilliseconds,
        exactOutsideRadiusMessageOrder: true,
        exitIntervalMilliseconds: outsideRadius.exitIntervalMilliseconds,
    };
}
