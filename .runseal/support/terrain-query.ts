import { event, fail, type Json, number, object, rejectedEvent } from "./canonical-runtime.ts";

function requireQueryRejection(value: Json, label: string): void {
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("terrain_query_failed: ")
    ) {
        fail(`${label} returned the wrong terrain-query rejection`);
    }
}

function requireZeroRuntimeWork(value: Json, label: string): void {
    if (
        value.perQueryAllocationBytes !== 0 || value.sourceReadCount !== 0 ||
        value.gpuCopyCount !== 0 ||
        value.gpuReadbackCount !== 0 || value.fenceWaitCount !== 0 ||
        value.synchronizationCount !== 0
    ) fail(`${label} performed runtime work outside the published CPU snapshot`);
}

export async function unavailableTerrainQueryGate(base: [number, number]): Promise<Json> {
    const unavailable = await rejectedEvent("canonical.terrain.height", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
    });
    requireQueryRejection(unavailable, "pre-publication terrain query");
    return unavailable;
}

export async function terrainQueryGates(
    base: [number, number],
    controlledFrame: Json,
    unavailable: Json,
): Promise<Json> {
    const invalidNegative = await rejectedEvent("canonical.terrain.height", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: -4097,
        local_z_q9: 0,
    });
    const invalidPositive = await rejectedEvent("canonical.terrain.height", {
        region_x: base[0],
        region_z: base[1],
        local_x_q9: 4096,
        local_z_q9: 0,
    });
    const outside = await rejectedEvent("canonical.terrain.height", {
        region_x: base[0] + 3,
        region_z: base[1],
        local_x_q9: 0,
        local_z_q9: 0,
    });
    for (
        const [label, value] of [
            ["negative local bound", invalidNegative],
            ["positive local bound", invalidPositive],
            ["outside active window", outside],
        ] as const
    ) requireQueryRejection(value, label);

    const samples: Json[] = [];
    for (
        const [triangle, coordinate] of [
            ["first", -4032],
            ["diagonal", -3968],
            ["second", -3904],
        ] as const
    ) {
        const value = await event("canonical.terrain.height", {
            region_x: base[0],
            region_z: base[1],
            local_x_q9: coordinate,
            local_z_q9: coordinate,
        });
        if (value.revision !== "exact-canonical-terrain-query-v1") {
            fail(`${triangle} terrain query revision diverged`);
        }
        const height = object(value, "height");
        if (
            number(height, "heightDenominator") !== 65_536 || height.triangle !== triangle ||
            !Number.isInteger(number(height, "heightNumerator"))
        ) fail(`${triangle} terrain query result diverged`);
        requireZeroRuntimeWork(value, `${triangle} terrain query`);
        samples.push(value);
    }

    const seam = await event("canonical.terrain.height", {
        region_x: base[0] + 1,
        region_z: base[1],
        local_x_q9: -4096,
        local_z_q9: 0,
    });
    requireZeroRuntimeWork(seam, "adjacent-region seam query");
    const seamPosition = object(seam, "position");
    if (
        number(object(seamPosition, "region"), "x") !== base[0] + 1 ||
        number(seamPosition, "localXQ9") !== -4096
    ) fail("positive seam did not belong to the adjacent signed region");

    const dense = object(object(controlledFrame, "stable"), "terrainQuery");
    const triangles = object(dense, "triangles");
    if (
        dense.resultSha256 === dense.identityKeyedSha256 ||
        triangles.first !== 25_600 || triangles.diagonal !== 25_600 ||
        triangles.second !== 25_600
    ) fail("dense terrain query evidence diverged");

    return { unavailable, invalidNegative, invalidPositive, outside, samples, seam, dense };
}
