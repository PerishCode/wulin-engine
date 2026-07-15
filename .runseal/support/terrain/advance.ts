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

const REVISION = "planar-first-terrain-body-advance-v1";
const HALF_HEIGHT = 65_536;
const REGION_SIDE_Q9 = 8192;
const I32_MAX = 2_147_483_647;
const ACCELERATION = -10;

type Sample = {
    regionX: number;
    regionZ: number;
    localXQ9: number;
    localZQ9: number;
    height: number;
};

type Motion = {
    sample: Sample;
    center: number;
    velocity: number;
};

function payload(
    motion: Motion,
    deltaXQ9: number,
    deltaZQ9: number,
    limit: number,
    acceleration = ACCELERATION,
): Json {
    return {
        region_x: motion.sample.regionX,
        region_z: motion.sample.regionZ,
        local_x_q9: motion.sample.localXQ9,
        local_z_q9: motion.sample.localZQ9,
        center_height_numerator: motion.center,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: motion.velocity,
        delta_x_q9: deltaXQ9,
        delta_z_q9: deltaZQ9,
        step_up_limit_q16: limit,
        step_acceleration_q16: acceleration,
    };
}

function requireRejection(value: Json, label: string, detail?: string): void {
    if (
        typeof value.error !== "string" || !value.error.startsWith("terrain_advance_failed: ") ||
        (detail !== undefined && !value.error.includes(detail))
    ) fail(`${label} returned the wrong terrain-advance rejection`);
}

function requireAdvance(response: Json, expectedQueries: number, label: string): Json {
    if (
        response.revision !== REVISION || response.perAdvanceAllocationBytes !== 0 ||
        response.sourceReadCount !== 0 || response.gpuCopyCount !== 0 ||
        response.gpuReadbackCount !== 0 || response.fenceWaitCount !== 0 ||
        response.synchronizationCount !== 0 || response.scheduleMutationCount !== 0 ||
        response.presentationMutationCount !== 0 || response.frameCount !== 0 ||
        response.rendererWorkCount !== 0
    ) fail(`${label} performed work outside one CPU terrain-body advance`);
    const advance = object(response, "advance");
    if (
        number(advance, "terrainQueryCount") !== expectedQueries ||
        number(advance, "positionDenominator") !== 512 ||
        number(advance, "heightDenominator") !== 65_536 ||
        number(advance, "stepsPerSecond") !== 60
    ) fail(`${label} returned invalid query/fixed-step authority`);
    return advance;
}

async function advance(request: Json, expectedQueries: number, label: string): Promise<Json> {
    return requireAdvance(
        await event("canonical.terrain.body.advance", request),
        expectedQueries,
        label,
    );
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

function displacement(source: Sample, destination: Sample): [number, number] {
    return [
        (destination.regionX - source.regionX) * REGION_SIDE_Q9 + destination.localXQ9 -
        source.localXQ9,
        (destination.regionZ - source.regionZ) * REGION_SIDE_Q9 + destination.localZQ9 -
        source.localZQ9,
    ];
}

function requirePosition(body: Json, expected: Sample, label: string): void {
    const position = object(body, "position");
    const region = object(position, "region");
    if (
        number(region, "x") !== expected.regionX || number(region, "z") !== expected.regionZ ||
        number(position, "localXQ9") !== expected.localXQ9 ||
        number(position, "localZQ9") !== expected.localZQ9
    ) fail(`${label} returned the wrong final position`);
}

async function directCases(lower: Sample, upper: Sample, seam: [Sample, Sample]): Promise<Json> {
    const rise = upper.height - lower.height;
    const [upX, upZ] = displacement(lower, upper);
    const accepted = await advance(
        payload({ sample: lower, center: lower.height + HALF_HEIGHT, velocity: 0 }, upX, upZ, rise),
        1,
        "accepted uphill",
    );
    if (
        object(accepted, "translation").blocked !== false || accepted.grounded !== true ||
        number(object(accepted, "output"), "stepVelocityQ16") !== 0
    ) fail("accepted uphill advance diverged");
    requirePosition(object(object(accepted, "output"), "body"), upper, "accepted uphill");

    const blocked = await advance(
        payload(
            { sample: lower, center: lower.height + HALF_HEIGHT, velocity: 0 },
            upX,
            upZ,
            rise - 1,
        ),
        2,
        "blocked uphill",
    );
    const blockedTranslation = object(blocked, "translation");
    if (blockedTranslation.blocked !== true || blocked.grounded !== true) {
        fail("blocked uphill did not retain planar input and run vertical grounding");
    }
    same(
        object(blockedTranslation, "output"),
        object(blockedTranslation, "input"),
        "blocked planar identity",
    );
    requirePosition(object(object(blocked, "output"), "body"), lower, "blocked uphill");

    const [downX, downZ] = displacement(upper, lower);
    const downhill = await advance(
        payload(
            { sample: upper, center: upper.height + HALF_HEIGHT, velocity: 0 },
            downX,
            downZ,
            0,
        ),
        1,
        "same-tick downhill",
    );
    if (
        object(object(downhill, "translation"), "contact").classification !== "separated" ||
        downhill.grounded !== false ||
        number(object(downhill, "output"), "stepVelocityQ16") !== ACCELERATION ||
        number(object(object(downhill, "output"), "body"), "centerHeightNumerator") !==
            upper.height + HALF_HEIGHT + ACCELERATION
    ) fail("downhill advance did not begin falling in the same tick");
    requirePosition(object(object(downhill, "output"), "body"), lower, "same-tick downhill");

    const zero = await advance(
        payload({ sample: lower, center: lower.height + HALF_HEIGHT, velocity: 0 }, 0, 0, 0),
        1,
        "zero displacement",
    );
    if (zero.grounded !== true) fail("zero-displacement advance did not remain grounded");

    const seamCenter = Math.max(seam[0].height, seam[1].height) + HALF_HEIGHT + 100;
    const [seamX, seamZ] = displacement(seam[0], seam[1]);
    const crossed = await advance(
        payload({ sample: seam[0], center: seamCenter, velocity: 10 }, seamX, seamZ, 0),
        1,
        "signed seam",
    );
    requirePosition(object(object(crossed, "output"), "body"), seam[1], "signed seam");

    return { rise, accepted, blocked, downhill, zero, crossed };
}

async function sequence(lower: Sample, upper: Sample, groups: number[]): Promise<Json> {
    const rise = upper.height - lower.height;
    let motion: Motion = { sample: lower, center: lower.height + HALF_HEIGHT, velocity: 0 };
    const steps: Json[] = [];
    for (const group of groups) {
        for (let index = 0; index < group; index += 1) {
            const destination = motion.sample.localXQ9 === lower.localXQ9 ? upper : lower;
            const [deltaX, deltaZ] = displacement(motion.sample, destination);
            const step = await advance(payload(motion, deltaX, deltaZ, rise), 1, "sequence step");
            const output = object(step, "output");
            const body = object(output, "body");
            requirePosition(body, destination, "sequence step");
            motion = {
                sample: destination,
                center: number(body, "centerHeightNumerator"),
                velocity: number(output, "stepVelocityQ16"),
            };
            steps.push(step);
        }
    }
    if (steps.length !== 60) fail("terrain-body advance sequence did not execute 60 steps");
    return { steps, finalMotion: motion, stepSha256: await sha256(steps) };
}

async function sha256(value: unknown): Promise<string> {
    const bytes = new TextEncoder().encode(JSON.stringify(value));
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function terrainAdvanceGates(
    terrainPath: string,
    objectPath: string,
    base: [number, number],
): Promise<Json> {
    console.log("==> planar-first terrain-body advance gates");
    const started = performance.now();
    await startClean();
    await event("workbench.pause");
    const empty: Sample = {
        regionX: base[0],
        regionZ: base[1],
        localXQ9: -3904,
        localZQ9: -3968,
        height: 0,
    };
    const initial: Motion = { sample: empty, center: HALF_HEIGHT, velocity: 0 };
    const negativeLimit = await rejectedEvent(
        "canonical.terrain.body.advance",
        payload(initial, 0, 0, -1),
    );
    requireRejection(negativeLimit, "negative limit", "step-up limit");
    const unavailable = await rejectedEvent(
        "canonical.terrain.body.advance",
        payload(initial, 0, 0, 0),
    );
    requireRejection(unavailable, "pre-publication advance");
    const malformed = await rejectedEvent("canonical.terrain.body.advance", {
        ...payload(initial, 0, 0, 0),
        step_acceleration_q16: "invalid",
    });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("malformed terrain-body advance returned the wrong rejection");
    }

    await openSources(terrainPath, objectPath);
    const publication = await publish(target(base));
    const beforeSchedule = await event("simulation.status");
    const beforePresentation = await event("canonical.time.status");
    const lower = await sample(base[0], base[1], -3904, -3968);
    const upper = await sample(base[0], base[1], -3776, -3968);
    if (upper.height <= lower.height) fail("controlled terrain advance rise is not positive");
    const seam: [Sample, Sample] = [
        await sample(base[0], base[1], 4095, 0),
        await sample(base[0] + 1, base[1], -4096, 0),
    ];
    const direct = await directCases(lower, upper, seam);
    const directReplay = await directCases(lower, upper, seam);
    const coarse = await sequence(lower, upper, [8, 8, 8, 8, 7, 7, 7, 7]);
    const nominal = await sequence(lower, upper, Array(60).fill(1));
    same(coarse.finalMotion, nominal.finalMotion, "advance batch final motion");
    if (coarse.stepSha256 !== nominal.stepSha256) fail("advance batch step hash diverged");
    const resultSha256 = await sha256({ direct, coarse });
    const replaySha256 = await sha256({ direct: directReplay, coarse: nominal });
    if (resultSha256 !== replaySha256) fail("terrain-body advance replay hash diverged");

    const outside = await rejectedEvent(
        "canonical.terrain.body.advance",
        payload(
            { sample: lower, center: lower.height + HALF_HEIGHT, velocity: 0 },
            3 * REGION_SIDE_Q9,
            0,
            0,
        ),
    );
    requireRejection(outside, "outside destination");
    const retainedOutside: Sample = { ...lower, regionX: base[0] + 3 };
    const [returnX, returnZ] = displacement(retainedOutside, lower);
    const outsideOrigin = await rejectedEvent(
        "canonical.terrain.body.advance",
        payload(
            { sample: retainedOutside, center: lower.height + HALF_HEIGHT - 1, velocity: 0 },
            returnX,
            returnZ,
            0,
        ),
    );
    requireRejection(outsideOrigin, "blocked retained origin");
    const velocityOverflow = await rejectedEvent(
        "canonical.terrain.body.advance",
        payload(
            { sample: lower, center: lower.height + HALF_HEIGHT, velocity: I32_MAX },
            0,
            0,
            0,
            1,
        ),
    );
    requireRejection(velocityOverflow, "vertical velocity overflow");
    const contactOverflow = await rejectedEvent("canonical.terrain.body.advance", {
        ...payload({ sample: upper, center: I32_MAX, velocity: 0 }, 0, 0, I32_MAX, 0),
        half_height_numerator: I32_MAX,
    });
    requireRejection(contactOverflow, "contact overflow");
    same(await event("simulation.status"), beforeSchedule, "advance schedule independence");
    same(
        await event("canonical.time.status"),
        beforePresentation,
        "advance presentation independence",
    );

    const processId = number(await status(), "processId");
    await lifecycle("stop");
    await assertStopped(processId);
    await lifecycle("start");
    return {
        negativeLimit,
        unavailable,
        malformed,
        publication,
        lower,
        upper,
        seam,
        direct,
        coarse,
        nominal,
        outside,
        outsideOrigin,
        velocityOverflow,
        contactOverflow,
        resultSha256,
        replaySha256,
        processId,
        elapsedMilliseconds: performance.now() - started,
    };
}
