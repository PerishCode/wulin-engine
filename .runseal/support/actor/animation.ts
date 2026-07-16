import {
    array,
    type Coord,
    event,
    fail,
    type Json,
    number,
    object,
    probe,
    same,
} from "../canonical-runtime.ts";

const CLOCK_FRAME_PERIOD = 31_002_560;
const SAMPLE_COUNT = 64;
const TIME_UNITS_PER_FRAME = 80;
const IMPORTED_CLIP_DURATION_UNITS = [16_400, 3_400, 5_560];
const HALF_HEIGHT_Q16 = 196_608;
const STEP_NANOSECONDS = 16_666_667;

export async function actorAnimationEpochGates(base: Coord): Promise<Json> {
    console.log("==> actor-local animation epoch gates");
    await event("canonical.time.pause");
    await event("canonical.time.set", { tick: 42 });

    const survey = object(await event("actor.spawn", await groundedActor(base)), "actor");
    const generation = number(object(survey, "handle"), "generation");
    requireEpoch(survey, 42, "Survey spawn");
    const surveyStart = phaseInvariant(await probe(), survey, 0, "Survey spawn");

    await event("canonical.time.step", { ticks: 4 });
    const surveyProgress = phaseInvariant(await probe(), survey, 1, "Survey progress");

    const walkResponse = await event(
        "simulation.actor.advance",
        simulationRequest(generation, 0, 1, 0),
    );
    const walkTransition = requireAdvance(walkResponse, "Survey-to-Walk transition");
    const walkInput = object(walkTransition, "input");
    const walk = object(walkTransition, "output");
    requireEpoch(walkInput, 42, "Survey-to-Walk input");
    requireEpoch(walk, 46, "Survey-to-Walk output");
    const walkStart = phaseInvariant(
        await event("canonical.probe"),
        walk,
        0,
        "Walk transition",
    );

    await event("canonical.time.step", { ticks: 1 });
    const walkProgress = phaseInvariant(
        await event("canonical.probe"),
        walk,
        1,
        "Walk progress",
    );

    const yawResponse = await event(
        "simulation.actor.advance",
        simulationRequest(generation, 1, 1, 32_768),
    );
    const yawTransition = requireAdvance(yawResponse, "same-clip yaw transition");
    const yawActor = object(yawTransition, "output");
    requireEpoch(yawActor, 46, "same-clip yaw output");
    const yawProgress = phaseInvariant(
        await event("canonical.probe"),
        yawActor,
        1,
        "same-clip yaw progress",
    );

    const fractionalResponse = await event(
        "simulation.actor.advance",
        simulationRequest(generation, 1, 0, 0, 1),
    );
    const fractional = object(
        object(fractionalResponse, "actorSimulationAdvance"),
        "actor",
    );
    if (
        fractionalResponse.revision !== "runtime-actor-simulation-v6" ||
        fractionalResponse.outcome !== "advanced" ||
        number(fractionalResponse, "preparedStepCount") !== 0 ||
        number(fractionalResponse, "presentationMutationCount") !== 0
    ) fail("fractional clip command returned the wrong outcome");
    if (fractional.lastStepGrounded !== null) {
        fail("fractional clip command published a grounded witness");
    }
    same(object(fractional, "input"), object(fractional, "output"), "fractional epoch rollback");
    same(
        object(await event("actor.read", { generation }), "actor"),
        yawActor,
        "fractional stored actor rollback",
    );

    await event("actor.despawn", { generation });
    await event("canonical.time.resume");
    return {
        surveyStart,
        surveyProgress,
        walkTransition: { inputEpoch: 42, outputEpoch: 46 },
        walkStart,
        walkProgress,
        sameClipEpoch: 46,
        yawProgress,
        fractionalEpochRollback: true,
        groundedWitness: {
            walkTransition: true,
            sameClipYaw: true,
            fractional: null,
        },
    };
}

async function groundedActor(base: Coord): Promise<Json> {
    const query = await event("canonical.terrain.height", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
    });
    return {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
        center_height_numerator: number(object(query, "height"), "heightNumerator") +
            HALF_HEIGHT_Q16,
        half_height_numerator: HALF_HEIGHT_Q16,
        step_velocity_q16: 0,
        archetype: 7,
        material: 63,
        yaw_q16: 0,
        animation: 0,
    };
}

function simulationRequest(
    generation: number,
    deltaXQ9: number,
    clip: number,
    yawQ16: number,
    elapsedNanoseconds = STEP_NANOSECONDS,
): Json {
    return {
        generation,
        elapsed_nanoseconds: elapsedNanoseconds,
        delta_x_q9: deltaXQ9,
        delta_z_q9: 0,
        step_up_limit_q16: 2_147_483_647,
        initial_step_velocity_delta_q16: 0,
        step_acceleration_q16: 0,
        archetype: 7,
        material: 63,
        yaw_q16: yawQ16,
        animation: clip,
    };
}

function requireAdvance(value: Json, label: string): Json {
    if (
        value.revision !== "runtime-actor-simulation-v6" || value.outcome !== "advanced" ||
        number(value, "preparedStepCount") !== 1 ||
        number(value, "terrainQueryCount") !== 1 ||
        number(value, "scheduleCommitCount") !== 1 || number(value, "actorCommitCount") !== 1
    ) fail(`${label} did not commit one actor step`);
    const actor = object(object(value, "actorSimulationAdvance"), "actor");
    if (actor.lastStepGrounded !== true) {
        fail(`${label} did not publish exact grounded contact`);
    }
    return actor;
}

function requireEpoch(actor: Json, expected: number, label: string): void {
    if (number(actor, "animationEpochTick") !== expected) {
        fail(`${label} animation epoch diverged`);
    }
}

function phaseInvariant(value: Json, actor: Json, expectedPhase: number, label: string): Json {
    const surface = object(value, "surface");
    const skeletal = object(surface, "skeletal");
    const globalTick = number(object(skeletal, "settings"), "timeTick");
    const candidate = object(object(surface, "occlusion"), "actorCandidate");
    const words = array(candidate, "recordWords");
    if (words.length !== 14 || words.some((word) => typeof word !== "number")) {
        fail(`${label} actor GPU record is absent`);
    }
    const storedAnimation = number(object(actor, "presentation"), "animation");
    const resolvedAnimation = words[13] as number;
    const clip = storedAnimation % 256;
    const authoredOffset = Math.floor(storedAnimation / 256) % 256;
    const variation = Math.floor(storedAnimation / 65_536);
    const resolvedClip = resolvedAnimation % 256;
    const effectiveOffset = Math.floor(resolvedAnimation / 256) % 256;
    const resolvedVariation = Math.floor(resolvedAnimation / 65_536);
    if (clip >= IMPORTED_CLIP_DURATION_UNITS.length || resolvedClip !== clip) {
        fail(`${label} resolved the wrong imported clip`);
    }
    if (resolvedVariation !== variation) fail(`${label} changed animation variation`);
    const epoch = number(actor, "animationEpochTick");
    const elapsedTick = (globalTick + CLOCK_FRAME_PERIOD - epoch) % CLOCK_FRAME_PERIOD;
    const globalPhase = phaseAtFrame(clip, globalTick);
    const localPhase = phaseAtFrame(clip, elapsedTick);
    const gpuPhase = (effectiveOffset + globalPhase) % SAMPLE_COUNT;
    const authoredLocalPhase = (authoredOffset + localPhase) % SAMPLE_COUNT;
    if (gpuPhase !== expectedPhase || authoredLocalPhase !== expectedPhase) {
        fail(`${label} actor-local GPU phase diverged`);
    }
    return {
        globalTick,
        epoch,
        elapsedTick,
        clip,
        authoredOffset,
        effectiveOffset,
        globalPhase,
        localPhase,
        gpuPhase,
        recordBytes: number(candidate, "uploadRecordBytes"),
        exactFieldMismatchCount: number(candidate, "exactFieldMismatchCount"),
    };
}

function phaseAtFrame(clip: number, frame: number): number {
    const duration = IMPORTED_CLIP_DURATION_UNITS[clip];
    return Math.floor(((frame * TIME_UNITS_PER_FRAME) % duration) * SAMPLE_COUNT / duration);
}
