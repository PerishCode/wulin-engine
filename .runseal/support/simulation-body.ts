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
} from "./canonical-runtime.ts";

const REVISION = "transactional-simulation-body-advance-v1";
const HALF_HEIGHT = 65_536;
const MAX_ELAPSED = 125_000_000;
const I32_MAX = 2_147_483_647;

type MotionPayload = {
    region_x: number;
    region_z: number;
    local_x_q9: number;
    local_z_q9: number;
    center_height_numerator: number;
    half_height_numerator: number;
    step_velocity_q16: number;
};

function request(
    generation: number,
    elapsed: number,
    deltaXQ9: number,
    deltaZQ9: number,
    limit: number,
    acceleration: number,
): Json {
    return {
        generation,
        elapsed_nanoseconds: elapsed,
        delta_x_q9: deltaXQ9,
        delta_z_q9: deltaZQ9,
        step_up_limit_q16: limit,
        step_acceleration_q16: acceleration,
    };
}

function requireFailure(value: Json, label: string, detail: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("simulation_body_advance_failed: ") ||
        !value.error.includes(detail)
    ) fail(`${label} returned the wrong simulation-body rejection: ${JSON.stringify(value)}`);
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
        response.revision !== REVISION || number(response, "stepCount") !== stepCount ||
        number(response, "terrainQueryCount") !== queryCount ||
        response.perOperationAllocationBytes !== 0 || response.sourceReadCount !== 0 ||
        response.gpuCopyCount !== 0 || response.gpuReadbackCount !== 0 ||
        response.fenceWaitCount !== 0 || response.synchronizationCount !== 0 ||
        response.scheduleCommitCount !== 1 || response.retainedCommitCount !== 1 ||
        response.presentationMutationCount !== 0 || response.frameCount !== 0 ||
        response.rendererWorkCount !== 0
    ) fail(`${label} performed work outside one CPU dual commit`);
    const value = object(response, "simulationBodyAdvance");
    const simulation = object(value, "simulation");
    const body = object(value, "body");
    if (
        number(simulation, "elapsedNanoseconds") !== elapsed ||
        number(simulation, "startTick") !== startTick ||
        number(simulation, "stepCount") !== stepCount ||
        number(simulation, "endTick") !== startTick + stepCount ||
        number(body, "stepCount") !== stepCount ||
        number(body, "terrainQueryCount") !== queryCount ||
        number(object(object(body, "input"), "handle"), "generation") !==
            number(object(object(body, "output"), "handle"), "generation")
    ) fail(`${label} schedule/body evidence diverged`);
    return value;
}

async function spawn(payload: MotionPayload): Promise<Json> {
    return object(await event("canonical.terrain.body.spawn", payload), "retained");
}

async function read(generation: number): Promise<Json> {
    return object(await event("canonical.terrain.body.read", { generation }), "retained");
}

async function despawn(generation: number, expected: Json): Promise<void> {
    const value = object(await event("canonical.terrain.body.despawn", { generation }), "retained");
    same(value, expected, "simulation-body despawn");
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
    const initialSimulation = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");
    const empty = await rejectedEvent(
        "simulation.terrain.body.advance",
        request(1, 1, 0, 0, 0, 0),
    );
    requireFailure(empty, "empty dual advance", "no retained terrain body is live");
    const malformed = await rejectedEvent("simulation.terrain.body.advance", {
        ...request(1, 1, 0, 0, 0, 0),
        elapsed_nanoseconds: -1,
    });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("negative dual elapsed returned the wrong rejection");
    }
    const motion: MotionPayload = {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
        center_height_numerator: HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
    };
    const stored = await spawn(motion);
    const stale = await rejectedEvent(
        "simulation.terrain.body.advance",
        request(2, 1, 0, 0, 0, 0),
    );
    requireFailure(stale, "stale dual advance", "handle is stale");
    const oversized = await rejectedEvent(
        "simulation.terrain.body.advance",
        request(1, MAX_ELAPSED + 1, 0, 0, 0, 0),
    );
    requireFailure(oversized, "oversized dual advance", "must be in [0, 125000000]");
    same(await event("simulation.status"), initialSimulation, "dual validation schedule rollback");
    same(await read(1), stored, "dual validation body rollback");
    const fractional = requireAdvance(
        await event("simulation.terrain.body.advance", request(1, 1, 17, -19, 0, 0)),
        1,
        0,
        0,
        0,
        "fractional dual advance",
    );
    same(object(object(fractional, "body"), "output"), stored, "fractional body identity");
    requireStatus(await event("simulation.status"), 0, 60, 1, 0, "fractional dual commit");
    same(await event("canonical.time.status"), initialPresentation, "dual time isolation");
    return { empty, malformed, stale, oversized, fractional };
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

async function groundMotion(base: [number, number]): Promise<MotionPayload> {
    const ground = await height(base[0], base[1], -3904, -3968);
    return {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: -3904,
        local_z_q9: -3968,
        center_height_numerator: ground + HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
    };
}

async function advanceSequence(intervals: number[]): Promise<Json> {
    const advances: Json[] = [];
    let startTick = 0;
    let terrainQueryCount = 0;
    for (const elapsed of intervals) {
        const response = await event(
            "simulation.terrain.body.advance",
            request(1, elapsed, 128, 0, I32_MAX, -1092),
        );
        const value = object(response, "simulationBodyAdvance");
        const stepCount = number(object(value, "simulation"), "stepCount");
        const queryCount = number(object(value, "body"), "terrainQueryCount");
        requireAdvance(response, elapsed, startTick, stepCount, queryCount, "dual sequence");
        if (queryCount !== stepCount) fail("dual sequence query/step count diverged");
        startTick += stepCount;
        terrainQueryCount += queryCount;
        advances.push(value);
    }
    return { advances, terrainQueryCount, retained: await read(1) };
}

async function coarseRun(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    const presentation = await startPublished(terrain, objects, base);
    const motion = await groundMotion(base);
    await spawn(motion);
    const result = await advanceSequence(Array(8).fill(MAX_ELAPSED));
    requireStatus(await event("simulation.status"), 60, 0, 8, 60, "coarse dual second");
    if (result.terrainQueryCount !== 60) fail("coarse dual query count diverged");
    same(await event("canonical.time.status"), presentation, "coarse dual time isolation");
    return { motion, ...result };
}

async function nominalRun(
    terrain: string,
    objects: string,
    base: [number, number],
    expected: Json,
): Promise<Json> {
    const presentation = await startPublished(terrain, objects, base);
    await spawn(expected.motion as MotionPayload);
    const intervals = [...Array(20).fill(16_666_666), ...Array(40).fill(16_666_667)];
    const result = await advanceSequence(intervals);
    requireStatus(await event("simulation.status"), 60, 0, 60, 60, "nominal dual second");
    if (result.terrainQueryCount !== 60) fail("nominal dual query count diverged");
    same(result.retained, expected.retained, "dual partition retained output");
    await despawn(1, result.retained as Json);

    const edgeGround = await height(base[0], base[1], 0, 0);
    const edgeMotion: MotionPayload = {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
        center_height_numerator: edgeGround + HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
    };
    const edgeStored = await spawn(edgeMotion);
    const beforeFailure = await event("simulation.status");
    const failed = await rejectedEvent(
        "simulation.terrain.body.advance",
        request(2, MAX_ELAPSED, 8192, 0, I32_MAX, 0),
    );
    requireFailure(failed, "dual mid-batch snapshot", "batch step 3 of 7 failed");
    same(await event("simulation.status"), beforeFailure, "dual query schedule rollback");
    same(await read(2), edgeStored, "dual query body rollback");
    await despawn(2, edgeStored);

    const overflowMotion: MotionPayload = {
        ...edgeMotion,
        step_velocity_q16: I32_MAX,
    };
    const overflowStored = await spawn(overflowMotion);
    const overflow = await rejectedEvent(
        "simulation.terrain.body.advance",
        request(3, MAX_ELAPSED, 0, 0, 0, 1),
    );
    requireFailure(
        overflow,
        "dual arithmetic",
        "vertical velocity is outside the signed 32-bit Q16 range",
    );
    same(await event("simulation.status"), beforeFailure, "dual arithmetic schedule rollback");
    same(await read(3), overflowStored, "dual arithmetic body rollback");
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

export async function simulationBodyGates(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    console.log("==> transactional simulation-body advance gates");
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
        coarse: coarse.retained,
        nominal: nominal.retained,
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
