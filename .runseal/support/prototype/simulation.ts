import { JUMP_VELOCITY_DELTA_Q16 } from "./jump.ts";

export type ExpectedCommand = {
    deltaXQ9: number;
    deltaZQ9: number;
    stepUpLimitQ16: number;
    initialVelocityDeltaQ16: number;
    groundedAfterBatch: boolean;
    animationClip: number;
    yawQ16: number;
};

export const STATIONARY_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: 0,
    stepUpLimitQ16: 32_768,
    initialVelocityDeltaQ16: 0,
    groundedAfterBatch: true,
    animationClip: 0,
    yawQ16: 0,
};
export const FORWARD_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: -32,
    stepUpLimitQ16: 32_768,
    initialVelocityDeltaQ16: 0,
    groundedAfterBatch: true,
    animationClip: 1,
    yawQ16: 49_152,
};
export const RUN_FORWARD_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: -64,
    stepUpLimitQ16: 32_768,
    initialVelocityDeltaQ16: 0,
    groundedAfterBatch: true,
    animationClip: 2,
    yawQ16: 49_152,
};
export const JUMP_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: 0,
    stepUpLimitQ16: 32_768,
    initialVelocityDeltaQ16: JUMP_VELOCITY_DELTA_Q16,
    groundedAfterBatch: false,
    animationClip: 0,
    yawQ16: 0,
};
