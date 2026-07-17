import type { Json } from "../../canonical-runtime.ts";
import {
    holdOrbitForwardKeys,
    holdPrototypeForwardKey,
    holdRunForwardKeys,
    postInvariantObjectAction,
} from "./actions.ts";
import { postPrototypeKeys, postPrototypeWindowAction } from "./mod.ts";

export async function pressPrototypeCameraClockwise(processId: number): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "E", virtualKey: 0x45 }], true);
}

export async function pressPrototypeJump(processId: number): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "Space", virtualKey: 0x20 }], true);
}

export async function repressJumpAndExit(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Space", virtualKey: 0x20, down: false },
            { key: "Space", virtualKey: 0x20, down: true },
            { key: "Escape", virtualKey: 0x1B, down: true },
        ],
        true,
        "input",
        [0, 0, 100],
    );
}

export async function postMidairSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Space", virtualKey: 0x20, down: true },
            { key: "Space", virtualKey: 0x20, down: false },
            { key: "Space", virtualKey: 0x20, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0, 200, 0],
        200,
    );
}

export async function postCameraRepeatSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "E", virtualKey: 0x45, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0],
        200,
    );
}

export async function postCameraRepressSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "E", virtualKey: 0x45, down: false },
            { key: "E", virtualKey: 0x45, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0, 0],
        200,
        true,
    );
}

export async function postRunReleaseSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Shift", virtualKey: 0x10, down: true },
            { key: "W", virtualKey: 0x57, down: true },
            { key: "Shift", virtualKey: 0x10, down: false },
        ],
        true,
        "input",
        [0, 0, 500],
        200,
    );
}

export async function postInvalidAliasSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "OutOfRangeE", virtualKey: 0x145, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0],
        200,
    );
}

export async function postOppositeCameraSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Q", virtualKey: 0x51, down: true },
            { key: "E", virtualKey: 0x45, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0, 0],
        200,
        true,
    );
}

export async function postCounterClockwiseSequence(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Q", virtualKey: 0x51, down: true },
            { key: "W", virtualKey: 0x57, down: true },
        ],
        true,
        "input",
        [0, 0],
        200,
        true,
    );
}

export type StartupInput =
    | "camera-clockwise"
    | "camera-forward"
    | "forward"
    | "jump"
    | "object-action"
    | "run-forward"
    | "run-release";

export async function applyStartupInput(
    processId: number,
    input?: StartupInput,
): Promise<Json | null> {
    switch (input) {
        case "camera-clockwise":
            return await pressPrototypeCameraClockwise(processId);
        case "camera-forward":
            return await holdOrbitForwardKeys(processId);
        case "forward":
            return await holdPrototypeForwardKey(processId);
        case "jump":
            return await pressPrototypeJump(processId);
        case "object-action":
            return await postInvariantObjectAction(processId);
        case "run-forward":
            return await holdRunForwardKeys(processId);
        case "run-release":
            return await postRunReleaseSequence(processId);
        case undefined:
            return null;
    }
}
