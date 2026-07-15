import {
    assertStopped,
    event,
    fail,
    type Json,
    lifecycle,
    number,
    object,
    rejectedEvent,
    same,
    startClean,
    status,
} from "../canonical-runtime.ts";

const REVISION = "retained-runtime-actor-v1";
const FIRST_MOTION = {
    region_x: -7,
    region_z: 11,
    local_x_q9: -4096,
    local_z_q9: 2048,
    center_height_numerator: 200_000,
    half_height_numerator: 65_536,
    step_velocity_q16: -17,
    archetype: 7,
    material: 63,
    yaw_q16: 0,
    animation: 1,
};
const SECOND_MOTION = {
    region_x: 13,
    region_z: -19,
    local_x_q9: 3072,
    local_z_q9: -1024,
    center_height_numerator: -300_000,
    half_height_numerator: 32_768,
    step_velocity_q16: 29,
    archetype: 2,
    material: 5,
    yaw_q16: 32_768,
    animation: 4_294_967_295,
};

function requireRejection(value: Json, label: string, detail?: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("actor_lifecycle_failed: ") ||
        (detail !== undefined && !value.error.includes(detail))
    ) fail(`${label} returned the wrong lifecycle rejection`);
}

function requireOperation(
    response: Json,
    operation: "spawn" | "read" | "despawn",
    generation: number,
    liveCount: number,
    expectedMotion: Json,
): Json {
    if (
        response.revision !== REVISION || response.operation !== operation ||
        number(response, "capacity") !== 1 || number(response, "liveCount") !== liveCount ||
        response.perOperationAllocationBytes !== 0 || response.terrainQueryCount !== 0 ||
        response.sourceReadCount !== 0 || response.gpuCopyCount !== 0 ||
        response.gpuReadbackCount !== 0 || response.fenceWaitCount !== 0 ||
        response.synchronizationCount !== 0 || response.scheduleMutationCount !== 0 ||
        number(response, "actorMutationCount") !== (operation === "read" ? 0 : 1) ||
        response.presentationMutationCount !== 0 || response.frameCount !== 0 ||
        response.rendererWorkCount !== 0
    ) fail(`${operation} performed work outside the retained actor slot`);
    const actor = object(response, "actor");
    if (number(object(actor, "handle"), "generation") !== generation) {
        fail(`${operation} returned the wrong generation`);
    }
    requireMotion(object(actor, "motion"), expectedMotion, operation);
    const presentation = object(actor, "presentation");
    for (
        const [field, source] of [
            ["archetype", "archetype"],
            ["material", "material"],
            ["yawQ16", "yaw_q16"],
            ["animation", "animation"],
        ]
    ) {
        if (number(presentation, field) !== number(expectedMotion, source)) {
            fail(`${operation} changed actor presentation ${field}`);
        }
    }
    return actor;
}

function requireMotion(motion: Json, expected: Json, label: string): void {
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
    ) fail(`${label} changed actor fixed-point motion`);
}

async function invalidPresentationGate(): Promise<Json[]> {
    const invalid = [
        { ...FIRST_MOTION, archetype: 8 },
        { ...FIRST_MOTION, material: 64 },
        { ...FIRST_MOTION, yaw_q16: 65_536 },
        { ...FIRST_MOTION, animation: 8 },
        { ...FIRST_MOTION, animation: 64 << 8 | 1 },
    ];
    const evidence: Json[] = [];
    for (const payload of invalid) {
        const rejected = await rejectedEvent("actor.spawn", payload);
        requireRejection(rejected, "invalid actor presentation", "presentation");
        evidence.push(rejected);
    }
    const empty = await rejectedEvent("actor.read", { generation: 1 });
    requireRejection(empty, "post-validation empty read", "no runtime actor is live");
    return evidence;
}

async function lifecycleSequence(): Promise<Json> {
    const empty = await rejectedEvent("actor.read", { generation: 1 });
    requireRejection(empty, "empty read", "no runtime actor is live");

    const invalidPresentations = await invalidPresentationGate();

    const firstSpawn = await event("actor.spawn", FIRST_MOTION);
    const first = requireOperation(firstSpawn, "spawn", 1, 1, FIRST_MOTION);
    const occupied = await rejectedEvent("actor.spawn", SECOND_MOTION);
    requireRejection(occupied, "occupied spawn", "slot is occupied");

    const wrongRead = await rejectedEvent("actor.read", { generation: 2 });
    requireRejection(wrongRead, "wrong read", "handle is stale");
    const wrongDespawn = await rejectedEvent("actor.despawn", { generation: 2 });
    requireRejection(wrongDespawn, "wrong despawn", "handle is stale");
    const firstRead = await event("actor.read", { generation: 1 });
    same(
        requireOperation(firstRead, "read", 1, 1, FIRST_MOTION),
        first,
        "failed-operation rollback",
    );
    const firstDespawn = await event("actor.despawn", { generation: 1 });
    same(
        requireOperation(firstDespawn, "despawn", 1, 0, FIRST_MOTION),
        first,
        "exact first despawn",
    );

    const secondSpawn = await event("actor.spawn", SECOND_MOTION);
    const second = requireOperation(secondSpawn, "spawn", 2, 1, SECOND_MOTION);
    const stale = await rejectedEvent("actor.read", { generation: 1 });
    requireRejection(stale, "stale read", "handle is stale");
    const secondRead = await event("actor.read", { generation: 2 });
    same(requireOperation(secondRead, "read", 2, 1, SECOND_MOTION), second, "exact second read");
    const secondDespawn = await event("actor.despawn", { generation: 2 });
    same(
        requireOperation(secondDespawn, "despawn", 2, 0, SECOND_MOTION),
        second,
        "exact second despawn",
    );
    return {
        empty,
        invalidPresentations,
        firstSpawn,
        occupied,
        wrongRead,
        wrongDespawn,
        firstRead,
        firstDespawn,
        secondSpawn,
        stale,
        secondRead,
        secondDespawn,
    };
}

async function sha256(value: unknown): Promise<string> {
    const bytes = new TextEncoder().encode(JSON.stringify(value));
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function actorGates(): Promise<Json> {
    console.log("==> retained runtime-actor lifecycle gates");
    await startClean();
    await event("workbench.pause");
    const initialSimulation = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");
    const zero = await rejectedEvent("actor.read", { generation: 0 });
    requireRejection(zero, "zero generation", "must be nonzero");
    const malformed = await rejectedEvent("actor.read", { generation: -1 });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("malformed generation returned the wrong rejection");
    }

    const first = await lifecycleSequence();
    same(await event("simulation.status"), initialSimulation, "lifecycle schedule independence");
    same(
        await event("canonical.time.status"),
        initialPresentation,
        "lifecycle presentation independence",
    );
    const firstSha256 = await sha256(first);
    const firstProcess = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(firstProcess);
    await lifecycle("start");
    await event("workbench.pause");

    const restartedEmpty = await rejectedEvent("actor.read", { generation: 1 });
    requireRejection(restartedEmpty, "restart empty", "no runtime actor is live");
    const replay = await lifecycleSequence();
    const replaySha256 = await sha256(replay);
    if (firstSha256 !== replaySha256) fail("actor lifecycle replay digest diverged");
    same(await event("simulation.status"), initialSimulation, "replay schedule independence");
    same(
        await event("canonical.time.status"),
        initialPresentation,
        "replay presentation independence",
    );
    return {
        zero,
        malformed,
        firstProcess,
        restartedEmpty,
        first,
        replay,
        firstSha256,
        replaySha256,
    };
}
