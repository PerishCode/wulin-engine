import { fail, type Json } from "../../canonical-runtime.ts";
import { postPrototypeKeys, postPrototypeWindowAction } from "./mod.ts";

export async function holdPrototypeBoundaryRun(processId: number): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [
            { key: "Shift", virtualKey: 0x10 },
            { key: "W", virtualKey: 0x57 },
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
        evidence.schema !== "prototype-native-window-action-v4" ||
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

export async function suspendWithActionBatch(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Space", virtualKey: 0x20, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
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

export async function postObjectActionExit(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "F", virtualKey: 0x46, down: true },
            { key: "Enter", virtualKey: 0x0D, down: true },
        ],
        true,
        "input",
        [0, 0],
        200,
        true,
    );
}

export async function postConsumptionCapacity(
    processId: number,
): Promise<Json> {
    const consumption = await postPrototypeWindowAction(
        processId,
        [
            { key: "F", virtualKey: 0x46, down: true },
            { key: "Enter", virtualKey: 0x0D, down: true },
        ],
        true,
        "input",
        [0, 0],
        0,
        true,
    );
    const consumptionStartedAt = performance.now();
    await new Promise((resolve) => setTimeout(resolve, 250));
    const consumptionHoldMilliseconds = performance.now() - consumptionStartedAt;
    const capacity = await postPrototypeCapacityRejection(processId);
    return {
        revision: "prototype-post-ready-consumption-capacity-input-v1",
        consumption,
        capacity,
        requestedConsumptionHoldMilliseconds: 250,
        consumptionHoldMilliseconds,
    };
}
