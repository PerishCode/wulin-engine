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
