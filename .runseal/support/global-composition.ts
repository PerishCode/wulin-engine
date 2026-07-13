import { captureEvidence, fail, field, object, same } from "./terrain.ts";
import { stableLodCompositionProbe, validateLodCompositionProbe } from "./composition.ts";
import { event, type GlobalConfig, globalConfig, sleep } from "./global-terrain.ts";

export { globalConfig };
export type { GlobalConfig };

export function localConfig(config: GlobalConfig): Record<string, number> {
    return {
        worldRegionSide: 128,
        activeCenterX: 64 + config.center_x - config.origin_x,
        activeCenterZ: 64 + config.center_z - config.origin_z,
        activeRadius: config.active_radius,
    };
}

function sameGlobal(actual: Record<string, unknown>, expected: GlobalConfig): boolean {
    const origin = object(actual, "globalOrigin");
    const center = object(actual, "globalCenter");
    return origin.x === expected.origin_x && origin.z === expected.origin_z &&
        center.x === expected.center_x && center.z === expected.center_z &&
        actual.activeRadius === expected.active_radius;
}

export async function waitPair(token: number): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        if (
            status.pending === null && status.published &&
            object(status, "published").token === token
        ) return status;
        await sleep(10);
    }
    fail(`global composition token ${token} publication timed out`);
}

export async function publishPair(config: GlobalConfig): Promise<Record<string, unknown>> {
    const started = performance.now();
    const scheduled = await event("composition.global.schedule", config);
    const token = field<number>(scheduled, "token", "number");
    await event("workbench.resume");
    const status = await waitPair(token);
    await event("workbench.pause");
    const published = object(status, "published");
    validatePair(published, config);
    return {
        scheduled,
        published,
        halves: await halfReports(published, config),
        operatorPublicationMs: performance.now() - started,
    };
}

export async function prepare(
    path: string,
    config: GlobalConfig,
): Promise<Record<string, unknown>> {
    await event("terrain.open", { path });
    await event("composition.fixture", { fixture: "arbitrary-q8" });
    const publication = await publishPair(config);
    await event("skeletal.configure", {
        animated_percent: 100,
        bone_count: 64,
        phase_count: 64,
        time_tick: 0,
        unique_poses: false,
        forced_lod: null,
    });
    await event("terrain.lod.configure", {
        near_patch_radius: 2,
        middle_patch_radius: 6,
        forced_lod: null,
    });
    await event("terrain.lod.enable");
    await event("composition.enable");
    await event("composition.order", { order: "terrain-first" });
    await event("camera.reset");
    await event("workbench.pause");
    return publication;
}

export function validatePair(pair: Record<string, unknown>, config: GlobalConfig): void {
    if (!sameGlobal(object(pair, "globalConfig"), config)) {
        fail("composition global config mismatch");
    }
    const expectedLocal = localConfig(config);
    same(object(pair, "config"), expectedLocal, "composition local config");
    const regions = pair.globalRegions;
    if (!Array.isArray(regions) || regions.length !== 25) {
        fail("composition global mapping length mismatch");
    }
    field<string>(pair, "globalMappingSha256", "string");
    if (
        !Array.isArray(pair.logicalRegionIds) || pair.logicalRegionIds.length !== 25 ||
        !Array.isArray(pair.instanceSlots) || pair.instanceSlots.length !== 25 ||
        !Array.isArray(pair.terrainSlots) || pair.terrainSlots.length !== 25
    ) fail("composition local mapping shape mismatch");
}

export async function halfReports(
    pair: Record<string, unknown>,
    config: GlobalConfig,
): Promise<Record<string, unknown>> {
    const terrainStatus = await event("terrain.status");
    const terrain = object(
        object(object(terrainStatus, "renderer"), "published"),
        "transaction",
    );
    const instanceStatus = await event("async.status");
    const instance = object(instanceStatus, "lastCompleted");
    if (
        terrain.transactionId !== pair.terrainTransactionId ||
        instance.transactionId !== pair.instanceTransactionId
    ) fail("composition half transaction IDs do not match the pair");
    if (
        !sameGlobal(object(terrain, "globalConfig"), config) ||
        !sameGlobal(object(instance, "globalConfig"), config)
    ) fail("composition half global configs diverged");
    return { terrain, instance };
}

export async function probe(
    config: GlobalConfig,
    requireVisible = true,
): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateLodCompositionProbe(value, requireVisible);
    const pair = object(object(value, "pair"), "published");
    validatePair(pair, config);
    const terrainGlobal = object(object(value, "terrain"), "globalAddressing");
    if (
        terrainGlobal.mappingSha256 !== pair.globalMappingSha256 ||
        terrainGlobal.entryCount !== 25 || terrainGlobal.duplicateGlobalCount !== 0 ||
        terrainGlobal.mismatchCount !== 0
    ) fail("terrain and composition global mapping evidence diverged");
    return value;
}

export function localProbe(value: Record<string, unknown>): Record<string, unknown> {
    const stable = stableLodCompositionProbe(value);
    const terrain = object(stable, "terrain");
    const { activeMapping: _mapping, activeMappingSha256: _hash, ...logicalTerrain } = terrain;
    return { ...stable, terrain: logicalTerrain };
}

export function globalProbe(value: Record<string, unknown>): Record<string, unknown> {
    const pair = object(object(value, "pair"), "published");
    return {
        config: pair.globalConfig,
        mappingSha256: pair.globalMappingSha256,
        regions: pair.globalRegions,
    };
}

export async function capture(id: string, collection: string): Promise<Record<string, unknown>> {
    const raw = await event("perception.capture", {
        id,
        collection,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const evidence = captureEvidence(raw);
    const visible = object(object(object(raw, "perception"), "evidence"), "fullFrame").objects;
    if (!Array.isArray(visible)) fail("composition capture omitted semantic objects");
    const kinds = new Set(visible.map((entry) => object({ entry }, "entry").kind));
    if (!kinds.has("terrain-region") || !kinds.has("region-proxy")) {
        fail("composition capture omitted one semantic class");
    }
    return {
        ...evidence,
        png: field<string>(object(raw, "image"), "pngSha256", "string"),
    };
}

export function expectCounts(
    publication: Record<string, unknown>,
    expected: Record<string, number>,
): void {
    const halves = object(publication, "halves");
    for (const name of ["terrain", "instance"]) {
        const report = object(halves, name);
        for (const [fieldName, value] of Object.entries(expected)) {
            if (report[fieldName] !== value) fail(`${name} ${fieldName} mismatch`);
        }
        if (field<number>(report, "residentRegionCount", "number") > 50) {
            fail(`${name} residency exceeded capacity`);
        }
    }
}

export async function hold(
    kind: "terrain-io" | "terrain-copy" | "object-copy",
    config: GlobalConfig,
    collection: string,
): Promise<Record<string, unknown>> {
    const beforeProbe = await probeFromStatus();
    const beforeCapture = await capture(`${kind}-before`, collection);
    const gate = kind === "object-copy"
        ? "async.gate"
        : kind === "terrain-io"
        ? "terrain.io_gate"
        : "terrain.copy_gate";
    await event(`${gate}.arm`);
    const scheduled = await event("composition.global.schedule", config);
    const token = field<number>(scheduled, "token", "number");
    await event("workbench.resume");
    const deadline = Date.now() + 10_000;
    let heldStatus: Record<string, unknown> | undefined;
    while (Date.now() < deadline) {
        const status = await event("composition.status");
        if (status.pending) {
            const pending = object(status, "pending");
            const reached = kind === "object-copy"
                ? pending.terrainStage === "staged" && pending.instanceStage === "in-flight"
                : pending.instanceStage === "staged" && pending.terrainStage === "in-flight";
            if (reached) {
                heldStatus = status;
                break;
            }
        }
        await sleep(10);
    }
    if (!heldStatus) fail(`${kind} did not reach one-half-ready state`);
    await event("workbench.pause");
    const heldProbe = await probeFromStatus();
    const heldCapture = await capture(`${kind}-held`, collection);
    same(globalProbe(heldProbe), globalProbe(beforeProbe), `${kind} old global pair`);
    same(localProbe(heldProbe), localProbe(beforeProbe), `${kind} old local pair`);
    same(heldCapture, beforeCapture, `${kind} old attachments`);
    await event(`${gate}.release`);
    await event("workbench.resume");
    const status = await waitPair(token);
    await event("workbench.pause");
    const published = object(status, "published");
    validatePair(published, config);
    return {
        beforeCapture,
        heldCapture,
        heldStatus,
        published,
        halves: await halfReports(published, config),
    };
}

async function probeFromStatus(): Promise<Record<string, unknown>> {
    const status = await event("composition.status");
    const pair = object(status, "published");
    const config = object(pair, "globalConfig");
    return await probe({
        origin_x: field<number>(object(config, "globalOrigin"), "x", "number"),
        origin_z: field<number>(object(config, "globalOrigin"), "z", "number"),
        center_x: field<number>(object(config, "globalCenter"), "x", "number"),
        center_z: field<number>(object(config, "globalCenter"), "z", "number"),
        active_radius: field<number>(config, "activeRadius", "number"),
    });
}
