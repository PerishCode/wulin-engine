import { fail, type Json, number } from "../../canonical-runtime.ts";

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
        delayedExit: true,
        exitIntervalMilliseconds: evidence.exitIntervalMilliseconds,
    };
}
