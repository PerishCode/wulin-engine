import {
    assertStopped,
    type Coord,
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
    waitStatus,
} from "../canonical-runtime.ts";

const HALF_HEIGHT_Q16 = 65_536;
const SHORT_STEP_NANOSECONDS = 16_666_666;
const LONG_STEP_NANOSECONDS = 16_666_667;
const I32_MAX = 2_147_483_647;
const REVISION = "runtime-actor-simulation-v3";
const SURVEY = { archetype: 7, material: 63, yaw_q16: 0, animation: 0 };
const WALK = { ...SURVEY, animation: 1 };

function request(
    generation: number,
    elapsedNanoseconds: number,
    deltaXQ9: number,
    presentation = SURVEY,
): Json {
    return {
        generation,
        elapsed_nanoseconds: elapsedNanoseconds,
        delta_x_q9: deltaXQ9,
        delta_z_q9: 0,
        step_up_limit_q16: I32_MAX,
        step_acceleration_q16: 0,
        ...presentation,
    };
}

function actorPayload(center: Coord, centerHeight: number): Json {
    return {
        region_x: center[0],
        region_z: center[1],
        local_x_q9: 0,
        local_z_q9: 0,
        center_height_numerator: centerHeight,
        half_height_numerator: HALF_HEIGHT_Q16,
        step_velocity_q16: 0,
        ...SURVEY,
    };
}

async function groundedActor(center: Coord): Promise<Json> {
    const query = await event("canonical.terrain.height", {
        region_x: center[0],
        region_z: center[1],
        local_x_q9: 0,
        local_z_q9: 0,
    });
    return actorPayload(
        center,
        number(object(query, "height"), "heightNumerator") + HALF_HEIGHT_Q16,
    );
}

function requireAdvance(value: Json, presentationMutations: number, label: string): Json {
    if (
        value.revision !== REVISION || value.outcome !== "advanced" ||
        number(value, "preparedStepCount") !== 1 || number(value, "terrainQueryCount") !== 1 ||
        number(value, "scheduleCommitCount") !== 1 || number(value, "actorCommitCount") !== 1 ||
        number(value, "presentationMutationCount") !== presentationMutations ||
        number(value, "frameCount") !== 0 || number(value, "rendererWorkCount") !== 0
    ) fail(`${label} dual-commit evidence diverged`);
    return object(value, "actorSimulationAdvance");
}

function pendingInvariant(value: Json): Json {
    const pending = object(value, "pending");
    return {
        token: pending.token,
        config: pending.config,
        globalConfig: pending.globalConfig,
        terrainStage: pending.terrainStage,
        instanceStage: pending.instanceStage,
        cameraDriven: pending.cameraDriven,
        prefetch: pending.prefetch,
    };
}

async function stopProcess(): Promise<number> {
    const pid = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(pid);
    return pid;
}

async function prepublication(center: Coord): Promise<Json> {
    await event("workbench.pause");
    const before = await event("simulation.status");
    const stored = object(
        await event("actor.spawn", actorPayload(center, HALF_HEIGHT_Q16)),
        "actor",
    );
    const invalidPresentation = await rejectedEvent("simulation.actor.advance", {
        ...request(1, SHORT_STEP_NANOSECONDS, 0),
        archetype: 8,
    });
    if (
        typeof invalidPresentation.error !== "string" ||
        !invalidPresentation.error.includes("presentation archetype 8 exceeds catalog capacity")
    ) fail("invalid simulation presentation returned the wrong rejection");
    same(await event("simulation.status"), before, "invalid presentation schedule rollback");
    same(
        object(await event("actor.read", { generation: 1 }), "actor"),
        stored,
        "invalid presentation actor rollback",
    );
    const response = await event("simulation.actor.advance", {
        generation: 1,
        elapsed_nanoseconds: 1,
        delta_x_q9: 17,
        delta_z_q9: -19,
        step_up_limit_q16: 0,
        step_acceleration_q16: 0,
        ...WALK,
    });
    if (
        response.revision !== REVISION || response.outcome !== "advanced" ||
        number(response, "preparedStepCount") !== 0 ||
        number(response, "terrainQueryCount") !== 0 ||
        number(response, "scheduleCommitCount") !== 1 ||
        number(response, "actorCommitCount") !== 1 ||
        number(response, "presentationMutationCount") !== 0
    ) fail("prepublication actor admission changed the fractional commit");
    same(
        object(object(response, "actorSimulationAdvance"), "actor").input,
        stored,
        "prepublication actor input",
    );
    same(
        object(object(response, "actorSimulationAdvance"), "actor").output,
        stored,
        "prepublication actor output",
    );
    const after = await event("simulation.status");
    if (
        number(after, "tick") !== 0 || number(after, "remainderNumerator") !== 60 ||
        number(after, "successfulAdvanceCount") !== number(before, "successfulAdvanceCount") + 1
    ) fail("prepublication schedule commit diverged");
    await event("actor.despawn", { generation: 1 });
    return { before, invalidPresentation, response, after };
}

async function heldPending(
    terrain: string,
    objects: string,
    base: Coord,
    diagonal: Coord,
): Promise<Json> {
    await openSources(terrain, objects);
    const publication = await publish(target(base));
    await event("canonical.objects.io_gate.arm");
    const scheduled = await event("canonical.schedule", target(diagonal));
    const token = number(scheduled, "token");
    await event("workbench.resume");
    const held = await waitStatus("actor admission pending hold", (value) => {
        if (value.pending === null) return false;
        const pending = object(value, "pending");
        return number(pending, "token") === token && pending.terrainStage === "staged" &&
            pending.instanceStage === "in-flight";
    });
    await event("workbench.pause");

    const actor = object(await event("actor.spawn", await groundedActor(base)), "actor");
    const admitted = requireAdvance(
        await event(
            "simulation.actor.advance",
            request(2, SHORT_STEP_NANOSECONDS, 1, WALK),
        ),
        1,
        "shared-window actor",
    );
    const committed = object(object(admitted, "actor"), "output");
    const committedPosition = object(object(object(committed, "motion"), "body"), "position");
    if (
        number(object(committedPosition, "region"), "x") !== base[0] ||
        number(object(committedPosition, "region"), "z") !== base[1] ||
        number(committedPosition, "localXQ9") !== 1 || number(committedPosition, "localZQ9") !== 0
    ) fail("shared-window actor committed the wrong position");
    same(committed.handle, actor.handle, "shared-window actor handle");
    same(
        object(object(admitted, "actor"), "input").presentation,
        actor.presentation,
        "shared-window actor input presentation",
    );
    const committedPresentation = object(committed, "presentation");
    if (
        number(committedPresentation, "archetype") !== 7 ||
        number(committedPresentation, "material") !== 63 ||
        number(committedPresentation, "yawQ16") !== 0 ||
        number(committedPresentation, "animation") !== 1
    ) fail("shared-window actor presentation commit diverged");

    const actorBeforeBlock = object(await event("actor.read", { generation: 2 }), "actor");
    const simulationBeforeBlock = await event("simulation.status");
    const pendingBeforeBlock = await event("canonical.status");
    const blocked = await event(
        "simulation.actor.advance",
        request(2, LONG_STEP_NANOSECONDS, -4_098, SURVEY),
    );
    if (
        blocked.revision !== REVISION || blocked.outcome !== "render-blocked" ||
        number(blocked, "preparedStepCount") !== 1 ||
        number(blocked, "terrainQueryCount") !== 1 ||
        number(blocked, "scheduleCommitCount") !== 0 ||
        number(blocked, "actorCommitCount") !== 0 ||
        number(blocked, "presentationMutationCount") !== 0 || "actorSimulationAdvance" in blocked
    ) fail(`pending-window actor returned the wrong backpressure: ${JSON.stringify(blocked)}`);
    same(
        object(await event("actor.read", { generation: 2 }), "actor"),
        actorBeforeBlock,
        "pending-window actor rollback",
    );
    same(
        await event("simulation.status"),
        simulationBeforeBlock,
        "pending-window schedule rollback",
    );
    const pendingAfterBlock = await event("canonical.status");
    same(
        pendingInvariant(pendingAfterBlock),
        pendingInvariant(pendingBeforeBlock),
        "pending-window composition stability",
    );
    const retainedFrame = await event("canonical.probe");
    same(
        object(retainedFrame, "simulationSchedule"),
        simulationBeforeBlock,
        "pending-window retained frame schedule",
    );

    await event("canonical.objects.io_gate.release");
    await event("workbench.resume");
    const completed = await waitStatus(
        "actor admission pending release",
        (value) =>
            value.pending === null && value.published !== null &&
            number(object(value, "published"), "token") === token,
    );
    await event("workbench.pause");
    await event("actor.despawn", { generation: 2 });
    return {
        processId: await stopProcess(),
        publication,
        scheduled,
        held,
        actor,
        admitted,
        blocked,
        actorBeforeBlock,
        simulationBeforeBlock,
        pendingBeforeBlock,
        retainedFrame,
        completed,
    };
}

export async function actorRenderAdmissionGates(
    terrain: string,
    objects: string,
    base: Coord,
    diagonal: Coord,
): Promise<Json> {
    console.log("==> transactional simulation-actor render admission gates");
    await startClean();
    const beforePublication = await prepublication(base);
    const pending = await heldPending(terrain, objects, base, diagonal);
    return { beforePublication, pending };
}
