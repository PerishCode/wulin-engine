import {
    assertStopped,
    event,
    fail,
    type Json,
    lifecycle,
    number,
    object,
    openSources,
    publish,
    rejectedEvent,
    same,
    startClean,
    status,
    target,
} from "../canonical-runtime.ts";

const REVISION = "exact-fixed-terrain-body-motion-v1";
const HALF_HEIGHT = 65_536;
const ACCELERATION = -180;

type MotionState = {
    center_height_numerator: number;
    step_velocity_q16: number;
};

function payload(base: [number, number], state: MotionState, acceleration = ACCELERATION): Json {
    return {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: -3968,
        local_z_q9: -3968,
        center_height_numerator: state.center_height_numerator,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: state.step_velocity_q16,
        step_acceleration_q16: acceleration,
    };
}

function requireRejection(value: Json, label: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("terrain_motion_failed: ")
    ) fail(`${label} returned the wrong terrain-motion rejection`);
}

async function stepMotion(
    base: [number, number],
    state: MotionState,
    label: string,
): Promise<{ state: MotionState; step: Json }> {
    const response = await event("canonical.terrain.body.step", payload(base, state));
    if (
        response.revision !== REVISION || response.perStepAllocationBytes !== 0 ||
        response.sourceReadCount !== 0 || response.gpuCopyCount !== 0 ||
        response.gpuReadbackCount !== 0 || response.fenceWaitCount !== 0 ||
        response.synchronizationCount !== 0 || response.scheduleMutationCount !== 0 ||
        response.presentationMutationCount !== 0
    ) fail(`${label} performed work outside one fixed CPU step`);
    const step = object(response, "step");
    const output = object(step, "output");
    const outputBody = object(output, "body");
    const input = object(step, "input");
    const inputBody = object(input, "body");
    if (
        number(step, "heightDenominator") !== 65_536 ||
        number(step, "stepsPerSecond") !== 60 ||
        number(step, "stepAccelerationQ16") !== ACCELERATION ||
        number(inputBody, "centerHeightNumerator") !== state.center_height_numerator ||
        number(input, "stepVelocityQ16") !== state.step_velocity_q16 ||
        number(outputBody, "halfHeightNumerator") !== HALF_HEIGHT
    ) fail(`${label} returned an invalid fixed terrain-body step`);
    return {
        state: {
            center_height_numerator: number(outputBody, "centerHeightNumerator"),
            step_velocity_q16: number(output, "stepVelocityQ16"),
        },
        step,
    };
}

async function runSequence(
    base: [number, number],
    initial: MotionState,
    intervals: number[],
    label: string,
): Promise<Json> {
    const started = performance.now();
    let state = initial;
    const batches: Json[] = [];
    const steps: Json[] = [];
    const presentationBefore = await event("canonical.time.status");
    for (const elapsed_nanoseconds of intervals) {
        const advanceResponse = await event("simulation.advance", { elapsed_nanoseconds });
        const advance = object(advanceResponse, "advance");
        const stepCount = number(advance, "stepCount");
        if (stepCount < 0 || stepCount > 8) fail(`${label} schedule batch was outside 0..=8`);
        batches.push(advance);
    }
    const scheduleBeforeSteps = await event("simulation.status");
    for (const advance of batches) {
        const stepCount = number(advance, "stepCount");
        for (let index = 0; index < stepCount; index += 1) {
            const result = await stepMotion(base, state, `${label} step ${steps.length + 1}`);
            state = result.state;
            steps.push(result.step);
        }
    }
    same(
        await event("simulation.status"),
        scheduleBeforeSteps,
        `${label} body-step schedule independence`,
    );
    same(
        await event("canonical.time.status"),
        presentationBefore,
        `${label} presentation independence`,
    );
    return {
        intervals,
        batches,
        steps,
        finalState: state,
        stepSha256: await sha256(steps),
        finalStateSha256: await sha256(state),
        elapsedMilliseconds: performance.now() - started,
    };
}

function analyzeSequence(value: Json, terrainHeight: number, label: string): Json {
    const steps = value.steps as Json[];
    if (steps.length !== 60) fail(`${label} did not execute exactly 60 due steps`);
    let firstGroundedTick: number | undefined;
    let groundedCount = 0;
    let correctionCount = 0;
    let maximumCenter = Number.MIN_SAFE_INTEGER;
    const apexTicks: number[] = [];
    for (let index = 0; index < steps.length; index += 1) {
        const step = steps[index];
        const output = object(step, "output");
        const outputBody = object(output, "body");
        const center = number(outputBody, "centerHeightNumerator");
        if (center > maximumCenter) {
            maximumCenter = center;
            apexTicks.length = 0;
            apexTicks.push(index + 1);
        } else if (center === maximumCenter) {
            apexTicks.push(index + 1);
        }
        if (step.grounded === true) {
            groundedCount += 1;
            firstGroundedTick ??= index + 1;
        }
        if (number(object(step, "contact"), "correctionNumerator") > 0) correctionCount += 1;
    }
    const finalState = object(value, "finalState");
    if (
        firstGroundedTick !== 19 || groundedCount !== 42 || correctionCount !== 41 ||
        JSON.stringify(apexTicks) !== "[9,10]" ||
        number(finalState, "center_height_numerator") !== terrainHeight + HALF_HEIGHT ||
        number(finalState, "step_velocity_q16") !== 0
    ) fail(`${label} jump/landing sequence diverged`);
    return { firstGroundedTick, groundedCount, correctionCount, maximumCenter, apexTicks };
}

async function sha256(value: unknown): Promise<string> {
    const bytes = new TextEncoder().encode(JSON.stringify(value));
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function terrainMotionGates(
    terrainPath: string,
    objectPath: string,
    base: [number, number],
): Promise<Json> {
    console.log("==> exact fixed terrain-body motion gates");
    await startClean();
    await event("workbench.pause");
    const unavailable = await rejectedEvent(
        "canonical.terrain.body.step",
        payload(base, {
            center_height_numerator: HALF_HEIGHT,
            step_velocity_q16: 0,
        }),
    );
    requireRejection(unavailable, "pre-publication motion");
    await openSources(terrainPath, objectPath);
    const publication = await publish(target(base));
    const terrain = await event("canonical.terrain.height", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: -3968,
        local_z_q9: -3968,
    });
    const terrainHeight = number(object(terrain, "height"), "heightNumerator");

    const malformed = await rejectedEvent("canonical.terrain.body.step", {
        ...payload(base, {
            center_height_numerator: terrainHeight + HALF_HEIGHT,
            step_velocity_q16: 0,
        }),
        step_velocity_q16: "invalid",
    });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("malformed terrain motion returned the wrong rejection");
    }
    const invalidShape = await rejectedEvent("canonical.terrain.body.step", {
        ...payload(base, {
            center_height_numerator: terrainHeight,
            step_velocity_q16: 0,
        }),
        half_height_numerator: 0,
    });
    const velocityOverflow = await rejectedEvent(
        "canonical.terrain.body.step",
        payload(
            base,
            {
                center_height_numerator: terrainHeight,
                step_velocity_q16: 2_147_483_647,
            },
            1,
        ),
    );
    const positionOverflow = await rejectedEvent(
        "canonical.terrain.body.step",
        payload(
            base,
            { center_height_numerator: 2_147_483_647, step_velocity_q16: 1 },
            0,
        ),
    );
    let positiveTerrain: Json | undefined;
    for (const local of [-4032, -3904, 0, 4032]) {
        const candidate = await event("canonical.terrain.height", {
            region_x: base[0],
            region_z: base[1],
            local_x_q9: local,
            local_z_q9: local,
        });
        if (number(object(candidate, "height"), "heightNumerator") > 0) {
            positiveTerrain = candidate;
            break;
        }
    }
    if (!positiveTerrain) fail("controlled terrain omitted a positive motion-overflow sample");
    const positivePosition = object(positiveTerrain, "position");
    const unrepresentableContact = await rejectedEvent("canonical.terrain.body.step", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: number(positivePosition, "localXQ9"),
        local_z_q9: number(positivePosition, "localZQ9"),
        center_height_numerator: 2_147_483_647,
        half_height_numerator: 2_147_483_647,
        step_velocity_q16: 0,
        step_acceleration_q16: 0,
    });
    for (
        const [label, value] of [
            ["invalid body shape", invalidShape],
            ["velocity overflow", velocityOverflow],
            ["position overflow", positionOverflow],
            ["unrepresentable contact", unrepresentableContact],
        ] as const
    ) requireRejection(value, label);

    const initial = {
        center_height_numerator: terrainHeight + HALF_HEIGHT,
        step_velocity_q16: 1_800,
    };
    const coarse = await runSequence(base, initial, Array(8).fill(125_000_000), "coarse");
    const coarseAnalysis = analyzeSequence(coarse, terrainHeight, "coarse");
    const coarseProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(coarseProcess);

    await lifecycle("start");
    await openSources(terrainPath, objectPath);
    await publish(target(base));
    const nominalIntervals = [
        ...Array(20).fill(16_666_666),
        ...Array(40).fill(16_666_667),
    ];
    const nominal = await runSequence(base, initial, nominalIntervals, "nominal");
    const nominalAnalysis = analyzeSequence(nominal, terrainHeight, "nominal");
    if (
        coarse.stepSha256 !== nominal.stepSha256 ||
        coarse.finalStateSha256 !== nominal.finalStateSha256
    ) {
        fail("terrain motion changed across equal schedule partitions");
    }
    same(coarseAnalysis, nominalAnalysis, "terrain motion partition analysis");
    const nominalProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(nominalProcess);
    await lifecycle("start");
    const cleanSchedule = await event("simulation.status");
    if (cleanSchedule.tick !== 0 || cleanSchedule.successfulAdvanceCount !== 0) {
        fail("clean terrain-motion process retained simulation state");
    }
    return {
        unavailable,
        publication,
        terrain,
        malformed,
        invalidShape,
        velocityOverflow,
        positionOverflow,
        unrepresentableContact,
        coarse,
        coarseAnalysis,
        nominal,
        nominalAnalysis,
        coarseProcess,
        nominalProcess,
        cleanSchedule,
    };
}
