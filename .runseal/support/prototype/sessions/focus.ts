import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";

function nativeFocusInvariant(suspended: Json, resumed: Json, processId: number): Json {
    if (
        suspended.schema !== "prototype-native-window-action-v4" ||
        suspended.action !== "suspend" ||
        suspended.processId !== processId ||
        suspended.activated !== true ||
        suspended.closeRequested !== false ||
        suspended.requiredVisible !== true ||
        suspended.windowWasVisible !== true ||
        JSON.stringify(suspended.keys) !==
            JSON.stringify([{ key: "W", virtualKey: 87, down: true }]) ||
        JSON.stringify(suspended.messages) !==
            JSON.stringify(["WM_SETFOCUS", "WM_KEYDOWN:W", "WM_KILLFOCUS"]) ||
        suspended.atomicBatch !== true ||
        number(suspended, "batchThreadId") <= 0 ||
        number(suspended, "batchSpanMilliseconds") !== 0 ||
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
        JSON.stringify(resumed.messages) !== JSON.stringify(["WM_SETFOCUS"])
    ) fail("prototype native focus-discontinuity evidence diverged");
    return {
        exactProcessWindow: true,
        suspendedMessages: suspended.messages,
        resumedMessages: resumed.messages,
        atomicWindowThreadBatch: {
            threadId: suspended.batchThreadId,
            spanMilliseconds: suspended.batchSpanMilliseconds,
        },
        synthesizedFocusState: false,
    };
}

export function focusSessionInvariant(launch: Json, session: Json): Json {
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    same(finalActor, readyActor, "prototype focus-discontinuity actor state");

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

    const postReadiness = object(launch, "postReadinessInput");
    return {
        ...session,
        actorStateUnchanged: true,
        clock: {
            ready: readyClock,
            final: finalClock,
            exactSuspendResumeCount: 1,
            postResumeResetCount: 1,
            elapsedBacklog: false,
        },
        nativeFocus: nativeFocusInvariant(
            object(postReadiness, "suspended"),
            object(postReadiness, "resumed"),
            number(launch, "processId"),
        ),
    };
}
