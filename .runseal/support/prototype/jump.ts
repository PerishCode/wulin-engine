import { fail, type Json, number, object, same } from "../canonical-runtime.ts";

export const JUMP_VELOCITY_DELTA_Q16 = 4_369;

export function jumpPolicyInvariant(launch: Json, expectedGrounded: boolean): Json {
    const driver = object(object(launch, "readiness"), "jump_driver");
    if (
        driver.revision !== "live-prototype-jump-policy-v1" ||
        number(driver, "stepVelocityDeltaQ16") !== JUMP_VELOCITY_DELTA_Q16
    ) fail("prototype jump policy identity diverged");
    const status = object(driver, "status");
    if (status.pending !== false || status.grounded !== expectedGrounded) {
        fail("prototype jump policy committed status diverged");
    }
    return {
        revision: driver.revision,
        stepVelocityDeltaQ16: JUMP_VELOCITY_DELTA_Q16,
        pending: false,
        grounded: expectedGrounded,
    };
}

export function jumpMotionInvariant(initial: Json, output: Json, stepCount: number): Json {
    const initialMotion = object(initial, "motion");
    const outputMotion = object(output, "motion");
    const initialBody = object(initialMotion, "body");
    const outputBody = object(outputMotion, "body");
    const expectedVelocity = JUMP_VELOCITY_DELTA_Q16 - 179 * stepCount;
    const expectedRise = JUMP_VELOCITY_DELTA_Q16 * stepCount -
        179 * stepCount * (stepCount + 1) / 2;
    same(
        object(output, "handle"),
        object(initial, "handle"),
        "prototype jump actor handle",
    );
    same(
        object(outputBody, "position"),
        object(initialBody, "position"),
        "prototype jump horizontal position",
    );
    if (
        number(initialMotion, "stepVelocityQ16") !== 0 ||
        number(outputMotion, "stepVelocityQ16") !== expectedVelocity ||
        number(outputBody, "halfHeightNumerator") !==
            number(initialBody, "halfHeightNumerator") ||
        number(outputBody, "centerHeightNumerator") !==
            number(initialBody, "centerHeightNumerator") + expectedRise
    ) fail("prototype jump vertical trajectory diverged");
    return { stepCount, expectedVelocity, expectedRise };
}
