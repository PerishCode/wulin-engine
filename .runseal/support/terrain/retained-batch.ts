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

const REVISION = "transactional-retained-terrain-body-batch-v1";
const HALF_HEIGHT = 65_536;
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
    stepCount: number,
    deltaXQ9: number,
    deltaZQ9: number,
    limit: number,
    acceleration: number,
): Json {
    return {
        generation,
        step_count: stepCount,
        delta_x_q9: deltaXQ9,
        delta_z_q9: deltaZQ9,
        step_up_limit_q16: limit,
        step_acceleration_q16: acceleration,
    };
}

function requireFailure(value: Json, label: string, detail: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("retained_terrain_batch_failed: ") ||
        !value.error.includes(detail)
    ) fail(`${label} returned the wrong retained-batch rejection: ${JSON.stringify(value)}`);
}

function requireBatch(response: Json, stepCount: number, queryCount: number, label: string): Json {
    if (
        response.revision !== REVISION || number(response, "stepCount") !== stepCount ||
        number(response, "terrainQueryCount") !== queryCount ||
        response.perOperationAllocationBytes !== 0 || response.sourceReadCount !== 0 ||
        response.gpuCopyCount !== 0 || response.gpuReadbackCount !== 0 ||
        response.fenceWaitCount !== 0 || response.synchronizationCount !== 0 ||
        response.scheduleMutationCount !== 0 || response.presentationMutationCount !== 0 ||
        response.frameCount !== 0 || response.rendererWorkCount !== 0
    ) fail(`${label} performed work outside one retained CPU batch`);
    const batch = object(response, "retainedBatch");
    if (
        number(batch, "stepCount") !== stepCount ||
        number(batch, "terrainQueryCount") !== queryCount ||
        number(object(object(batch, "input"), "handle"), "generation") !==
            number(object(object(batch, "output"), "handle"), "generation")
    ) fail(`${label} changed generation or batch evidence`);
    return batch;
}

async function spawn(payload: MotionPayload): Promise<Json> {
    return object(await event("canonical.terrain.body.spawn", payload), "retained");
}

async function read(generation: number): Promise<Json> {
    return object(await event("canonical.terrain.body.read", { generation }), "retained");
}

async function despawn(generation: number, expected: Json): Promise<void> {
    const value = object(await event("canonical.terrain.body.despawn", { generation }), "retained");
    same(value, expected, "retained batch despawn");
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

async function prepublication(base: [number, number]): Promise<Json> {
    await startClean();
    await event("workbench.pause");
    const initialSimulation = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");
    const empty = await rejectedEvent(
        "canonical.terrain.body.retained.batch",
        request(1, 0, 0, 0, 0, 0),
    );
    requireFailure(empty, "empty batch", "no retained terrain body is live");
    const malformed = await rejectedEvent("canonical.terrain.body.retained.batch", {
        ...request(1, 0, 0, 0, 0, 0),
        step_count: -1,
    });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("negative batch count returned the wrong rejection");
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
        "canonical.terrain.body.retained.batch",
        request(2, 0, 0, 0, 0, 0),
    );
    requireFailure(stale, "stale batch", "handle is stale");
    const oversized = await rejectedEvent(
        "canonical.terrain.body.retained.batch",
        request(1, 9, 0, 0, 0, 0),
    );
    requireFailure(oversized, "oversized batch", "must be in [0, 8]");
    const zero = requireBatch(
        await event("canonical.terrain.body.retained.batch", request(1, 0, 17, -19, 0, 0)),
        0,
        0,
        "zero batch",
    );
    same(object(zero, "input"), stored, "zero batch input");
    same(object(zero, "output"), stored, "zero batch output");
    const invalid = await rejectedEvent(
        "canonical.terrain.body.retained.batch",
        request(1, 1, 0, 0, -1, 0),
    );
    requireFailure(invalid, "invalid batch limit", "must be nonnegative");
    same(await read(1), stored, "prepublication batch rollback");
    await despawn(1, stored);
    same(await event("simulation.status"), initialSimulation, "batch schedule isolation");
    same(await event("canonical.time.status"), initialPresentation, "batch time isolation");
    return { empty, malformed, stale, oversized, zero, invalid };
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
    return {
        simulation: await event("simulation.status"),
        presentation: await event("canonical.time.status"),
    };
}

async function batchRun(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    const initial = await startPublished(terrain, objects, base);
    const ground = await height(base[0], base[1], -3904, -3968);
    const motion: MotionPayload = {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: -3904,
        local_z_q9: -3968,
        center_height_numerator: ground + HALF_HEIGHT,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: 0,
    };
    const stored = await spawn(motion);
    const batch = requireBatch(
        await event(
            "canonical.terrain.body.retained.batch",
            request(1, 8, 128, 0, I32_MAX, -1092),
        ),
        8,
        8,
        "eight-step batch",
    );
    same(object(batch, "input"), stored, "eight-step batch input");
    same(await read(1), object(batch, "output"), "eight-step batch commit");
    same(await event("simulation.status"), initial.simulation, "batch schedule isolation");
    same(await event("canonical.time.status"), initial.presentation, "batch time isolation");
    return { motion, batch };
}

async function partitionRun(
    terrain: string,
    objects: string,
    base: [number, number],
    expected: Json,
): Promise<Json> {
    const initial = await startPublished(terrain, objects, base);
    let retained = await spawn(expected.motion as MotionPayload);
    let terrainQueryCount = 0;
    const advances: Json[] = [];
    for (let index = 0; index < 8; index += 1) {
        const response = await event("canonical.terrain.body.retained.advance", {
            generation: 1,
            delta_x_q9: 128,
            delta_z_q9: 0,
            step_up_limit_q16: I32_MAX,
            step_acceleration_q16: -1092,
        });
        const advance = object(response, "retainedAdvance");
        retained = object(advance, "output");
        terrainQueryCount += number(object(advance, "advance"), "terrainQueryCount");
        advances.push(advance);
    }
    same(retained, object(object(expected, "batch"), "output"), "batch partition output");
    if (terrainQueryCount !== 8) fail("partitioned retained query count diverged");
    await despawn(1, retained);

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
    const failed = await rejectedEvent(
        "canonical.terrain.body.retained.batch",
        request(2, 8, 8192, 0, I32_MAX, 0),
    );
    requireFailure(failed, "mid-batch snapshot", "batch step 3 of 8 failed");
    same(await read(2), edgeStored, "mid-batch snapshot rollback");
    same(await event("simulation.status"), initial.simulation, "partition schedule isolation");
    same(await event("canonical.time.status"), initial.presentation, "partition time isolation");
    return { advances, terrainQueryCount, retained, failed, edgeStored };
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

export async function retainedBatchGates(
    terrain: string,
    objects: string,
    base: [number, number],
): Promise<Json> {
    console.log("==> transactional retained terrain-body batch gates");
    const beforePublication = await prepublication(base);
    const prepublicationProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(prepublicationProcess);

    const batched = await batchRun(terrain, objects, base);
    const batchProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(batchProcess);

    const partitioned = await partitionRun(terrain, objects, base, batched);
    const resultSha256 = await sha256({
        batched: object(object(batched, "batch"), "output"),
        partitioned: partitioned.retained,
        terrainQueryCount: partitioned.terrainQueryCount,
    });
    const partitionProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(partitionProcess);
    await lifecycle("start");
    await event("workbench.pause");
    return {
        prepublicationProcess,
        batchProcess,
        partitionProcess,
        beforePublication,
        batched,
        partitioned,
        resultSha256,
    };
}
