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
        orderedMessages: evidence.messages,
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
        orderedMessages: evidence.messages,
        noFeedbackCandidate: true,
    };
}

export function objectRecoveryInputInvariant(
    evidence: Json,
    processId: number,
    windowHandle: unknown,
): Json {
    const intervals = evidence.keyPostIntervalsMilliseconds;
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
                { key: "Enter", virtualKey: 13, down: false },
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: true },
            ]) ||
        JSON.stringify(evidence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYUP:Enter",
                "WM_KEYDOWN:F",
                "WM_KEYDOWN:Enter",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        intervals.some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        evidence.atomicBatch !== true ||
        number(evidence, "atomicPrefixLength") !== 3 ||
        !Number.isSafeInteger(evidence.batchThreadId) ||
        number(evidence, "batchThreadId") <= 0 ||
        number(evidence, "batchSpanMilliseconds") < 0 ||
        number(evidence, "batchSpanMilliseconds") > 50 ||
        number(evidence, "exitAfterLastMilliseconds") !== 250 ||
        number(evidence, "exitIntervalMilliseconds") < 250 ||
        number(evidence, "exitIntervalMilliseconds") > 750
    ) fail("prototype object recovery native input evidence diverged");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: evidence.batchThreadId,
        batchSpanMilliseconds: evidence.batchSpanMilliseconds,
        keyPostIntervalsMilliseconds: intervals,
        orderedMessages: evidence.messages,
        releasedMissingTargetKey: true,
        exitIntervalMilliseconds: evidence.exitIntervalMilliseconds,
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
        keyPostIntervalsMilliseconds: intervals,
        motionMessages: motion.messages,
        outsideRadiusBatchSpanMilliseconds: outsideRadius.batchSpanMilliseconds,
        outsideRadiusMessages: outsideRadius.messages,
        exitIntervalMilliseconds: outsideRadius.exitIntervalMilliseconds,
    };
}
