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

const REVISION = "bounded-terrain-body-translation-v1";
const HALF_HEIGHT = 65_536;
const REGION_SIDE_Q9 = 8192;
const I32_MAX = 2_147_483_647;

type Sample = {
    regionX: number;
    regionZ: number;
    localXQ9: number;
    localZQ9: number;
    height: number;
};

function payload(
    sample: Sample,
    center: number,
    velocity: number,
    deltaXQ9: number,
    deltaZQ9: number,
    stepUpLimitQ16: number,
): Json {
    return {
        region_x: sample.regionX,
        region_z: sample.regionZ,
        local_x_q9: sample.localXQ9,
        local_z_q9: sample.localZQ9,
        center_height_numerator: center,
        half_height_numerator: HALF_HEIGHT,
        step_velocity_q16: velocity,
        delta_x_q9: deltaXQ9,
        delta_z_q9: deltaZQ9,
        step_up_limit_q16: stepUpLimitQ16,
    };
}

function requireRejection(value: Json, label: string, detail?: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("terrain_translation_failed: ") ||
        (detail !== undefined && !value.error.includes(detail))
    ) fail(`${label} returned the wrong terrain-translation rejection`);
}

function requireTranslation(response: Json, label: string): Json {
    if (
        response.revision !== REVISION || response.terrainQueryCount !== 1 ||
        response.perTranslationAllocationBytes !== 0 || response.sourceReadCount !== 0 ||
        response.gpuCopyCount !== 0 || response.gpuReadbackCount !== 0 ||
        response.fenceWaitCount !== 0 || response.synchronizationCount !== 0 ||
        response.scheduleMutationCount !== 0 || response.presentationMutationCount !== 0 ||
        response.frameCount !== 0 || response.rendererWorkCount !== 0
    ) fail(`${label} performed work outside one CPU terrain translation`);
    const translation = object(response, "translation");
    if (
        number(translation, "positionDenominator") !== 512 ||
        number(translation, "heightDenominator") !== 65_536
    ) fail(`${label} returned invalid fixed-point authority`);
    return translation;
}

async function translate(request: Json, label: string): Promise<Json> {
    return requireTranslation(
        await event("canonical.terrain.body.translate", request),
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

async function slope(base: [number, number]): Promise<{ lower: Sample; upper: Sample }> {
    for (const fixed of [-3968, 0, 3968]) {
        let previous = await sample(base[0], base[1], -4032, fixed);
        for (let local = -3904; local <= 4032; local += 128) {
            const current = await sample(base[0], base[1], local, fixed);
            if (current.height !== previous.height) {
                return current.height > previous.height
                    ? { lower: previous, upper: current }
                    : { lower: current, upper: previous };
            }
            previous = current;
        }
    }
    for (const fixed of [-3968, 0, 3968]) {
        let previous = await sample(base[0], base[1], fixed, -4032);
        for (let local = -3904; local <= 4032; local += 128) {
            const current = await sample(base[0], base[1], fixed, local);
            if (current.height !== previous.height) {
                return current.height > previous.height
                    ? { lower: previous, upper: current }
                    : { lower: current, upper: previous };
            }
            previous = current;
        }
    }
    fail("controlled terrain exposed no nonzero adjacent rise");
}

async function positiveSample(base: [number, number]): Promise<Sample> {
    for (const local of [-4032, -3904, 0, 4032]) {
        const value = await sample(base[0], base[1], local, local);
        if (value.height > 0) return value;
    }
    fail("controlled terrain exposed no positive contact-overflow sample");
}

function displacement(source: Sample, destination: Sample): [number, number] {
    return [
        (destination.regionX - source.regionX) * REGION_SIDE_Q9 + destination.localXQ9 -
        source.localXQ9,
        (destination.regionZ - source.regionZ) * REGION_SIDE_Q9 + destination.localZQ9 -
        source.localZQ9,
    ];
}

function requireBodyPosition(body: Json, expected: Sample, label: string): void {
    const position = object(body, "position");
    const region = object(position, "region");
    if (
        number(region, "x") !== expected.regionX || number(region, "z") !== expected.regionZ ||
        number(position, "localXQ9") !== expected.localXQ9 ||
        number(position, "localZQ9") !== expected.localZQ9
    ) fail(`${label} returned the wrong canonical position`);
}

async function cases(lower: Sample, upper: Sample, seam: [Sample, Sample]): Promise<Json> {
    const rise = upper.height - lower.height;
    const [upX, upZ] = displacement(lower, upper);
    const equal = await translate(
        payload(lower, lower.height + HALF_HEIGHT, -777, upX, upZ, rise),
        "equal-limit uphill",
    );
    const below = await translate(
        payload(lower, lower.height + HALF_HEIGHT, -777, upX, upZ, rise + 1),
        "below-limit uphill",
    );
    for (const [label, value] of [["equal", equal], ["below", below]] as const) {
        const contact = object(value, "contact");
        const output = object(value, "output");
        if (
            value.blocked !== false || number(contact, "correctionNumerator") !== rise ||
            number(output, "stepVelocityQ16") !== -777 ||
            number(object(output, "body"), "centerHeightNumerator") !== upper.height + HALF_HEIGHT
        ) fail(`${label}-limit uphill translation diverged`);
        requireBodyPosition(object(output, "body"), upper, `${label}-limit uphill`);
    }

    const blocked = await translate(
        payload(lower, lower.height + HALF_HEIGHT, -777, upX, upZ, rise - 1),
        "blocked uphill",
    );
    if (
        blocked.blocked !== true ||
        number(object(blocked, "contact"), "correctionNumerator") !== rise
    ) fail("excess uphill correction was not blocked");
    same(object(blocked, "output"), object(blocked, "input"), "blocked output identity");

    const zero = await translate(
        payload(lower, lower.height + HALF_HEIGHT, -777, 0, 0, 0),
        "zero-limit touching",
    );
    if (zero.blocked !== false || object(zero, "contact").classification !== "touching") {
        fail("zero-limit touching translation diverged");
    }

    const [downX, downZ] = displacement(upper, lower);
    const downhill = await translate(
        payload(upper, upper.height + HALF_HEIGHT, 333, downX, downZ, 0),
        "downhill",
    );
    const downhillContact = object(downhill, "contact");
    const downhillOutput = object(downhill, "output");
    if (
        downhill.blocked !== false || downhillContact.classification !== "separated" ||
        number(downhillContact, "separationNumerator") !== rise ||
        number(object(downhillOutput, "body"), "centerHeightNumerator") !==
            upper.height + HALF_HEIGHT ||
        number(downhillOutput, "stepVelocityQ16") !== 333
    ) fail("downhill translation snapped or changed velocity");
    requireBodyPosition(object(downhillOutput, "body"), lower, "downhill");

    const seamCenter = Math.max(seam[0].height, seam[1].height) + HALF_HEIGHT + 1;
    const [seamX, seamZ] = displacement(seam[0], seam[1]);
    const crossed = await translate(
        payload(seam[0], seamCenter, 99, seamX, seamZ, 0),
        "signed seam",
    );
    if (crossed.blocked !== false) fail("signed seam translation was blocked");
    requireBodyPosition(object(object(crossed, "output"), "body"), seam[1], "signed seam");
    const returned = await translate(
        payload(seam[1], seamCenter, 99, -seamX, -seamZ, 0),
        "partitioned return",
    );
    const direct = await translate(
        payload(seam[0], seamCenter, 99, 0, 0, 0),
        "direct return",
    );
    same(object(returned, "output"), object(direct, "output"), "translation partition output");

    return { rise, equal, below, blocked, zero, downhill, crossed, returned, direct };
}

async function sha256(value: unknown): Promise<string> {
    const bytes = new TextEncoder().encode(JSON.stringify(value));
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join(
        "",
    );
}

export async function terrainTranslationGates(
    terrainPath: string,
    objectPath: string,
    base: [number, number],
): Promise<Json> {
    console.log("==> bounded terrain-body translation gates");
    const started = performance.now();
    await startClean();
    await event("workbench.pause");
    const empty: Sample = {
        regionX: base[0],
        regionZ: base[1],
        localXQ9: 0,
        localZQ9: 0,
        height: 0,
    };
    const negativeLimit = await rejectedEvent(
        "canonical.terrain.body.translate",
        payload(empty, HALF_HEIGHT, 0, 0, 0, -1),
    );
    requireRejection(negativeLimit, "negative limit", "step-up limit");
    const unavailable = await rejectedEvent(
        "canonical.terrain.body.translate",
        payload(empty, HALF_HEIGHT, 0, 0, 0, 0),
    );
    requireRejection(unavailable, "pre-publication translation");
    const malformed = await rejectedEvent("canonical.terrain.body.translate", {
        ...payload(empty, HALF_HEIGHT, 0, 0, 0, 0),
        step_up_limit_q16: "invalid",
    });
    if (typeof malformed.error !== "string" || !malformed.error.startsWith("invalid_payload: ")) {
        fail("malformed terrain translation returned the wrong rejection");
    }

    await openSources(terrainPath, objectPath);
    const publication = await publish(target(base));
    const beforeSchedule = await event("simulation.status");
    const beforePresentation = await event("canonical.time.status");
    const controlledSlope = await slope(base);
    const seam: [Sample, Sample] = [
        await sample(base[0], base[1], 4095, 0),
        await sample(base[0] + 1, base[1], -4096, 0),
    ];
    const evidence = await cases(controlledSlope.lower, controlledSlope.upper, seam);
    const replay = await cases(controlledSlope.lower, controlledSlope.upper, seam);
    const resultSha256 = await sha256(evidence);
    const replaySha256 = await sha256(replay);
    if (resultSha256 !== replaySha256) fail("terrain translation replay hash diverged");

    const outside = await rejectedEvent(
        "canonical.terrain.body.translate",
        payload(empty, HALF_HEIGHT, 0, 3 * REGION_SIDE_Q9, 0, 0),
    );
    requireRejection(outside, "outside-snapshot translation");
    const positive = await positiveSample(base);
    const unrepresentable = await rejectedEvent("canonical.terrain.body.translate", {
        ...payload(positive, I32_MAX, 0, 0, 0, I32_MAX),
        half_height_numerator: I32_MAX,
    });
    requireRejection(unrepresentable, "unrepresentable contact");
    same(await event("simulation.status"), beforeSchedule, "translation schedule independence");
    same(
        await event("canonical.time.status"),
        beforePresentation,
        "translation presentation independence",
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
        slope: controlledSlope,
        seam,
        evidence,
        outside,
        unrepresentable,
        resultSha256,
        replaySha256,
        processId,
        elapsedMilliseconds: performance.now() - started,
    };
}
