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

const REVISION = "transactional-retained-terrain-body-advance-v1";
const HALF_HEIGHT = 65_536;
const REGION_SIDE_Q9 = 8192;
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

type Sample = {
    regionX: number;
    regionZ: number;
    localXQ9: number;
    localZQ9: number;
    height: number;
};

function requireFailure(value: Json, label: string, detail: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("retained_terrain_advance_failed: ") ||
        !value.error.includes(detail)
    ) fail(`${label} returned the wrong retained-advance rejection: ${JSON.stringify(value)}`);
}

function retained(
    response: Json,
    generation: number,
    expected: MotionPayload,
    label: string,
): Json {
    const value = object(response, "retained");
    if (number(object(value, "handle"), "generation") !== generation) {
        fail(`${label} returned the wrong retained generation`);
    }
    requireMotion(object(value, "motion"), expected, label);
    return value;
}

function requireMotion(motion: Json, expected: MotionPayload, label: string): void {
    const body = object(motion, "body");
    const position = object(body, "position");
    const region = object(position, "region");
    if (
        number(region, "x") !== expected.region_x || number(region, "z") !== expected.region_z ||
        number(position, "localXQ9") !== expected.local_x_q9 ||
        number(position, "localZQ9") !== expected.local_z_q9 ||
        number(body, "centerHeightNumerator") !== expected.center_height_numerator ||
        number(body, "halfHeightNumerator") !== expected.half_height_numerator ||
        number(motion, "stepVelocityQ16") !== expected.step_velocity_q16
    ) fail(`${label} changed retained motion`);
}

function position(motion: Json): Json {
    return object(object(motion, "body"), "position");
}

function requirePosition(position: Json, sample: Sample, label: string): void {
    const region = object(position, "region");
    if (
        number(region, "x") !== sample.regionX || number(region, "z") !== sample.regionZ ||
        number(position, "localXQ9") !== sample.localXQ9 ||
        number(position, "localZQ9") !== sample.localZQ9
    ) fail(`${label} returned the wrong terrain position`);
}

async function read(generation: number): Promise<Json> {
    return await event("canonical.terrain.body.read", { generation });
}

async function spawn(payload: MotionPayload, generation: number): Promise<Json> {
    const response = await event("canonical.terrain.body.spawn", payload);
    return retained(response, generation, payload, "spawn");
}

async function despawn(generation: number, expected: Json): Promise<void> {
    const response = await event("canonical.terrain.body.despawn", { generation });
    same(object(response, "retained"), expected, "despawn exact retained value");
}

function advancePayload(
    generation: number,
    deltaXQ9: number,
    deltaZQ9: number,
    limit: number,
    acceleration: number,
): Json {
    return {
        generation,
        delta_x_q9: deltaXQ9,
        delta_z_q9: deltaZQ9,
        step_up_limit_q16: limit,
        step_acceleration_q16: acceleration,
    };
}

async function advance(request: Json, expectedQueries: number, label: string): Promise<Json> {
    const response = await event("canonical.terrain.body.retained.advance", request);
    if (
        response.revision !== REVISION ||
        number(response, "terrainQueryCount") !== expectedQueries ||
        response.perOperationAllocationBytes !== 0 || response.sourceReadCount !== 0 ||
        response.gpuCopyCount !== 0 || response.gpuReadbackCount !== 0 ||
        response.fenceWaitCount !== 0 || response.synchronizationCount !== 0 ||
        response.scheduleMutationCount !== 0 || response.presentationMutationCount !== 0 ||
        response.frameCount !== 0 || response.rendererWorkCount !== 0
    ) fail(`${label} performed work outside one retained CPU advance`);
    const retainedAdvance = object(response, "retainedAdvance");
    const input = object(retainedAdvance, "input");
    const output = object(retainedAdvance, "output");
    const copiedAdvance = object(retainedAdvance, "advance");
    if (
        number(object(input, "handle"), "generation") !==
            number(object(output, "handle"), "generation") ||
        number(copiedAdvance, "terrainQueryCount") !== expectedQueries
    ) fail(`${label} changed generation or query authority`);
    same(object(input, "motion"), object(copiedAdvance, "input"), `${label} copied input`);
    same(object(output, "motion"), object(copiedAdvance, "output"), `${label} committed output`);
    return retainedAdvance;
}

async function sample(
    regionX: number,
    regionZ: number,
    localXQ9: number,
    localZQ9: number,
): Promise<Sample> {
    const response = await event("canonical.terrain.height", {
        region_x: regionX,
        region_z: regionZ,
        local_x_q9: localXQ9,
        local_z_q9: localZQ9,
    });
    return {
        regionX,
        regionZ,
        localXQ9,
        localZQ9,
        height: number(object(response, "height"), "heightNumerator"),
    };
}

async function prepublication(base: [number, number]): Promise<Json> {
    const initialSimulation = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");
    const empty = await rejectedEvent(
        "canonical.terrain.body.retained.advance",
        advancePayload(1, 0, 0, 0, 0),
    );
    requireFailure(empty, "empty handle", "no retained terrain body is live");
    const malformed = await rejectedEvent("canonical.terrain.body.retained.advance", {
        ...advancePayload(1, 0, 0, 0, 0),
        generation: -1,
    });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("malformed retained generation returned the wrong rejection");
    }

    const payload: MotionPayload = {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: -3904,
        local_z_q9: -3968,
        center_height_numerator: 200_000,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: -17,
    };
    const stored = await spawn(payload, 1);
    const stale = await rejectedEvent(
        "canonical.terrain.body.retained.advance",
        advancePayload(2, 0, 0, 0, 0),
    );
    requireFailure(stale, "stale prepublication handle", "handle is stale");
    const negative = await rejectedEvent(
        "canonical.terrain.body.retained.advance",
        advancePayload(1, 0, 0, -1, 0),
    );
    requireFailure(negative, "negative limit", "must be nonnegative");
    same(object(await read(1), "retained"), stored, "prepublication failure rollback");
    await despawn(1, stored);
    same(await event("simulation.status"), initialSimulation, "prepublication schedule isolation");
    same(
        await event("canonical.time.status"),
        initialPresentation,
        "prepublication presentation isolation",
    );
    return { empty, malformed, stale, negative, stored };
}

async function trajectory(base: [number, number]): Promise<Json> {
    const lower = await sample(base[0], base[1], -3904, -3968);
    const upper = await sample(base[0], base[1], -3776, -3968);
    if (upper.height - lower.height !== 128) fail("controlled retained-advance rise diverged");
    const initialSimulation = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");

    const start: MotionPayload = {
        region_x: lower.regionX,
        region_z: lower.regionZ,
        local_x_q9: lower.localXQ9,
        local_z_q9: lower.localZQ9,
        center_height_numerator: lower.height + HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
    };
    await spawn(start, 2);
    const accepted = await advance(advancePayload(2, 128, 0, 128, -10), 1, "accepted uphill");
    const acceptedAdvance = object(accepted, "advance");
    const acceptedMotion = object(object(accepted, "output"), "motion");
    requirePosition(position(acceptedMotion), upper, "accepted uphill");
    if (
        object(acceptedAdvance, "translation").blocked !== false ||
        acceptedAdvance.grounded !== true || number(acceptedMotion, "stepVelocityQ16") !== 0 ||
        number(object(acceptedMotion, "body"), "centerHeightNumerator") !==
            upper.height + HALF_HEIGHT
    ) fail("accepted retained uphill result diverged");
    same(object(await read(2), "retained"), object(accepted, "output"), "accepted commit readback");

    const downhill = await advance(advancePayload(2, -128, 0, 0, -10), 1, "downhill");
    const downhillAdvance = object(downhill, "advance");
    const downhillMotion = object(object(downhill, "output"), "motion");
    requirePosition(position(downhillMotion), lower, "downhill");
    if (
        object(downhillAdvance, "translation").blocked !== false ||
        downhillAdvance.grounded !== false || number(downhillMotion, "stepVelocityQ16") !== -10 ||
        number(object(downhillMotion, "body"), "centerHeightNumerator") !==
            upper.height + HALF_HEIGHT - 10
    ) fail("retained downhill result diverged");

    const blocked = await advance(advancePayload(2, 128, 0, 0, -10), 2, "blocked uphill");
    const blockedAdvance = object(blocked, "advance");
    const blockedMotion = object(object(blocked, "output"), "motion");
    requirePosition(position(blockedMotion), lower, "blocked uphill");
    if (
        object(blockedAdvance, "translation").blocked !== true ||
        number(blockedMotion, "stepVelocityQ16") !== -20 ||
        number(object(blockedMotion, "body"), "centerHeightNumerator") !==
            upper.height + HALF_HEIGHT - 30
    ) fail("retained blocked result diverged");

    const beforeOutside = object(await read(2), "retained");
    const outside = await rejectedEvent(
        "canonical.terrain.body.retained.advance",
        advancePayload(2, 100 * REGION_SIDE_Q9, 0, 0, -10),
    );
    requireFailure(outside, "outside destination", "outside the published active window");
    same(object(await read(2), "retained"), beforeOutside, "outside failure rollback");
    await despawn(2, beforeOutside);

    const overflowPayload: MotionPayload = {
        ...start,
        step_velocity_q16: I32_MAX,
    };
    const overflowStored = await spawn(overflowPayload, 3);
    const overflow = await rejectedEvent(
        "canonical.terrain.body.retained.advance",
        advancePayload(3, 0, 0, 0, 1),
    );
    requireFailure(overflow, "velocity overflow", "outside the signed 32-bit Q16 range");
    same(object(await read(3), "retained"), overflowStored, "overflow failure rollback");
    await despawn(3, overflowStored);
    same(await event("simulation.status"), initialSimulation, "trajectory schedule isolation");
    same(
        await event("canonical.time.status"),
        initialPresentation,
        "trajectory presentation isolation",
    );
    return { lower, upper, accepted, downhill, blocked, outside, overflow };
}

async function runOnce(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    await startClean();
    await event("workbench.pause");
    const beforePublication = await prepublication(base);
    await openSources(terrain, objects);
    await publish(target(base));
    const published = await trajectory(base);
    return { beforePublication, published };
}

async function sha256(value: unknown): Promise<string> {
    const bytes = new TextEncoder().encode(JSON.stringify(value));
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function retainedAdvanceGates(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    console.log("==> transactional retained terrain-body advance gates");
    const first = await runOnce(terrain, objects, base);
    const firstSha256 = await sha256(first);
    const firstProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(firstProcess);

    const replay = await runOnce(terrain, objects, base);
    const replaySha256 = await sha256(replay);
    if (firstSha256 !== replaySha256) fail("retained advance replay digest diverged");
    const replayProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(replayProcess);
    await lifecycle("start");
    await event("workbench.pause");
    const clean = await rejectedEvent("canonical.terrain.body.read", { generation: 1 });
    if (
        typeof clean.error !== "string" ||
        !clean.error.includes("no retained terrain body is live")
    ) fail("clean retained-advance process did not restart empty");
    return {
        firstProcess,
        replayProcess,
        first,
        replay,
        firstSha256,
        replaySha256,
        clean,
    };
}
