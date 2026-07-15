import { event, fail, type Json, number, object, rejectedEvent } from "../canonical-runtime.ts";

const I32_MAX = 2_147_483_647;
const HALF_HEIGHT = 65_536;

function requireContactRejection(value: Json, label: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("terrain_contact_failed: ")
    ) fail(`${label} returned the wrong terrain-contact rejection`);
}

function requireZeroRuntimeWork(value: Json, label: string): void {
    if (
        value.perResolutionAllocationBytes !== 0 || value.sourceReadCount !== 0 ||
        value.gpuCopyCount !== 0 || value.gpuReadbackCount !== 0 ||
        value.fenceWaitCount !== 0 || value.synchronizationCount !== 0
    ) fail(`${label} performed work outside the committed CPU snapshot`);
}

function payload(
    base: [number, number],
    local: number,
    centerHeightNumerator: number,
    halfHeightNumerator = HALF_HEIGHT,
): Json {
    return {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: local,
        local_z_q9: local,
        center_height_numerator: centerHeightNumerator,
        half_height_numerator: halfHeightNumerator,
    };
}

export async function unavailableTerrainContactGate(base: [number, number]): Promise<Json> {
    const unavailable = await rejectedEvent(
        "canonical.terrain.contact",
        payload(base, 0, HALF_HEIGHT),
    );
    requireContactRejection(unavailable, "pre-publication terrain contact");
    return unavailable;
}

export async function terrainContactGates(
    base: [number, number],
    controlledFrame: Json,
    unavailable: Json,
): Promise<Json> {
    const invalidZeroHalfHeight = await rejectedEvent(
        "canonical.terrain.contact",
        payload(base, 0, 0, 0),
    );
    const invalidNegativeHalfHeight = await rejectedEvent(
        "canonical.terrain.contact",
        payload(base, 0, 0, -1),
    );
    const invalidLocal = await rejectedEvent(
        "canonical.terrain.contact",
        payload(base, -4097, 0),
    );
    const outside = await rejectedEvent("canonical.terrain.contact", {
        ...payload(base, 0, HALF_HEIGHT),
        region_x: base[0] + 3,
    });
    for (
        const [label, value] of [
            ["zero half-height", invalidZeroHalfHeight],
            ["negative half-height", invalidNegativeHalfHeight],
            ["invalid horizontal bound", invalidLocal],
            ["outside active window", outside],
        ] as const
    ) requireContactRejection(value, label);

    let terrainSample: Json | undefined;
    for (const local of [-4032, -3904, 0, 4032]) {
        const candidate = await event("canonical.terrain.height", {
            region_x: base[0],
            region_z: base[1],
            local_x_q9: local,
            local_z_q9: local,
        });
        if (number(object(candidate, "height"), "heightNumerator") > 0) {
            terrainSample = candidate;
            break;
        }
    }
    if (!terrainSample) fail("controlled terrain did not expose a positive overflow sample");
    const terrainHeight = number(object(terrainSample, "height"), "heightNumerator");
    const terrainPosition = object(terrainSample, "position");
    const overflow = await rejectedEvent("canonical.terrain.contact", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: number(terrainPosition, "localXQ9"),
        local_z_q9: number(terrainPosition, "localZQ9"),
        center_height_numerator: I32_MAX,
        half_height_numerator: I32_MAX - terrainHeight + 1,
    });
    requireContactRejection(overflow, "unrepresentable resolved center");

    const samples: Json[] = [];
    const local = -3968;
    const terrain = await event("canonical.terrain.height", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: local,
        local_z_q9: local,
    });
    const height = number(object(terrain, "height"), "heightNumerator");
    for (
        const [classification, footOffset, correction] of [
            ["penetrating", -1, 1],
            ["touching", 0, 0],
            ["separated", 1, 0],
        ] as const
    ) {
        const value = await event(
            "canonical.terrain.contact",
            payload(base, local, height + HALF_HEIGHT + footOffset),
        );
        if (value.revision !== "exact-terrain-body-contact-v1") {
            fail(`${classification} terrain-contact revision diverged`);
        }
        const inputBody = object(value, "inputBody");
        const contact = object(value, "contact");
        const resolvedBody = object(contact, "resolvedBody");
        if (
            contact.classification !== classification ||
            number(contact, "separationNumerator") !== footOffset ||
            number(contact, "correctionNumerator") !== correction ||
            number(contact, "heightDenominator") !== 65_536 ||
            number(resolvedBody, "centerHeightNumerator") !==
                number(inputBody, "centerHeightNumerator") + correction ||
            number(resolvedBody, "halfHeightNumerator") !== HALF_HEIGHT
        ) fail(`${classification} exact terrain contact diverged`);
        requireZeroRuntimeWork(value, `${classification} terrain contact`);
        samples.push(value);
    }

    const witness = object(object(controlledFrame, "stable"), "terrainContactWitness");
    const witnessClassifications = object(witness, "classifications");
    if (
        witness.resultSha256 === witness.identityKeyedSha256 ||
        witnessClassifications.separated !== 75 || witnessClassifications.touching !== 75 ||
        witnessClassifications.penetrating !== 75 || witness.correctedCount !== 75
    ) fail("terrain body-contact transition witness diverged");

    return {
        unavailable,
        invalidZeroHalfHeight,
        invalidNegativeHalfHeight,
        invalidLocal,
        outside,
        overflow,
        samples,
        witness,
    };
}
