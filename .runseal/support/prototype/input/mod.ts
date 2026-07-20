import { fail, type Json } from "../../canonical-runtime.ts";
import { startPreparedWindowAction } from "./prepared.ts";

export type PrototypeKey = {
    key:
        | "A"
        | "D"
        | "E"
        | "Enter"
        | "Escape"
        | "F"
        | "OutOfRangeE"
        | "Q"
        | "S"
        | "Shift"
        | "Space"
        | "W";
    virtualKey: number;
};

export type PrototypeKeyTransition = PrototypeKey & {
    down: boolean;
};

export type PrototypeWindowAction = "close" | "input" | "resume" | "suspend";

export type PreparedPrototypeWindowAction = {
    evidence: Promise<Json>;
};

export async function preparePrototypeWindowAction(
    processId: number,
    keys: PrototypeKeyTransition[],
    requireVisible: boolean,
    action: PrototypeWindowAction = "input",
    delaysBeforeKeysMilliseconds: number[] = [],
    exitAfterLastMilliseconds = 0,
    atomicBatch = false,
    atomicPrefixLength = 0,
): Promise<PreparedPrototypeWindowAction> {
    if (!Number.isSafeInteger(processId) || processId <= 0) {
        fail(`prototype native input received invalid process id ${processId}`);
    }
    const requiresKeys = action === "input" || action === "suspend";
    if (requiresKeys === (keys.length === 0)) {
        fail(`prototype native ${action} action key shape diverged`);
    }
    const keyDelays = delaysBeforeKeysMilliseconds.length === 0
        ? keys.map(() => 0)
        : delaysBeforeKeysMilliseconds;
    if (
        keyDelays.length !== keys.length ||
        keyDelays.some((delay) =>
            !Number.isSafeInteger(delay) ||
            delay < 0 ||
            delay > 1_000
        ) ||
        !Number.isSafeInteger(exitAfterLastMilliseconds) ||
        exitAfterLastMilliseconds < 0 ||
        exitAfterLastMilliseconds > 1_000 ||
        (exitAfterLastMilliseconds > 0 && action !== "input") ||
        !Number.isSafeInteger(atomicPrefixLength) ||
        atomicPrefixLength < 0 ||
        atomicPrefixLength > keys.length ||
        ((atomicBatch || atomicPrefixLength > 0) &&
            ((action !== "input" && action !== "suspend") ||
                keys.length < 1 ||
                (atomicBatch &&
                    atomicPrefixLength !== 0 &&
                    atomicPrefixLength !== keys.length) ||
                keyDelays
                    .slice(0, atomicBatch ? keys.length : atomicPrefixLength)
                    .some((delay) => delay !== 0)))
    ) fail(`prototype native ${action} action delay diverged`);
    const resolvedAtomicPrefixLength = atomicBatch ? keys.length : atomicPrefixLength;
    const expectedMessages = requiresKeys
        ? [
            "WM_SETFOCUS",
            ...keys.map(({ key, down }) => `${down ? "WM_KEYDOWN" : "WM_KEYUP"}:${key}`),
            ...(action === "suspend" ? ["WM_KILLFOCUS"] : []),
            ...(exitAfterLastMilliseconds > 0 ? ["WM_KEYDOWN:Escape"] : []),
        ]
        : [action === "resume" ? "WM_SETFOCUS" : "WM_CLOSE"];
    return await startPreparedWindowAction({
        action,
        processId,
        nativeKeys: keys,
        requireVisible,
        keyDelays,
        exitAfterLastMilliseconds,
        atomicBatch,
        atomicPrefixLength: resolvedAtomicPrefixLength,
        expectedMessages,
    });
}

export async function postPrototypeWindowAction(
    processId: number,
    keys: PrototypeKeyTransition[],
    requireVisible: boolean,
    action: PrototypeWindowAction = "input",
    delaysBeforeKeysMilliseconds: number[] = [],
    exitAfterLastMilliseconds = 0,
    atomicBatch = false,
    atomicPrefixLength = 0,
): Promise<Json> {
    const prepared = await preparePrototypeWindowAction(
        processId,
        keys,
        requireVisible,
        action,
        delaysBeforeKeysMilliseconds,
        exitAfterLastMilliseconds,
        atomicBatch,
        atomicPrefixLength,
    );
    return await prepared.evidence;
}

export async function postPrototypeKeys(
    processId: number,
    keys: PrototypeKey[],
    requireVisible: boolean,
    atomicBatch = false,
): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        keys.map((key) => ({ ...key, down: true })),
        requireVisible,
        "input",
        [],
        0,
        atomicBatch,
    );
}
