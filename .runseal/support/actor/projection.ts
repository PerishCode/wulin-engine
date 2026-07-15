import {
    type Coord,
    event,
    fail,
    type GlobalConfig,
    type Json,
    number,
    object,
    openSources,
    publish,
    rejectedEvent,
    same,
    startClean,
    target,
    targetMatches,
} from "../canonical-runtime.ts";

const REVISION = "bounded-actor-render-projection-v1";

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

function requireFailure(value: Json, label: string, detail: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("actor_projection_failed: ") ||
        !value.error.includes(detail)
    ) fail(`${label} returned the wrong actor projection rejection: ${JSON.stringify(value)}`);
}

function requireProjection(
    response: Json,
    expectedActor: Json,
    expectedConfig: GlobalConfig,
    activeRegionIndex: number,
    semanticRegion: number,
    windowPositionQ9: [number, number],
    payload: ActorPayload,
): Json {
    if (
        response.revision !== REVISION || response.perOperationAllocationBytes !== 0 ||
        response.actorReadCount !== 1 || response.compositionSnapshotReadCount !== 1 ||
        response.terrainQueryCount !== 0 || response.sourceReadCount !== 0 ||
        response.gpuCopyCount !== 0 || response.gpuReadbackCount !== 0 ||
        response.fenceWaitCount !== 0 || response.synchronizationCount !== 0 ||
        response.scheduleMutationCount !== 0 || response.actorMutationCount !== 0 ||
        response.presentationMutationCount !== 0 || response.frameCount !== 0 ||
        response.rendererWorkCount !== 0
    ) fail("actor projection performed work outside the copied projection transaction");
    const projection = object(response, "projection");
    if (!targetMatches(projection, expectedConfig)) fail("actor projection config diverged");
    same(object(projection, "actor"), expectedActor, "actor projection input preservation");
    if (
        number(projection, "activeRegionIndex") !== activeRegionIndex ||
        number(projection, "semanticRegion") !== semanticRegion ||
        number(projection, "centerHeightQ16") !== payload.center_height_numerator ||
        number(projection, "halfHeightQ16") !== payload.half_height_numerator ||
        number(projection, "positionDenominator") !== 512 ||
        number(projection, "heightDenominator") !== 65_536
    ) fail("actor projection scalar evidence diverged");
    same(projection.windowPositionQ9, windowPositionQ9, "actor projection exact Q9 position");
    return projection;
}

async function spawn(payload: ActorPayload): Promise<Json> {
    return object(await event("actor.spawn", payload), "actor");
}

async function despawn(generation: number): Promise<void> {
    await event("actor.despawn", { generation });
}

async function sha256(value: unknown): Promise<string> {
    const bytes = new TextEncoder().encode(JSON.stringify(value));
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function actorProjectionGates(
    terrain: string,
    objects: string,
    base: Coord,
): Promise<Json> {
    console.log("==> bounded actor render projection gates");
    await startClean();
    await event("workbench.pause");
    const initialSimulation = await event("simulation.status");
    const initialPresentation = await event("canonical.time.status");

    const empty = await rejectedEvent("actor.project", { generation: 1 });
    requireFailure(empty, "empty projection", "no runtime actor is live");
    const malformed = await rejectedEvent("actor.project", { generation: -1 });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("malformed actor projection generation returned the wrong rejection");
    }

    const payload: ActorPayload = {
        region_x: base[0] + 1,
        region_z: base[1] - 1,
        local_x_q9: 73,
        local_z_q9: -91,
        center_height_numerator: 200_003,
        half_height_numerator: 65_537,
        step_velocity_q16: -19,
        archetype: 7,
        material: 63,
        yaw_q16: 32_768,
        animation: (2 << 16) | (17 << 8) | 1,
    };
    const actor = await spawn(payload);
    const unavailable = await rejectedEvent("actor.project", { generation: 1 });
    requireFailure(unavailable, "pre-publication projection", "enabled canonical composition");
    same(
        await event("canonical.time.status"),
        initialPresentation,
        "unavailable projection presentation independence",
    );

    await openSources(terrain, objects);
    const canonicalConfig = target(base);
    const canonicalPublication = await publish(canonicalConfig);
    const canonicalPresentation = await event("canonical.time.status");
    const projected = requireProjection(
        await event("actor.project", { generation: 1 }),
        actor,
        canonicalConfig,
        8,
        63 * 128 + 65,
        [8_265, -8_283],
        payload,
    );
    const replay = requireProjection(
        await event("actor.project", { generation: 1 }),
        actor,
        canonicalConfig,
        8,
        63 * 128 + 65,
        [8_265, -8_283],
        payload,
    );
    same(replay, projected, "actor projection immediate replay");
    const resultSha256 = await sha256(projected);
    const replaySha256 = await sha256(replay);
    if (resultSha256 !== replaySha256) fail("actor projection replay digest diverged");
    const stale = await rejectedEvent("actor.project", { generation: 2 });
    requireFailure(stale, "stale projection", "handle is stale");
    same(
        object(await event("actor.read", { generation: 1 }), "actor"),
        actor,
        "projection rollback",
    );
    same(
        await event("canonical.time.status"),
        canonicalPresentation,
        "canonical projection presentation independence",
    );

    const aliasConfig = target(base, 96, 32);
    const aliasPublication = await publish(aliasConfig);
    const aliasPresentation = await event("canonical.time.status");
    const aliased = requireProjection(
        await event("actor.project", { generation: 1 }),
        actor,
        aliasConfig,
        8,
        63 * 128 + 65,
        [8_265, -8_283],
        payload,
    );
    for (const field of ["activeRegionIndex", "semanticRegion", "windowPositionQ9"] as const) {
        same(aliased[field], projected[field], `actor projection alias invariance ${field}`);
    }
    await despawn(1);

    const edgePayload: ActorPayload = {
        ...payload,
        region_x: base[0] + 2,
        region_z: base[1] - 2,
        local_x_q9: 4_095,
        local_z_q9: -4_096,
    };
    const edgeActor = await spawn(edgePayload);
    const edge = requireProjection(
        await event("actor.project", { generation: 2 }),
        edgeActor,
        aliasConfig,
        4,
        62 * 128 + 66,
        [20_479, -20_480],
        edgePayload,
    );
    await despawn(2);

    const outsideActor = await spawn({ ...payload, region_x: base[0] + 3 });
    const outside = await rejectedEvent("actor.project", { generation: 3 });
    requireFailure(outside, "outside-window projection", "outside the active render window");
    same(
        object(await event("actor.read", { generation: 3 }), "actor"),
        outsideActor,
        "outside projection rollback",
    );
    await despawn(3);

    same(await event("simulation.status"), initialSimulation, "projection schedule independence");
    same(
        await event("canonical.time.status"),
        aliasPresentation,
        "projection presentation independence",
    );
    return {
        empty,
        malformed,
        unavailable,
        canonicalPublication,
        projected,
        replay,
        resultSha256,
        replaySha256,
        stale,
        aliasPublication,
        aliased,
        edge,
        outside,
    };
}
