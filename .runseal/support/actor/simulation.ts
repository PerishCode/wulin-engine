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

const REVISION = "runtime-actor-simulation-v6";
const HALF_HEIGHT = 65_536;
const MAX_ELAPSED = 125_000_000;
const I32_MAX = 2_147_483_647;

type ActorPayload = {
    region_x: number;
    region_z: number;
    local_x_q9: number;
    local_z_q9: number;
    center_height_numerator: number;
    half_height_numerator: number;
    step_velocity_q16: number;
    archetype: number;
    material: number;
    yaw_q16: number;
    animation: number;
};

const PRESENTATION = { archetype: 7, material: 63, yaw_q16: 0, animation: 1 };

function request(
    generation: number,
    elapsed: number,
    deltaXQ9: number,
    deltaZQ9: number,
    limit: number,
    initialVelocityDelta: number,
    acceleration: number,
): Json {
    return {
        generation,
        elapsed_nanoseconds: elapsed,
        delta_x_q9: deltaXQ9,
        delta_z_q9: deltaZQ9,
        step_up_limit_q16: limit,
        initial_step_velocity_delta_q16: initialVelocityDelta,
        step_acceleration_q16: acceleration,
        ...PRESENTATION,
    };
}

function requireFailure(value: Json, label: string, detail: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("actor_simulation_advance_failed: ") ||
        !value.error.includes(detail)
    ) fail(`${label} returned the wrong simulation-actor rejection: ${JSON.stringify(value)}`);
}

async function retiredControlGate(): Promise<Json[]> {
    const requests: [string, Json][] = [
        ["simulation.advance", { elapsed_nanoseconds: 1 }],
        ["simulation.probe", {}],
        ["canonical.terrain.body.spawn", {}],
        ["canonical.terrain.body.read", { generation: 1 }],
        ["canonical.terrain.body.despawn", { generation: 1 }],
        ["simulation.terrain.body.advance", request(1, 1, 0, 0, 0, 0, 0)],
        [
            "canonical.terrain.body.retained.advance",
            {
                generation: 1,
                delta_x_q9: 0,
                delta_z_q9: 0,
                step_up_limit_q16: 0,
                step_acceleration_q16: 0,
            },
        ],
        [
            "canonical.terrain.body.retained.batch",
            {
                generation: 1,
                step_count: 1,
                delta_x_q9: 0,
                delta_z_q9: 0,
                step_up_limit_q16: 0,
                step_acceleration_q16: 0,
            },
        ],
    ];
    const evidence: Json[] = [];
    for (const [verb, payload] of requests) {
        const rejected = await rejectedEvent(verb, payload);
        if (typeof rejected.error !== "string" || !rejected.error.startsWith("unknown_event: ")) {
            fail(`${verb} did not fail through the retired-control contract`);
        }
        evidence.push({ verb, rejected });
    }
    return evidence;
}

function requireAdvance(
    response: Json,
    elapsed: number,
    startTick: number,
    stepCount: number,
    queryCount: number,
    label: string,
): Json {
    if (
        response.revision !== REVISION || response.outcome !== "advanced" ||
        number(response, "preparedStepCount") !== stepCount ||
        number(response, "terrainQueryCount") !== queryCount ||
        response.perOperationAllocationBytes !== 0 || response.sourceReadCount !== 0 ||
        response.gpuCopyCount !== 0 || response.gpuReadbackCount !== 0 ||
        response.fenceWaitCount !== 0 || response.synchronizationCount !== 0 ||
        response.scheduleCommitCount !== 1 || response.actorCommitCount !== 1 ||
        response.presentationMutationCount !== 0 || response.frameCount !== 0 ||
        response.rendererWorkCount !== 0
    ) fail(`${label} performed work outside one CPU dual commit`);
    const value = object(response, "actorSimulationAdvance");
    const simulation = object(value, "simulation");
    const actor = object(value, "actor");
    if (
        number(simulation, "elapsedNanoseconds") !== elapsed ||
        number(simulation, "startTick") !== startTick ||
        number(simulation, "stepCount") !== stepCount ||
        number(simulation, "endTick") !== startTick + stepCount ||
        number(actor, "stepCount") !== stepCount ||
        number(actor, "terrainQueryCount") !== queryCount ||
        number(object(object(actor, "input"), "handle"), "generation") !==
            number(object(object(actor, "output"), "handle"), "generation")
    ) fail(`${label} schedule/actor evidence diverged`);
    if (
        (stepCount === 0 && actor.lastStepGrounded !== null) ||
        (stepCount !== 0 && typeof actor.lastStepGrounded !== "boolean")
    ) fail(`${label} last-step grounded witness diverged`);
    same(
        object(object(actor, "input"), "presentation"),
        object(object(actor, "output"), "presentation"),
        `${label} actor presentation`,
    );
    return value;
}

async function spawn(payload: ActorPayload): Promise<Json> {
    return object(await event("actor.spawn", payload), "actor");
}

async function read(generation: number): Promise<Json> {
    return object(await event("actor.read", { generation }), "actor");
}

async function despawn(generation: number, expected: Json): Promise<void> {
    const value = object(await event("actor.despawn", { generation }), "actor");
    same(value, expected, "simulation-actor despawn");
}

async function height(regionX: number, regionZ: number, localXQ9: number, localZQ9: number) {
    const response = await event("canonical.terrain.height", {
        region_x: regionX,
        region_z: regionZ,
        local_x_q9: localXQ9,
        local_z_q9: localZQ9,
    });
    return number(object(response, "height"), "heightNumerator");
}

function requireStatus(
    value: Json,
    tick: number,
    remainder: number,
    advances: number,
    emitted: number,
    label: string,
): void {
    if (
        number(value, "tick") !== tick || number(value, "remainderNumerator") !== remainder ||
        number(value, "successfulAdvanceCount") !== advances ||
        number(value, "emittedStepCount") !== emitted
    ) fail(`${label} simulation status diverged: ${JSON.stringify(value)}`);
}

async function prepublication(base: [number, number]): Promise<Json> {
    await startClean();
    await event("workbench.pause");
    const retiredControls = await retiredControlGate();
    const initialSimulation = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");
    const empty = await rejectedEvent(
        "simulation.actor.advance",
        request(1, 1, 0, 0, 0, 0, 0),
    );
    requireFailure(empty, "empty dual advance", "no runtime actor is live");
    const malformed = await rejectedEvent("simulation.actor.advance", {
        ...request(1, 1, 0, 0, 0, 0, 0),
        elapsed_nanoseconds: -1,
    });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("negative dual elapsed returned the wrong rejection");
    }
    const actor: ActorPayload = {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
        center_height_numerator: HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
        ...PRESENTATION,
    };
    const stored = await spawn(actor);
    const stale = await rejectedEvent(
        "simulation.actor.advance",
        request(2, 1, 0, 0, 0, 0, 0),
    );
    requireFailure(stale, "stale dual advance", "handle is stale");
    const oversized = await rejectedEvent(
        "simulation.actor.advance",
        request(1, MAX_ELAPSED + 1, 0, 0, 0, 0, 0),
    );
    requireFailure(oversized, "oversized dual advance", "must be in [0, 125000000]");
    same(await event("simulation.status"), initialSimulation, "dual validation schedule rollback");
    same(await read(1), stored, "dual validation actor rollback");
    const fractional = requireAdvance(
        await event("simulation.actor.advance", request(1, 1, 17, -19, 0, 0, 0)),
        1,
        0,
        0,
        0,
        "fractional dual advance",
    );
    same(object(object(fractional, "actor"), "output"), stored, "fractional actor identity");
    requireStatus(await event("simulation.status"), 0, 60, 1, 0, "fractional dual commit");
    same(await event("canonical.time.status"), initialPresentation, "dual time isolation");
    return { retiredControls, empty, malformed, stale, oversized, fractional };
}

async function startPublished(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    await startClean();
    await event("workbench.pause");
    await openSources(terrain, objects);
    await publish(target(base));
    return await event("canonical.time.status");
}

async function groundActor(base: [number, number]): Promise<ActorPayload> {
    const ground = await height(base[0], base[1], -3904, -3968);
    return {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: -3904,
        local_z_q9: -3968,
        center_height_numerator: ground + HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
        ...PRESENTATION,
    };
}

async function advanceSequence(intervals: number[]): Promise<Json> {
    const advances: Json[] = [];
    let startTick = 0;
    let terrainQueryCount = 0;
    for (const elapsed of intervals) {
        const response = await event(
            "simulation.actor.advance",
            request(1, elapsed, 128, 0, I32_MAX, 0, -1092),
        );
        const value = object(response, "actorSimulationAdvance");
        const stepCount = number(object(value, "simulation"), "stepCount");
        const queryCount = number(object(value, "actor"), "terrainQueryCount");
        requireAdvance(response, elapsed, startTick, stepCount, queryCount, "dual sequence");
        if (queryCount !== stepCount) fail("dual sequence query/step count diverged");
        startTick += stepCount;
        terrainQueryCount += queryCount;
        advances.push(value);
    }
    return { advances, terrainQueryCount, actor: await read(1) };
}

async function coarseRun(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    const presentation = await startPublished(terrain, objects, base);
    const actor = await groundActor(base);
    await spawn(actor);
    const result = await advanceSequence(Array(8).fill(MAX_ELAPSED));
    requireStatus(await event("simulation.status"), 60, 0, 8, 60, "coarse dual second");
    if (result.terrainQueryCount !== 60) fail("coarse dual query count diverged");
    same(await event("canonical.time.status"), presentation, "coarse dual time isolation");
    return { actorInput: actor, ...result };
}

async function nominalRun(
    terrain: string,
    objects: string,
    base: [number, number],
    expected: Json,
): Promise<Json> {
    const presentation = await startPublished(terrain, objects, base);
    await spawn(expected.actorInput as ActorPayload);
    const intervals = [...Array(20).fill(16_666_666), ...Array(40).fill(16_666_667)];
    const result = await advanceSequence(intervals);
    requireStatus(await event("simulation.status"), 60, 0, 60, 60, "nominal dual second");
    if (result.terrainQueryCount !== 60) fail("nominal dual query count diverged");
    same(result.actor, expected.actor, "dual partition actor output");
    await despawn(1, result.actor as Json);

    const edgeGround = await height(base[0], base[1], 0, 0);
    const edgeActor: ActorPayload = {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
        center_height_numerator: edgeGround + HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
        ...PRESENTATION,
    };
    const edgeStored = await spawn(edgeActor);
    const beforeFailure = await event("simulation.status");
    const failed = await rejectedEvent(
        "simulation.actor.advance",
        request(2, MAX_ELAPSED, 8192, 0, I32_MAX, 0, 0),
    );
    requireFailure(failed, "dual mid-batch snapshot", "batch step 3 of 7 failed");
    same(await event("simulation.status"), beforeFailure, "dual query schedule rollback");
    same(await read(2), edgeStored, "dual query actor rollback");
    await despawn(2, edgeStored);

    const overflowActor: ActorPayload = {
        ...edgeActor,
        step_velocity_q16: I32_MAX,
    };
    const overflowStored = await spawn(overflowActor);
    const overflow = await rejectedEvent(
        "simulation.actor.advance",
        request(3, MAX_ELAPSED, 0, 0, 0, 0, 1),
    );
    requireFailure(
        overflow,
        "dual arithmetic",
        "vertical velocity is outside the signed 32-bit Q16 range",
    );
    same(await event("simulation.status"), beforeFailure, "dual arithmetic schedule rollback");
    same(await read(3), overflowStored, "dual arithmetic actor rollback");
    same(await event("canonical.time.status"), presentation, "nominal dual time isolation");
    return { intervals, ...result, failed, edgeStored, overflow, overflowStored };
}

async function sha256(value: unknown): Promise<string> {
    const digest = await crypto.subtle.digest(
        "SHA-256",
        new TextEncoder().encode(JSON.stringify(value)),
    );
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function simulationActorGates(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    console.log("==> transactional simulation-actor advance gates");
    const beforePublication = await prepublication(base);
    const prepublicationProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(prepublicationProcess);

    const coarse = await coarseRun(terrain, objects, base);
    const coarseProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(coarseProcess);

    const nominal = await nominalRun(terrain, objects, base, coarse);
    const resultSha256 = await sha256({
        coarse: coarse.actor,
        nominal: nominal.actor,
        coarseQueries: coarse.terrainQueryCount,
        nominalQueries: nominal.terrainQueryCount,
    });
    const nominalProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(nominalProcess);
    await lifecycle("start");
    await event("workbench.pause");
    return {
        prepublicationProcess,
        coarseProcess,
        nominalProcess,
        beforePublication,
        coarse,
        nominal,
        resultSha256,
    };
}
