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

type SessionFlight = {
    readyActor: Json;
    finalActor: Json;
    readyBody: Json;
    finalBody: Json;
    readyClock: Json;
    finalClock: Json;
    stepCount: number;
    expectedVelocity: number;
    expectedRise: number;
};

function sessionFlightInvariant(
    launch: Json,
    label: string,
    focusDiscontinuity = false,
): SessionFlight {
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");

    same(object(finalActor, "handle"), object(readyActor, "handle"), `${label} actor handle`);
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
        number(finalBody, "halfHeightNumerator") !== number(readyBody, "halfHeightNumerator")
    ) {
        fail(
            `prototype ${label} Jump trajectory diverged: ${
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
    const discontinuityCount = focusDiscontinuity ? 1 : 0;
    const readySuspendedSamples = number(readyClock, "suspendedSampleCount");
    const finalSuspendedSamples = number(finalClock, "suspendedSampleCount");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !==
            number(readyClock, "suspendCount") + discontinuityCount ||
        number(finalClock, "resumeCount") !==
            number(readyClock, "resumeCount") + discontinuityCount ||
        number(finalClock, "resetCount") !==
            number(readyClock, "resetCount") + discontinuityCount ||
        (focusDiscontinuity
            ? finalSuspendedSamples <= readySuspendedSamples
            : finalSuspendedSamples !== readySuspendedSamples) ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail(`prototype ${label} Jump clock continuity diverged`);

    return {
        readyActor,
        finalActor,
        readyBody,
        finalBody,
        readyClock,
        finalClock,
        stepCount,
        expectedVelocity: finalVelocity,
        expectedRise,
    };
}

function nativeReadmissionInvariant(
    first: Json,
    suspended: Json,
    resumed: Json,
    second: Json,
    processId: number,
): Json {
    const intervals = second.keyPostIntervalsMilliseconds;
    if (
        first.schema !== "prototype-native-window-action-v4" ||
        first.action !== "input" ||
        first.processId !== processId ||
        first.activated !== true ||
        first.closeRequested !== false ||
        first.requiredVisible !== true ||
        first.windowWasVisible !== true ||
        JSON.stringify(first.keys) !==
            JSON.stringify([{ key: "Space", virtualKey: 32, down: true }]) ||
        JSON.stringify(first.messages) !==
            JSON.stringify(["WM_SETFOCUS", "WM_KEYDOWN:Space"]) ||
        suspended.schema !== "prototype-native-window-action-v4" ||
        suspended.action !== "suspend" ||
        suspended.processId !== processId ||
        suspended.windowHandle !== first.windowHandle ||
        suspended.activated !== true ||
        suspended.closeRequested !== false ||
        suspended.requiredVisible !== true ||
        suspended.windowWasVisible !== true ||
        JSON.stringify(suspended.keys) !==
            JSON.stringify([{ key: "Space", virtualKey: 32, down: true }]) ||
        JSON.stringify(suspended.messages) !==
            JSON.stringify(["WM_SETFOCUS", "WM_KEYDOWN:Space", "WM_KILLFOCUS"]) ||
        JSON.stringify(suspended.delaysBeforeKeysMilliseconds) !== JSON.stringify([0]) ||
        !Array.isArray(suspended.keyPostIntervalsMilliseconds) ||
        suspended.keyPostIntervalsMilliseconds.length !== 0 ||
        suspended.atomicBatch !== true ||
        number(suspended, "atomicPrefixLength") !== 1 ||
        typeof suspended.batchThreadId !== "number" ||
        !Number.isSafeInteger(suspended.batchThreadId) ||
        suspended.batchThreadId <= 0 ||
        number(suspended, "batchSpanMilliseconds") !== 0 ||
        resumed.schema !== "prototype-native-window-action-v4" ||
        resumed.action !== "resume" ||
        resumed.processId !== processId ||
        resumed.windowHandle !== first.windowHandle ||
        resumed.activated !== true ||
        resumed.closeRequested !== false ||
        resumed.requiredVisible !== true ||
        resumed.windowWasVisible !== true ||
        !Array.isArray(resumed.keys) ||
        resumed.keys.length !== 0 ||
        JSON.stringify(resumed.messages) !== JSON.stringify(["WM_SETFOCUS"]) ||
        second.schema !== "prototype-native-window-action-v4" ||
        second.action !== "input" ||
        second.processId !== processId ||
        second.windowHandle !== first.windowHandle ||
        second.activated !== true ||
        second.closeRequested !== false ||
        second.requiredVisible !== true ||
        second.windowWasVisible !== true ||
        JSON.stringify(second.keys) !== JSON.stringify([
                { key: "Space", virtualKey: 32, down: false },
                { key: "Space", virtualKey: 32, down: true },
                { key: "Escape", virtualKey: 27, down: true },
            ]) ||
        JSON.stringify(second.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYUP:Space",
                "WM_KEYDOWN:Space",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(second.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0, 100]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        typeof intervals[0] !== "number" ||
        intervals[0] < 0 ||
        intervals[0] > 50 ||
        typeof intervals[1] !== "number" ||
        intervals[1] < 100 ||
        intervals[1] > 700 ||
        second.exitAfterLastMilliseconds !== 0 ||
        second.exitIntervalMilliseconds !== null
    ) fail("prototype native Jump-readmission evidence diverged");
    return {
        exactProcessWindow: true,
        exactFirstMessageOrder: true,
        exactSuspendedMessageOrder: true,
        exactResumedMessageOrder: true,
        exactSecondMessageOrder: true,
        atomicHeldCleanup: {
            threadId: suspended.batchThreadId,
            spanMilliseconds: suspended.batchSpanMilliseconds,
        },
        secondToExitIntervalMs: intervals[1],
        heldPressBeforeFocusLoss: true,
        focusClearedHeldJump: true,
        freshPressAfterFocus: true,
        normalizedSecondPress: true,
    };
}

function nativeMidairInvariant(evidence: Json, processId: number): Json {
    const intervals = evidence.keyPostIntervalsMilliseconds;
    const keys = [
        { key: "Space", virtualKey: 32, down: true },
        { key: "Space", virtualKey: 32, down: false },
        { key: "Space", virtualKey: 32, down: true },
        { key: "W", virtualKey: 87, down: true },
    ];
    if (
        evidence.schema !== "prototype-native-window-action-v4" ||
        evidence.action !== "input" ||
        evidence.processId !== processId ||
        evidence.activated !== true ||
        evidence.closeRequested !== false ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        JSON.stringify(evidence.keys) !== JSON.stringify(keys) ||
        JSON.stringify(evidence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Space",
                "WM_KEYUP:Space",
                "WM_KEYDOWN:Space",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !==
            JSON.stringify([0, 0, 200, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 3 ||
        intervals.some((interval) => typeof interval !== "number") ||
        evidence.exitAfterLastMilliseconds !== 200 ||
        typeof evidence.exitIntervalMilliseconds !== "number"
    ) fail("prototype native midair-Jump evidence diverged");
    const firstToSecondMs = intervals[0] + intervals[1];
    const secondToExitMs = evidence.exitIntervalMilliseconds;
    if (
        firstToSecondMs < 200 ||
        firstToSecondMs > 700 ||
        secondToExitMs < 200 ||
        secondToExitMs > 700
    ) fail("prototype native midair-Jump timing diverged");
    return {
        exactProcessWindow: true,
        exactMessageOrder: true,
        firstToSecondMs,
        secondToExitMs,
        normalizedMidairPress: true,
        forwardWitness: true,
    };
}

export function jumpReadmissionInvariant(launch: Json, session: Json): Json {
    const flight = sessionFlightInvariant(launch, "readmission", true);
    same(
        object(flight.finalActor, "presentation"),
        object(flight.readyActor, "presentation"),
        "Jump readmission presentation",
    );
    same(
        object(flight.finalBody, "position"),
        object(flight.readyBody, "position"),
        "Jump readmission horizontal position",
    );
    if (
        number(flight.finalActor, "animationEpochTick") !==
            number(flight.readyActor, "animationEpochTick")
    ) fail("prototype Jump-readmission animation epoch diverged");

    const nativeInput = object(launch, "nativeInput");
    const landingLowerMs = number(
        nativeInput,
        "firstToSecondPostingLowerBoundMs",
    );
    const nativeJump = nativeReadmissionInvariant(
        object(nativeInput, "firstJump"),
        object(nativeInput, "suspended"),
        object(nativeInput, "resumed"),
        object(nativeInput, "secondJump"),
        number(launch, "processId"),
    );
    const flightIntervalMs = number(nativeJump, "secondToExitIntervalMs");
    if (landingLowerMs < 1_750 || flightIntervalMs > 700) {
        fail("prototype Jump-readmission wall-time bounds diverged");
    }

    return {
        ...session,
        firstFlightLandedBeforeReadmission: true,
        heldJumpFocusCleanup: true,
        freshJumpAfterFocusReadmitted: true,
        secondFlight: {
            stepCount: flight.stepCount,
            expectedVelocity: flight.expectedVelocity,
            expectedRise: flight.expectedRise,
            grounded: false,
        },
        wallTimeBounds: {
            firstToSecondPostingLowerBoundMs: landingLowerMs,
            secondActionToExitPostingUpperBoundMs: flightIntervalMs,
        },
        clock: {
            continuityValidated: true,
            exactFocusSuspendResumeCount: 1,
            postResumeResetCount: 1,
            elapsedBacklog: false,
        },
        nativeJump,
    };
}

export function jumpMidairInvariant(launch: Json, session: Json): Json {
    const flight = sessionFlightInvariant(launch, "midair-rejection");
    const readyPosition = object(flight.readyBody, "position");
    const finalPosition = object(flight.finalBody, "position");
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "midair-Jump actor region",
    );
    const localX = number(finalPosition, "localXQ9");
    const readyX = number(readyPosition, "localXQ9");
    const horizontalDelta = number(readyPosition, "localZQ9") -
        number(finalPosition, "localZQ9");
    const horizontalSteps = horizontalDelta / 32;
    const presentation = object(flight.finalActor, "presentation");
    if (
        localX !== readyX ||
        !Number.isInteger(horizontalSteps) ||
        horizontalSteps < 1 ||
        horizontalSteps > flight.stepCount ||
        number(presentation, "archetype") !== 7 ||
        number(presentation, "material") !== 63 ||
        number(presentation, "animation") !== 1 ||
        number(presentation, "yawQ16") !== 49_152 ||
        number(flight.finalActor, "animationEpochTick") <=
            number(flight.readyActor, "animationEpochTick")
    ) fail("prototype midair-Jump forward witness diverged");

    const nativeInput = object(launch, "nativeInput");
    const nativeJump = nativeMidairInvariant(
        object(nativeInput, "sequence"),
        number(launch, "processId"),
    );
    return {
        ...session,
        midairPressRejected: true,
        singleFlight: {
            stepCount: flight.stepCount,
            expectedVelocity: flight.expectedVelocity,
            expectedRise: flight.expectedRise,
            grounded: false,
        },
        forwardWitness: {
            horizontalSteps,
            deltaZQ9: -horizontalDelta,
            presentation: "walk",
        },
        clock: {
            continuityValidated: true,
            elapsedBacklog: false,
        },
        nativeJump,
    };
}
