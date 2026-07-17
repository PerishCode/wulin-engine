import type { Json } from "../../canonical-runtime.ts";
import type { PreparedPrototypeWindowAction, PrototypeKeyTransition } from "./mod.ts";
import {
    postPrototypeKeys,
    postPrototypeWindowAction,
    preparePrototypeWindowAction,
} from "./mod.ts";

export async function pressPrototypeCameraClockwise(processId: number | null): Promise<Json> {
    return await postPrototypeKeys(processId, [{ key: "E", virtualKey: 0x45 }], true);
}

export async function pressPrototypeJump(processId: number | null): Promise<Json> {
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

export async function releaseOpposedRun(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [{ key: "S", virtualKey: 0x53, down: false }],
        true,
        "input",
        [0],
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
    | "diagonal-run"
    | "diagonal-walk"
    | "forward"
    | "jump"
    | "opposed-run"
    | "run-forward"
    | "run-release"
    | "run-repress";

export async function prepareStartupInput(
    input?: StartupInput,
): Promise<PreparedPrototypeWindowAction | null> {
    const request = startupInputRequest(input);
    if (request === null) return null;
    return await preparePrototypeWindowAction(
        null,
        request.keys,
        request.requireVisible,
        "input",
        request.delaysBeforeKeysMilliseconds,
        request.exitAfterLastMilliseconds,
        request.atomicPrefixLength === request.keys.length,
        request.atomicPrefixLength,
    );
}

type StartupInputRequest = {
    keys: PrototypeKeyTransition[];
    requireVisible: boolean;
    delaysBeforeKeysMilliseconds?: number[];
    exitAfterLastMilliseconds?: number;
    atomicPrefixLength: number;
};

function startupInputRequest(input?: StartupInput): StartupInputRequest | null {
    switch (input) {
        case "camera-clockwise":
            return {
                keys: [{ key: "E", virtualKey: 0x45, down: true }],
                requireVisible: true,
                atomicPrefixLength: 1,
            };
        case "camera-forward":
            return {
                keys: [
                    { key: "E", virtualKey: 0x45, down: true },
                    { key: "W", virtualKey: 0x57, down: true },
                ],
                requireVisible: true,
                atomicPrefixLength: 2,
            };
        case "diagonal-walk":
            return {
                keys: [
                    { key: "W", virtualKey: 0x57, down: true },
                    { key: "A", virtualKey: 0x41, down: true },
                ],
                requireVisible: true,
                delaysBeforeKeysMilliseconds: [0, 0],
                exitAfterLastMilliseconds: 200,
                atomicPrefixLength: 2,
            };
        case "diagonal-run":
            return {
                keys: [
                    { key: "Shift", virtualKey: 0x10, down: true },
                    { key: "W", virtualKey: 0x57, down: true },
                    { key: "A", virtualKey: 0x41, down: true },
                ],
                requireVisible: true,
                delaysBeforeKeysMilliseconds: [0, 0, 0],
                exitAfterLastMilliseconds: 200,
                atomicPrefixLength: 3,
            };
        case "forward":
            return {
                keys: [{ key: "W", virtualKey: 0x57, down: true }],
                requireVisible: false,
                atomicPrefixLength: 1,
            };
        case "jump":
            return {
                keys: [{ key: "Space", virtualKey: 0x20, down: true }],
                requireVisible: true,
                atomicPrefixLength: 1,
            };
        case "opposed-run":
            return {
                keys: [
                    { key: "Shift", virtualKey: 0x10, down: true },
                    { key: "W", virtualKey: 0x57, down: true },
                    { key: "S", virtualKey: 0x53, down: true },
                ],
                requireVisible: true,
                atomicPrefixLength: 3,
            };
        case "run-forward":
            return {
                keys: [
                    { key: "Shift", virtualKey: 0x10, down: true },
                    { key: "W", virtualKey: 0x57, down: true },
                ],
                requireVisible: true,
                atomicPrefixLength: 2,
            };
        case "run-release":
            return {
                keys: [
                    { key: "Shift", virtualKey: 0x10, down: true },
                    { key: "W", virtualKey: 0x57, down: true },
                    { key: "Shift", virtualKey: 0x10, down: false },
                ],
                requireVisible: true,
                delaysBeforeKeysMilliseconds: [0, 0, 500],
                exitAfterLastMilliseconds: 200,
                atomicPrefixLength: 2,
            };
        case "run-repress":
            return {
                keys: [
                    { key: "W", virtualKey: 0x57, down: true },
                    { key: "Shift", virtualKey: 0x10, down: true },
                ],
                requireVisible: true,
                delaysBeforeKeysMilliseconds: [0, 500],
                exitAfterLastMilliseconds: 200,
                atomicPrefixLength: 1,
            };
        case undefined:
            return null;
    }
}
