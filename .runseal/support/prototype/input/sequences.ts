import type { Json } from "../../canonical-runtime.ts";
import { postPrototypeKeys, postPrototypeWindowAction } from "./mod.ts";

export async function pressPrototypeCameraClockwise(processId: number): Promise<Json> {
    return await postPrototypeKeys(
        processId,
        [{ key: "E", virtualKey: 0x45 }],
        true,
        true,
    );
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

export async function postDiagonalWalk(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "W", virtualKey: 0x57, down: true },
            { key: "A", virtualKey: 0x41, down: true },
            { key: "W", virtualKey: 0x57, down: false },
            { key: "A", virtualKey: 0x41, down: false },
        ],
        true,
        "input",
        [0, 0, 250, 250],
        250,
        false,
        2,
    );
}

export async function postDiagonalRun(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Shift", virtualKey: 0x10, down: true },
            { key: "W", virtualKey: 0x57, down: true },
            { key: "A", virtualKey: 0x41, down: true },
            { key: "W", virtualKey: 0x57, down: false },
            { key: "A", virtualKey: 0x41, down: false },
        ],
        true,
        "input",
        [0, 0, 0, 250, 250],
        250,
        false,
        3,
    );
}

export async function postRunRelease(processId: number): Promise<Json> {
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
        false,
        2,
    );
}

export async function postRunRepress(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "W", virtualKey: 0x57, down: true },
            { key: "Shift", virtualKey: 0x10, down: true },
        ],
        true,
        "input",
        [0, 500],
        200,
        false,
        1,
    );
}

export async function postForwardRelease(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "W", virtualKey: 0x57, down: true },
            { key: "W", virtualKey: 0x57, down: false },
        ],
        true,
        "input",
        [0, 250],
        250,
        false,
        1,
    );
}

export async function postOpposedRun(processId: number): Promise<Json> {
    return await postPrototypeWindowAction(
        processId,
        [
            { key: "Shift", virtualKey: 0x10, down: true },
            { key: "W", virtualKey: 0x57, down: true },
            { key: "S", virtualKey: 0x53, down: true },
        ],
        true,
        "input",
        [0, 0, 0],
        0,
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
