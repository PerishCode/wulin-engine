import { fail, type Json } from "../../canonical-runtime.ts";
import type {
    PreparedPrototypeWindowAction,
    PrototypeKeyTransition,
    PrototypeWindowAction,
} from "./mod.ts";

const WINDOW_CLASS = wideNullTerminated("WulinEnginePrototypeWindow");
const WINDOW_TITLE = wideNullTerminated("Wulin Engine Prototype");
const WINDOW_SEARCH_TIMEOUT_MILLISECONDS = 20_000;
const THREAD_SUSPEND_RESUME = 0x0002;
const FAILURE_U32 = 0xffff_ffff;
const WM_SETFOCUS = 0x0007;
const WM_KILLFOCUS = 0x0008;
const WM_KEYDOWN = 0x0100;
const WM_KEYUP = 0x0101;
const WM_CLOSE = 0x0010;

const USER32_SYMBOLS = {
    FindWindowW: {
        parameters: ["buffer", "buffer"],
        result: "pointer",
    },
    GetWindowThreadProcessId: {
        parameters: ["pointer", "buffer"],
        result: "u32",
    },
    IsWindowVisible: {
        parameters: ["pointer"],
        result: "i32",
    },
    PostMessageW: {
        parameters: ["pointer", "u32", "usize", "isize"],
        result: "i32",
    },
} as const;

const KERNEL32_SYMBOLS = {
    OpenThread: {
        parameters: ["u32", "i32", "u32"],
        result: "pointer",
    },
    SuspendThread: {
        parameters: ["pointer"],
        result: "u32",
    },
    ResumeThread: {
        parameters: ["pointer"],
        result: "u32",
    },
    CloseHandle: {
        parameters: ["pointer"],
        result: "i32",
    },
    GetLastError: {
        parameters: [],
        result: "u32",
    },
} as const;

type ExpectedWindowAction = {
    action: PrototypeWindowAction;
    processId: number;
    nativeKeys: PrototypeKeyTransition[];
    requireVisible: boolean;
    keyDelays: number[];
    exitAfterLastMilliseconds: number;
    atomicBatch: boolean;
    atomicPrefixLength: number;
    expectedMessages: string[];
};

type NativeLibraries = {
    user32: Deno.DynamicLibrary<typeof USER32_SYMBOLS>;
    kernel32: Deno.DynamicLibrary<typeof KERNEL32_SYMBOLS>;
};

type WindowHandle = Deno.PointerValue;

let nativeLibraries: NativeLibraries | undefined;

export async function startPreparedWindowAction(
    expected: ExpectedWindowAction,
): Promise<PreparedPrototypeWindowAction> {
    const libraries = loadNativeLibraries();
    return {
        evidence: executePrototypeWindowAction(libraries, expected)
            .then((evidence) => validatePrototypeWindowAction(evidence, expected)),
    };
}

export function closeNativeTransport(): void {
    const libraries = nativeLibraries;
    nativeLibraries = undefined;
    if (!libraries) return;
    libraries.user32.close();
    libraries.kernel32.close();
}

export function wideNullTerminated(value: string): Uint16Array {
    const result = new Uint16Array(value.length + 1);
    for (let index = 0; index < value.length; index += 1) {
        result[index] = value.charCodeAt(index);
    }
    return result;
}

function loadNativeLibraries(): NativeLibraries {
    if (nativeLibraries) return nativeLibraries;
    if (Deno.build.os !== "windows") {
        fail(`prototype native input requires Windows, received ${Deno.build.os}`);
    }
    const user32 = Deno.dlopen("user32.dll", USER32_SYMBOLS);
    try {
        const kernel32 = Deno.dlopen("kernel32.dll", KERNEL32_SYMBOLS);
        nativeLibraries = { user32, kernel32 };
        return nativeLibraries;
    } catch (error) {
        user32.close();
        throw error;
    }
}

async function executePrototypeWindowAction(
    libraries: NativeLibraries,
    expected: ExpectedWindowAction,
): Promise<Json> {
    const window = await exactPrototypeWindow(libraries, expected);
    const windowWasVisible = libraries.user32.symbols.IsWindowVisible(window) !== 0;
    const messages: string[] = [];
    const keyPostIntervalsMilliseconds: number[] = [];
    let previousKeyAt: number | undefined;
    let lastKeyAt: number | undefined;
    let batchThreadId: number | null = null;
    let batchSpanMilliseconds: number | null = null;
    let exitIntervalMilliseconds: number | null = null;

    if (expected.atomicPrefixLength > 0) {
        const batch = postAtomicInputBatch(
            libraries,
            window,
            expected.nativeKeys.slice(0, expected.atomicPrefixLength),
            expected.action === "suspend" && expected.atomicBatch,
        );
        batchThreadId = batch.threadId;
        messages.push("WM_SETFOCUS");
        for (let index = 0; index < batch.keyTimes.length; index += 1) {
            const keyAt = batch.keyTimes[index];
            if (previousKeyAt !== undefined) {
                keyPostIntervalsMilliseconds.push(keyAt - previousKeyAt);
            }
            previousKeyAt = keyAt;
            lastKeyAt = keyAt;
            messages.push(keyMessage(expected.nativeKeys[index]));
        }
        if (expected.action === "suspend" && expected.atomicBatch) {
            messages.push("WM_KILLFOCUS");
        }
        batchSpanMilliseconds = batch.keyTimes.at(-1)! - batch.keyTimes[0];
    } else if (expected.action === "input" || expected.action === "suspend") {
        postMessage(libraries, window, WM_SETFOCUS, 0n, "focus activation");
        messages.push("WM_SETFOCUS");
    }

    for (
        let index = expected.atomicPrefixLength;
        index < expected.nativeKeys.length;
        index += 1
    ) {
        const delayBase = lastKeyAt ?? performance.now();
        await waitUntil(delayBase + expected.keyDelays[index]);
        const key = expected.nativeKeys[index];
        postMessage(
            libraries,
            window,
            key.down ? WM_KEYDOWN : WM_KEYUP,
            BigInt(key.virtualKey),
            `${key.key} key ${key.down ? "down" : "up"}`,
        );
        const keyAt = performance.now();
        if (previousKeyAt !== undefined) {
            keyPostIntervalsMilliseconds.push(keyAt - previousKeyAt);
        }
        previousKeyAt = keyAt;
        lastKeyAt = keyAt;
        messages.push(keyMessage(key));
    }

    if (expected.exitAfterLastMilliseconds > 0) {
        if (lastKeyAt === undefined) fail("prototype delayed exit has no preceding key");
        await waitUntil(lastKeyAt + expected.exitAfterLastMilliseconds);
        postMessage(libraries, window, WM_KEYDOWN, 0x1bn, "delayed Escape key down");
        exitIntervalMilliseconds = performance.now() - lastKeyAt;
        messages.push("WM_KEYDOWN:Escape");
    }

    if (expected.action === "suspend" && !expected.atomicBatch) {
        postMessage(libraries, window, WM_KILLFOCUS, 0n, "focus suspension");
        messages.push("WM_KILLFOCUS");
    } else if (expected.action === "resume") {
        postMessage(libraries, window, WM_SETFOCUS, 0n, "focus resume");
        messages.push("WM_SETFOCUS");
    } else if (expected.action === "close") {
        postMessage(libraries, window, WM_CLOSE, 0n, "window close");
        messages.push("WM_CLOSE");
    }

    return {
        schema: "prototype-native-window-action-v4",
        action: expected.action,
        processId: expected.processId,
        windowHandle: Deno.UnsafePointer.value(window).toString(),
        activated: expected.action !== "close",
        closeRequested: expected.action === "close",
        requiredVisible: expected.requireVisible,
        windowWasVisible,
        keys: expected.nativeKeys,
        messages,
        delaysBeforeKeysMilliseconds: expected.keyDelays,
        keyPostIntervalsMilliseconds,
        exitAfterLastMilliseconds: expected.exitAfterLastMilliseconds,
        exitIntervalMilliseconds,
        atomicPrefixLength: expected.atomicPrefixLength,
        atomicBatch: expected.atomicBatch,
        batchThreadId,
        batchSpanMilliseconds,
    };
}

async function exactPrototypeWindow(
    libraries: NativeLibraries,
    expected: ExpectedWindowAction,
): Promise<WindowHandle> {
    const deadline = performance.now() + WINDOW_SEARCH_TIMEOUT_MILLISECONDS;
    while (performance.now() < deadline) {
        const window = libraries.user32.symbols.FindWindowW(WINDOW_CLASS, WINDOW_TITLE);
        if (window !== null) {
            const processId = new Uint32Array(1);
            libraries.user32.symbols.GetWindowThreadProcessId(window, processId);
            const visible = libraries.user32.symbols.IsWindowVisible(window) !== 0;
            if (
                processId[0] === expected.processId &&
                (!expected.requireVisible || visible)
            ) {
                return window;
            }
        }
        await new Promise((resolve) => setTimeout(resolve, 1));
    }
    fail(`prototype window for process ${expected.processId} was not found`);
}

function postAtomicInputBatch(
    libraries: NativeLibraries,
    window: WindowHandle,
    keys: PrototypeKeyTransition[],
    suspendAfterInput: boolean,
): { threadId: number; keyTimes: number[] } {
    if (keys.length < 1) fail("prototype atomic input batch shape diverged");
    const processId = new Uint32Array(1);
    const threadId = libraries.user32.symbols.GetWindowThreadProcessId(window, processId);
    const thread = libraries.kernel32.symbols.OpenThread(
        THREAD_SUSPEND_RESUME,
        0,
        threadId,
    );
    if (thread === null) {
        fail(`opening prototype window thread failed with Win32 error ${lastError(libraries)}`);
    }
    if (libraries.kernel32.symbols.SuspendThread(thread) === FAILURE_U32) {
        libraries.kernel32.symbols.CloseHandle(thread);
        fail(`suspending prototype window thread failed with Win32 error ${lastError(libraries)}`);
    }
    try {
        postMessage(libraries, window, WM_SETFOCUS, 0n, "focus activation");
        const keyTimes = keys.map((key) => {
            postMessage(
                libraries,
                window,
                key.down ? WM_KEYDOWN : WM_KEYUP,
                BigInt(key.virtualKey),
                `atomic ${key.key} key ${key.down ? "down" : "up"}`,
            );
            return performance.now();
        });
        if (suspendAfterInput) {
            postMessage(libraries, window, WM_KILLFOCUS, 0n, "atomic focus suspension");
        }
        return { threadId, keyTimes };
    } finally {
        const resumeResult = libraries.kernel32.symbols.ResumeThread(thread);
        const closeResult = libraries.kernel32.symbols.CloseHandle(thread);
        if (resumeResult === FAILURE_U32 || closeResult === 0) {
            fail(
                `restoring prototype window thread failed with Win32 error ${lastError(libraries)}`,
            );
        }
    }
}

function postMessage(
    libraries: NativeLibraries,
    window: WindowHandle,
    message: number,
    wParam: bigint,
    label: string,
): void {
    const lParam = message === WM_KEYDOWN || message === WM_KEYUP ? 1n : 0n;
    if (libraries.user32.symbols.PostMessageW(window, message, wParam, lParam) === 0) {
        fail(`posting prototype ${label} failed with Win32 error ${lastError(libraries)}`);
    }
}

function lastError(libraries: NativeLibraries): number {
    return libraries.kernel32.symbols.GetLastError();
}

async function waitUntil(deadline: number): Promise<void> {
    while (true) {
        const remaining = deadline - performance.now();
        if (remaining <= 0) return;
        if (remaining > 2) {
            await new Promise((resolve) => setTimeout(resolve, Math.max(1, remaining - 1)));
        }
    }
}

function keyMessage(key: PrototypeKeyTransition): string {
    return `${key.down ? "WM_KEYDOWN" : "WM_KEYUP"}:${key.key}`;
}

function validatePrototypeWindowAction(
    evidence: Json,
    expected: ExpectedWindowAction,
): Json {
    if (
        evidence.schema !== "prototype-native-window-action-v4" ||
        evidence.action !== expected.action ||
        typeof evidence.processId !== "number" ||
        !Number.isSafeInteger(evidence.processId) ||
        evidence.processId <= 0 ||
        evidence.processId !== expected.processId ||
        evidence.activated !== (expected.action !== "close") ||
        evidence.closeRequested !== (expected.action === "close") ||
        evidence.requiredVisible !== expected.requireVisible ||
        (expected.requireVisible && evidence.windowWasVisible !== true) ||
        JSON.stringify(evidence.keys) !== JSON.stringify(expected.nativeKeys) ||
        JSON.stringify(evidence.messages) !== JSON.stringify(expected.expectedMessages) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify(expected.keyDelays) ||
        !Array.isArray(evidence.keyPostIntervalsMilliseconds) ||
        evidence.keyPostIntervalsMilliseconds.length !==
            Math.max(0, expected.nativeKeys.length - 1) ||
        evidence.keyPostIntervalsMilliseconds.some((interval, index) =>
            typeof interval !== "number" ||
            interval < expected.keyDelays[index + 1]
        ) ||
        evidence.exitAfterLastMilliseconds !== expected.exitAfterLastMilliseconds ||
        evidence.atomicPrefixLength !== expected.atomicPrefixLength ||
        evidence.atomicBatch !== expected.atomicBatch ||
        (expected.atomicPrefixLength > 0
            ? typeof evidence.batchThreadId !== "number" ||
                !Number.isSafeInteger(evidence.batchThreadId) ||
                evidence.batchThreadId <= 0 ||
                typeof evidence.batchSpanMilliseconds !== "number" ||
                evidence.batchSpanMilliseconds < 0 ||
                evidence.batchSpanMilliseconds > 50
            : evidence.batchThreadId !== null || evidence.batchSpanMilliseconds !== null) ||
        (expected.exitAfterLastMilliseconds === 0
            ? evidence.exitIntervalMilliseconds !== null
            : typeof evidence.exitIntervalMilliseconds !== "number" ||
                evidence.exitIntervalMilliseconds < expected.exitAfterLastMilliseconds)
    ) {
        fail(
            `prototype native window action evidence diverged: ${
                JSON.stringify({ expected, evidence })
            }`,
        );
    }
    return evidence;
}
