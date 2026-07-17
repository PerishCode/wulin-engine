import { fail, type Json, number, object, same } from "../canonical-runtime.ts";
import { nativeJumpReadmissionInvariant } from "./input.ts";

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

export function jumpReadmissionInvariant(launch: Json, session: Json): Json {
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");

    same(object(finalActor, "handle"), object(readyActor, "handle"), "Jump readmission handle");
    same(
        object(finalActor, "presentation"),
        object(readyActor, "presentation"),
        "Jump readmission presentation",
    );
    same(
        object(finalBody, "position"),
        object(readyBody, "position"),
        "Jump readmission horizontal position",
    );
    const finalVelocity = number(finalMotion, "stepVelocityQ16");
    const velocityDifference = JUMP_VELOCITY_DELTA_Q16 - finalVelocity;
    const stepCount = velocityDifference / 179;
    const expectedRise = JUMP_VELOCITY_DELTA_Q16 * stepCount -
        179 * stepCount * (stepCount + 1) / 2;
    if (
        number(readyMotion, "stepVelocityQ16") !== 0 ||
        !Number.isInteger(stepCount) ||
        stepCount < 1 ||
        stepCount > 43 ||
        number(finalBody, "centerHeightNumerator") !==
            number(readyBody, "centerHeightNumerator") + expectedRise ||
        number(finalBody, "centerHeightNumerator") <=
            number(readyBody, "centerHeightNumerator") ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalActor, "animationEpochTick") !==
            number(readyActor, "animationEpochTick")
    ) {
        fail(
            `prototype second Jump trajectory diverged: ${
                JSON.stringify({
                    stepCount,
                    finalVelocity,
                    expectedRise,
                    readyCenter: number(readyBody, "centerHeightNumerator"),
                    finalCenter: number(finalBody, "centerHeightNumerator"),
                })
            }`,
        );
    }

    const readyClock = object(object(readiness, "simulation_driver"), "clock");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype Jump-readmission clock continuity diverged");

    const postReadiness = object(launch, "postReadinessInput");
    const landingLowerMs = number(
        postReadiness,
        "firstToSecondPostingLowerBoundMs",
    );
    const nativeJump = nativeJumpReadmissionInvariant(
        object(postReadiness, "firstJump"),
        object(postReadiness, "secondJump"),
        number(launch, "processId"),
    );
    const flightIntervalMs = number(nativeJump, "secondToExitIntervalMs");
    if (landingLowerMs < 1_250 || flightIntervalMs > 700) {
        fail("prototype Jump-readmission wall-time bounds diverged");
    }

    return {
        ...session,
        firstFlightLandedBeforeReadmission: true,
        secondFlight: {
            stepCount,
            expectedVelocity: finalVelocity,
            expectedRise,
            grounded: false,
        },
        wallTimeBounds: {
            firstToSecondPostingLowerBoundMs: landingLowerMs,
            secondActionToExitPostingUpperBoundMs: flightIntervalMs,
        },
        clock: {
            ready: readyClock,
            final: finalClock,
            elapsedBacklog: false,
        },
        nativeJump,
    };
}
