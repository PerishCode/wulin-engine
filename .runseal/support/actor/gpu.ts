import {
    array,
    capture,
    type Coord,
    event,
    fail,
    type Json,
    number,
    object,
    probe,
    rejectedEvent,
    same,
    status,
    string,
} from "../canonical-runtime.ts";

const ACTOR_CANDIDATE_INDEX = 25_600;
const VISIBLE_RECORD_BYTES = 56;
const FRAME_SLOTS = 2;
const HALF_HEIGHT_Q16 = 196_608;
const ACTOR_WORKLOAD_DELTA = {
    activePoses: 0,
    animated: 1,
    emittedTriangles: 576,
    emittedVertices: 1_014,
    evaluatedBones: 0,
    lodCounts: [1, 0, 0],
    meshlets: 16,
    observedArchetypeMask: 0,
    rejected: 0,
    reusedPoses: 1,
    skinInfluences: 4_056,
    staticCount: 0,
    visible: 1,
};
const REJECTED_WORKLOAD_DELTA = {
    ...ACTOR_WORKLOAD_DELTA,
    animated: 0,
    emittedTriangles: 0,
    emittedVertices: 0,
    lodCounts: [0, 0, 0],
    meshlets: 0,
    rejected: 1,
    reusedPoses: 0,
    skinInfluences: 0,
    visible: 0,
};
const FIRST_CAPTURE = {
    color: "d7fbc2e6c26cfe7f74a8b5751e1e6239f07f37f3e0204322032fe7ca4e50329e",
    png: "7e9d40d716870dd7a00dd54cebfd333ee9344de086093566b15b74b8e635962c",
    objectId: "a9fa7a8f1bec4fc5fe62dc7a969bcd87332b308352e59bf0616c2248841bcaf6",
    diagnostic: "eba4b91018819328e51b9a503ba00503887b3a4aa88bf3dd4426abc6d888b3bb",
};

type ActorState = {
    generation: number;
    probe: Json;
    actor: Json;
};

export async function actorGpuGates(base: Coord, collection: string): Promise<Json> {
    console.log("==> frame-safe actor GPU admission gates");
    const baseline = await probe();
    requireAbsent(baseline, "baseline");
    const baselineWrites = actorWriteCount(baseline);

    const firstPayload = await groundedActor(base, 0, 0, 63, 32_768, 1);
    const firstActor = object(await event("actor.spawn", firstPayload), "actor");
    const first = requireActor(await probe(), firstActor, 1, "first actor");
    const firstReplay = requireActor(await probe(), firstActor, 1, "first actor replay");
    requireWriteCount(first.probe, baselineWrites + 1, "first actor");
    requireWriteCount(firstReplay.probe, baselineWrites + 2, "first actor replay");
    same(
        actorCandidate(first.probe).recordWords,
        actorCandidate(firstReplay.probe).recordWords,
        "first actor immediate visible record replay",
    );
    const firstCapture = await capture("actor-generation-1", collection);
    requireSemanticActor(firstCapture, "first actor capture");
    same(captureHashes(firstCapture), FIRST_CAPTURE, "first actor exact capture");
    same(
        workloadDelta(baseline, first.probe),
        ACTOR_WORKLOAD_DELTA,
        "first actor exact GPU workload",
    );
    if (
        string(actorCandidate(first.probe), "recordSha256") !==
            "2f14c5bbe4268821f0cdd3cbc7a8fbce6d92c08c944476a91f82f353f505ac3a"
    ) fail("first actor visible-record hash diverged");

    await event("actor.despawn", { generation: 1 });
    const firstCleared = await probe();
    requireAbsent(firstCleared, "first actor cleared slot");
    requireWriteCount(firstCleared, baselineWrites + 4, "first actor cleared slot");

    const secondPayload = await groundedActor(base, 768, -512, 62, 49_152, (7 << 8) | 2);
    const secondActor = object(await event("actor.spawn", secondPayload), "actor");
    const second = requireActor(await probe(), secondActor, 2, "second actor");
    const secondReplay = requireActor(await probe(), secondActor, 2, "second actor replay");
    requireWriteCount(second.probe, baselineWrites + 5, "second actor");
    requireWriteCount(secondReplay.probe, baselineWrites + 6, "second actor replay");
    same(
        actorCandidate(second.probe).recordWords,
        actorCandidate(secondReplay.probe).recordWords,
        "second actor immediate visible record replay",
    );
    if (
        string(actorCandidate(first.probe), "recordSha256") ===
            string(actorCandidate(second.probe), "recordSha256")
    ) fail("actor generation/presentation replacement retained a stale GPU record");
    same(
        workloadDelta(baseline, second.probe),
        ACTOR_WORKLOAD_DELTA,
        "second actor exact GPU workload",
    );
    if (
        string(actorCandidate(second.probe), "recordSha256") !==
            "ab33d261aebcc263cf32a7849fb3637a2a82e37c130a021a1962327a8198dc78"
    ) fail("second actor visible-record hash diverged");

    await event("actor.despawn", { generation: 2 });
    const secondCleared = await probe();
    requireAbsent(secondCleared, "second actor cleared slot");
    requireWriteCount(secondCleared, baselineWrites + 7, "second actor cleared slot");

    const rejectedPosition: Coord = [base[0] + 2, base[1] + 2];
    const rejectedPayload = await groundedActor(rejectedPosition, 4_095, 4_095, 61, 0, 1);
    const rejectedActor = object(await event("actor.spawn", rejectedPayload), "actor");
    const rejected = requireActor(await probe(), rejectedActor, 3, "frustum-rejected actor", false);
    requireWriteCount(rejected.probe, baselineWrites + 8, "frustum-rejected actor");
    same(
        workloadDelta(baseline, rejected.probe),
        REJECTED_WORKLOAD_DELTA,
        "frustum-rejected actor exact GPU workload",
    );
    await event("actor.despawn", { generation: 3 });

    const outsidePayload = { ...firstPayload, region_x: base[0] + 3 };
    const outsideActor = object(await event("actor.spawn", outsidePayload), "actor");
    const beforeOutside = await status();
    const outside = await rejectedEvent("canonical.probe");
    if (
        typeof outside.error !== "string" ||
        !outside.error.startsWith("render_failed: ") ||
        !outside.error.includes("outside the active render window")
    ) fail(`outside-window frame returned the wrong rejection: ${JSON.stringify(outside)}`);
    const afterOutside = await status();
    if (
        number(afterOutside, "frameIndex") !== number(beforeOutside, "frameIndex") ||
        afterOutside.state !== "paused" ||
        typeof afterOutside.lastError !== "string" ||
        !afterOutside.lastError.includes("outside the active render window")
    ) fail("outside-window frame rejection changed the submitted frame transaction");
    same(
        object(await event("actor.read", { generation: 4 }), "actor"),
        outsideActor,
        "outside-window frame actor rollback",
    );
    await event("actor.despawn", { generation: 4 });
    const final = await probe();
    requireAbsent(final, "final cleared slot");
    requireWriteCount(final, baselineWrites + 9, "final cleared slot");

    return {
        baseline,
        first,
        firstReplay,
        firstCapture,
        firstCleared,
        second,
        secondReplay,
        secondCleared,
        rejected,
        outside: {
            actor: outsideActor,
            response: outside,
            beforeFrameIndex: number(beforeOutside, "frameIndex"),
            afterFrameIndex: number(afterOutside, "frameIndex"),
            afterState: afterOutside.state,
            afterLastError: afterOutside.lastError,
        },
        final,
    };
}

async function groundedActor(
    region: Coord,
    localXQ9: number,
    localZQ9: number,
    material: number,
    yawQ16: number,
    animation: number,
): Promise<Json> {
    const query = await event("canonical.terrain.height", {
        region_x: region[0],
        region_z: region[1],
        local_x_q9: localXQ9,
        local_z_q9: localZQ9,
    });
    const ground = number(object(query, "height"), "heightNumerator");
    return {
        region_x: region[0],
        region_z: region[1],
        local_x_q9: localXQ9,
        local_z_q9: localZQ9,
        center_height_numerator: ground + HALF_HEIGHT_Q16,
        half_height_numerator: HALF_HEIGHT_Q16,
        step_velocity_q16: 0,
        archetype: 7,
        material,
        yaw_q16: yawQ16,
        animation,
    };
}

function requireActor(
    value: Json,
    actor: Json,
    generation: number,
    label: string,
    visible = true,
): ActorState {
    const surface = object(value, "surface");
    const skeletal = object(surface, "skeletal");
    if (
        number(skeletal, "streamedCandidateCount") !== 25_600 ||
        number(skeletal, "dynamicCandidateCount") !== 1 ||
        number(skeletal, "candidateCapacity") !== 25_601 ||
        number(skeletal, "visibleRecordBytes") !== VISIBLE_RECORD_BYTES ||
        number(skeletal, "actorUploadRecordBytes") !== VISIBLE_RECORD_BYTES ||
        number(skeletal, "actorUploadFrameSlots") !== FRAME_SLOTS ||
        number(skeletal, "actorUploadAllocationBytes") !== VISIBLE_RECORD_BYTES * FRAME_SLOTS ||
        number(skeletal, "actorUploadResourceCount") !== 1 ||
        number(skeletal, "actorUploadWriteCount") < 1 ||
        number(skeletal, "actorUploadGpuCopyCount") !== 0
    ) fail(`${label} actor upload ownership diverged`);
    const candidate = actorCandidate(value);
    if (
        number(candidate, "candidateIndex") !== ACTOR_CANDIDATE_INDEX ||
        string(candidate, "generationDecimal") !== String(generation) ||
        number(candidate, "uploadRecordBytes") !== VISIBLE_RECORD_BYTES ||
        candidate.sourceVisible !== visible ||
        number(candidate, "exactFieldMismatchCount") !== 0
    ) fail(`${label} candidate evidence diverged`);
    same(candidate.stableIdentity, [generation, 0], `${label} full generation identity`);
    if (visible) {
        if (
            candidate.sourceVisibleIndex === null || candidate.recordWords === null ||
            typeof candidate.recordSha256 !== "string" ||
            ![1, 2].includes(number(candidate, "occlusionMask"))
        ) fail(`${label} did not enter the canonical visible record`);
        if ((candidate.occlusionMask === 1) !== (candidate.filteredVisibleIndex !== null)) {
            fail(`${label} filtered record disagreed with its oracle classification`);
        }
    } else if (
        candidate.sourceVisibleIndex !== null || candidate.filteredVisibleIndex !== null ||
        candidate.recordWords !== null || candidate.recordSha256 !== null ||
        number(candidate, "occlusionMask") !== 0
    ) fail(`${label} leaked past frustum rejection`);
    same(
        number(object(actor, "handle"), "generation"),
        generation,
        `${label} authoritative generation`,
    );
    return { generation, probe: value, actor };
}

function actorWriteCount(value: Json): number {
    return number(object(object(value, "surface"), "skeletal"), "actorUploadWriteCount");
}

function requireWriteCount(value: Json, expected: number, label: string): void {
    if (actorWriteCount(value) !== expected) fail(`${label} actor upload write count diverged`);
}

function requireAbsent(value: Json, label: string): void {
    const skeletal = object(object(value, "surface"), "skeletal");
    if (
        number(skeletal, "dynamicCandidateCount") !== 0 ||
        number(skeletal, "candidateCapacity") !== 25_601 ||
        object(value, "surface").occlusion === undefined ||
        object(object(value, "surface"), "occlusion").actorCandidate !== null
    ) fail(`${label} retained an actor GPU candidate`);
}

function actorCandidate(value: Json): Json {
    return object(object(object(value, "surface"), "occlusion"), "actorCandidate");
}

function requireSemanticActor(value: Json, label: string): void {
    const visible = array(value, "visible") as Json[];
    const actors = visible.filter((entry) => entry.kind === "runtime-actor");
    if (actors.length !== 1 || actors[0].name !== "runtime.actor") {
        fail(`${label} did not expose exactly one runtime actor semantic ID`);
    }
}

function workloadDelta(baseline: Json, actor: Json): Json {
    const before = object(object(baseline, "surface"), "skeletal").gpu as Json;
    const after = object(object(actor, "surface"), "skeletal").gpu as Json;
    const delta: Json = {};
    for (const key of Object.keys(ACTOR_WORKLOAD_DELTA)) {
        if (key === "lodCounts") {
            const beforeLods = array(before, key) as number[];
            const afterLods = array(after, key) as number[];
            delta[key] = afterLods.map((value, index) => value - beforeLods[index]);
        } else if (key === "observedArchetypeMask") {
            delta[key] = number(after, key) ^ number(before, key);
        } else {
            delta[key] = number(after, key) - number(before, key);
        }
    }
    return delta;
}

function captureHashes(value: Json): Json {
    return {
        color: string(value, "color"),
        png: string(value, "png"),
        objectId: string(value, "objectId"),
        diagnostic: string(value, "diagnostic"),
    };
}
