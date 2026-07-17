import { fail, type Json } from "../../canonical-runtime.ts";
import { postPrototypeKeys, postPrototypeWindowAction } from "./mod.ts";

export async function holdPrototypeForwardKey(processId: number | null): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "W", virtualKey: 0x57 }], false);
}

export async function holdRunForwardKeys(processId: number | null): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [{ key: "Shift", virtualKey: 0x10 }, { key: "W", virtualKey: 0x57 }],
        true,
    );
}

export async function holdOpposedRunKeys(processId: number | null): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [
            { key: "Shift", virtualKey: 0x10 },
            { key: "W", virtualKey: 0x57 },
            { key: "S", virtualKey: 0x53 },
        ],
        true,
        true,
    );
}

export async function holdOrbitForwardKeys(processId: number | null): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [{ key: "E", virtualKey: 0x45 }, { key: "W", virtualKey: 0x57 }],
        true,
    );
}

export async function postInvariantObjectAction(processId: number | null): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [
            { key: "F", virtualKey: 0x46 },
            { key: "Enter", virtualKey: 0x0D },
        ],
        true,
        true,
    );
}

export async function pressPrototypeEscape(processId: number): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "Escape", virtualKey: 0x1B }], false);
}

export async function requestPrototypeWindowClose(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(processId, [], true, "close");
}

export function nativeWindowCloseInvariant(evidence: Json, processId: number): Json {
    if (
        evidence.schema !== "prototype-native-window-action-v3" ||
        evidence.action !== "close" ||
        evidence.processId !== processId ||
        evidence.activated !== false ||
        evidence.closeRequested !== true ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        !Array.isArray(evidence.keys) ||
        evidence.keys.length !== 0 ||
        JSON.stringify(evidence.messages) !== JSON.stringify(["WM_CLOSE"])
    ) fail("prototype native window-close evidence diverged");
    return {
        exactProcessWindow: true,
        message: "WM_CLOSE",
        directDestroy: false,
    };
}

export async function suspendWithForward(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [{ key: "W", virtualKey: 0x57, down: true }],
        true,
        "suspend",
        [],
        0,
        true,
    );
}

export async function resumePrototypeFocus(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(processId, [], true, "resume");
}

export async function postPrototypeCapacityRejection(processId: number): Promise<Json> {
    const motion = await postPrototypeWindowAction(
        processId,
        [{ key: "D", virtualKey: 0x44, down: true }],
        true,
    );
    const motionStartedAt = performance.now();
    await new Promise((resolve) => setTimeout(resolve, 250));
    const action = await postPrototypeWindowAction(
        processId,
        [
            { key: "D", virtualKey: 0x44, down: false },
            { key: "F", virtualKey: 0x46, down: false },
            { key: "F", virtualKey: 0x46, down: true },
            { key: "Enter", virtualKey: 0x0D, down: false },
            { key: "Enter", virtualKey: 0x0D, down: true },
        ],
        true,
    );
    return {
        revision: "prototype-capacity-rejection-input-v1",
        motion,
        action,
        requestedMotionHoldMilliseconds: 250,
        motionHoldMilliseconds: performance.now() - motionStartedAt,
    };
}
