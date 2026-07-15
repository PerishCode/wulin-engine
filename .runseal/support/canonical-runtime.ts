export type Json = Record<string, unknown>;
export type Coord = [number, number];
export type GlobalConfig = {
    origin_x: number;
    origin_z: number;
    center_x: number;
    center_z: number;
    active_radius: number;
};

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) throw new Error("RUNSEAL_PROFILE_PATH is not set");
export const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
let sidecarConfig = "sidecar.toml";

export function fail(message: string): never {
    throw new Error(message);
}

export function object(value: Json, name: string): Json {
    const field = value[name];
    if (!field || typeof field !== "object" || Array.isArray(field)) {
        fail(`${name} must be an object`);
    }
    return field as Json;
}

export function array(value: Json, name: string): unknown[] {
    const field = value[name];
    if (!Array.isArray(field)) fail(`${name} must be an array`);
    return field;
}

export function number(value: Json, name: string): number {
    const field = value[name];
    if (typeof field !== "number" || !Number.isFinite(field)) {
        fail(`${name} must be a finite number`);
    }
    return field;
}

export function string(value: Json, name: string): string {
    const field = value[name];
    if (typeof field !== "string") fail(`${name} must be a string`);
    return field;
}

export function assertObjectCopies(publication: Json, expected: number, label: string): void {
    const objects = object(object(publication, "published"), "objects");
    for (const key of ["uploadedRegionCount", "identityCopyCount", "presentationCopyCount"]) {
        if (number(objects, key) !== expected) fail(`${label} object triple copy count diverged`);
    }
}

export function same(actual: unknown, expected: unknown, label: string): void {
    const left = JSON.stringify(actual);
    const right = JSON.stringify(expected);
    if (left !== right) fail(`${label} diverged at ${firstDifference(actual, expected)}`);
}

function firstDifference(actual: unknown, expected: unknown, path = "$."): string {
    if (Object.is(actual, expected)) return `${path}(equal)`;
    if (Array.isArray(actual) && Array.isArray(expected)) {
        if (actual.length !== expected.length) {
            return `${path}length (actual=${actual.length}, expected=${expected.length})`;
        }
        for (let index = 0; index < actual.length; index++) {
            if (JSON.stringify(actual[index]) !== JSON.stringify(expected[index])) {
                return firstDifference(actual[index], expected[index], `${path}[${index}]`);
            }
        }
    }
    if (
        actual !== null && expected !== null && typeof actual === "object" &&
        typeof expected === "object" && !Array.isArray(actual) && !Array.isArray(expected)
    ) {
        const actualObject = actual as Json;
        const expectedObject = expected as Json;
        const keys = [...new Set([...Object.keys(actualObject), ...Object.keys(expectedObject)])]
            .sort();
        for (const key of keys) {
            if (JSON.stringify(actualObject[key]) !== JSON.stringify(expectedObject[key])) {
                return firstDifference(actualObject[key], expectedObject[key], `${path}${key}.`);
            }
        }
    }
    return `${path.slice(0, -1)} (actual=${JSON.stringify(actual)}, expected=${
        JSON.stringify(expected)
    })`;
}

export function sleep(milliseconds: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export function target(center: Coord, localX = 64, localZ = 64): GlobalConfig {
    return {
        origin_x: center[0] - (localX - 64),
        origin_z: center[1] - (localZ - 64),
        center_x: center[0],
        center_z: center[1],
        active_radius: 2,
    };
}

export function useSidecar(config: string): void {
    sidecarConfig = config;
}

export async function stopCanonicalProcesses(): Promise<void> {
    for (
        const config of [
            "sidecar.toml",
            "sidecar.benchmark.toml",
            "sidecar.bootstrap.toml",
            "sidecar.prototype.toml",
        ]
    ) {
        useSidecar(config);
        await lifecycle("stop");
    }
    useSidecar("sidecar.toml");
}

export async function run(command: string, args: string[], label: string): Promise<void> {
    console.log(`==> ${label}`);
    const status = await new Deno.Command(command, {
        args,
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`${label} failed with exit code ${status.code}`);
}

async function invoke(args: string[], allowFailure = false): Promise<Json> {
    for (let attempt = 0; attempt < 3; attempt += 1) {
        const output = await new Deno.Command("sidecar", {
            args: [...args, "--config", sidecarConfig, "--format", "json"],
            cwd: root,
            stdout: "piped",
            stderr: "piped",
        }).output();
        const text = decoder.decode(output.stdout).trim();
        if (!text) {
            if (attempt < 2) {
                await sleep(50);
                continue;
            }
            fail(`sidecar ${args.join(" ")} returned no JSON`);
        }
        let value: Json;
        try {
            value = JSON.parse(text) as Json;
        } catch (error) {
            if (attempt < 2) {
                await sleep(50);
                continue;
            }
            fail(`sidecar returned invalid JSON: ${error}: ${text}`);
        }
        if (!allowFailure && (!output.success || value.ok === false)) {
            fail(`sidecar ${args.join(" ")} failed: ${JSON.stringify(value.error)}`);
        }
        return value;
    }
    fail("sidecar invocation retry exhausted");
}

export async function lifecycle(verb: "start" | "stop" | "restart"): Promise<void> {
    await run("sidecar", [verb, "--config", sidecarConfig], `sidecar ${sidecarConfig} ${verb}`);
}

export async function startClean(config = "sidecar.toml"): Promise<void> {
    useSidecar(config);
    await lifecycle("stop");
    await lifecycle("start");
}

export async function event(verb: string, payload: unknown = {}): Promise<Json> {
    const response = await invoke([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]);
    return object(response, "data");
}

export async function rejectedEvent(verb: string, payload: unknown = {}): Promise<Json> {
    const response = await invoke([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ], true);
    if (response.ok !== false) fail(`${verb} unexpectedly succeeded`);
    return response;
}

export async function status(): Promise<Json> {
    return await event("workbench.status");
}

export async function openSources(terrain: string, objects: string): Promise<void> {
    await event("source.terrain.open", { path: terrain });
    await event("source.objects.open", { path: objects });
    await event("canonical.time.pause");
    await event("canonical.time.set", { tick: 0 });
}

export async function cookTerrain(path: string, centers: Coord[]): Promise<Json> {
    const args = ["run", "--locked", "--release", "-p", "terrain-cooker", "--", path];
    for (const [x, z] of centers) args.push("--global-center", String(x), String(z));
    const output = await new Deno.Command("cargo", {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("canonical terrain cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim()) as Json;
}

export async function cookObjects(
    path: string,
    centers: Coord[],
    order: "a" | "b",
    presentation:
        | "base"
        | "archetype"
        | "material"
        | "yaw"
        | "animation"
        | "imported"
        | "imported-duration" = "base",
): Promise<Json> {
    const args = [
        "run",
        "--locked",
        "--release",
        "-p",
        "region-cooker",
        "--",
        path,
        "--physical-order",
        order,
        "--presentation",
        presentation,
    ];
    for (const [x, z] of centers) args.push("--global-center", String(x), String(z));
    const output = await new Deno.Command("cargo", {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("canonical object cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim()) as Json;
}

export async function corruptTerrain(path: string, region: Coord): Promise<Json> {
    return await corruptPayload(path, region, 64, 16);
}

export async function corruptObjects(path: string, region: Coord): Promise<Json> {
    return await corruptPayload(path, region, 96, 20_480 + 4_096 + 12);
}

async function corruptPayload(
    path: string,
    region: Coord,
    indexOffset: number,
    payloadByteOffset: number,
): Promise<Json> {
    const bytes = await Deno.readFile(`${root}/${path}`);
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const count = view.getUint32(16, true);
    for (let index = 0; index < count; index += 1) {
        const entry = indexOffset + index * 64;
        if (
            Number(view.getBigInt64(entry, true)) !== region[0] ||
            Number(view.getBigInt64(entry + 8, true)) !== region[1]
        ) continue;
        const offset = Number(view.getBigUint64(entry + 16, true)) + payloadByteOffset;
        const original = bytes[offset];
        bytes[offset] ^= 1;
        await Deno.writeFile(`${root}/${path}`, bytes);
        return { offset, original, corrupted: bytes[offset] };
    }
    fail(`pack region ${region[0]},${region[1]} was not found`);
}

export async function waitStatus(
    label: string,
    predicate: (value: Json) => boolean,
    timeoutMs = 20_000,
): Promise<Json> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
        const value = await event("canonical.status");
        if (predicate(value)) return value;
        await sleep(10);
    }
    fail(`${label} timed out`);
}

export function targetMatches(value: unknown, expected: GlobalConfig): boolean {
    if (!value || typeof value !== "object" || Array.isArray(value)) return false;
    const owner = value as Json;
    const global = owner.globalConfig;
    if (!global || typeof global !== "object" || Array.isArray(global)) return false;
    const config = global as Json;
    const origin = config.globalOrigin;
    const center = config.globalCenter;
    if (
        !origin || typeof origin !== "object" || Array.isArray(origin) ||
        !center || typeof center !== "object" || Array.isArray(center)
    ) return false;
    const originValue = origin as Json;
    const centerValue = center as Json;
    return originValue.x === expected.origin_x && originValue.z === expected.origin_z &&
        centerValue.x === expected.center_x && centerValue.z === expected.center_z &&
        config.activeRadius === expected.active_radius;
}

export async function publish(config: GlobalConfig): Promise<Json> {
    const scheduled = await event("canonical.schedule", config);
    const token = number(scheduled, "token");
    await event("workbench.resume");
    const completed = await waitStatus(
        `canonical publication ${token}`,
        (value) =>
            value.pending === null && value.published !== null &&
            number(object(value, "published"), "token") === token,
    );
    await event("workbench.pause");
    const published = object(completed, "published");
    if (!targetMatches(published, config)) fail("canonical publication target diverged");
    return { scheduled, status: completed, published };
}

export async function probe(allowPending = false): Promise<Json> {
    const value = await event("canonical.probe");
    validateProbe(value, allowPending);
    return value;
}

export async function warmProbe(allowPending = false): Promise<Json> {
    await probe(allowPending);
    return await probe(allowPending);
}

export async function capture(id: string, collection: string): Promise<Json> {
    const value = await event("perception.capture", {
        id,
        collection,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    if (value.lastError !== null || object(value, "renderer").deviceRemovedReason !== null) {
        fail("canonical capture reported a renderer failure");
    }
    const evidence = object(object(value, "perception"), "evidence");
    if (array(evidence, "unknownIds").length !== 0) {
        fail("canonical capture contained unknown semantic IDs");
    }
    const visible = array(object(evidence, "fullFrame"), "objects") as Json[];
    const kinds = new Set(visible.map((entry) => entry.kind));
    if (!kinds.has("terrain-region") || !kinds.has("region-proxy")) {
        fail("canonical capture omitted a semantic class");
    }
    return {
        color: string(object(value, "image"), "pixelSha256"),
        png: string(object(value, "image"), "pngSha256"),
        objectId: string(object(value, "perception"), "rawSha256"),
        diagnostic: string(object(value, "perception"), "diagnosticPngSha256"),
        visible,
    };
}

export function validateProbe(value: Json, allowPending = false): void {
    if (value.revision !== "atomic-terrain-object-composition-v1") {
        fail("canonical probe revision diverged");
    }
    const pair = object(value, "pair");
    if (
        pair.enabled !== true || pair.published === null ||
        (!allowPending && pair.pending !== null)
    ) {
        fail("canonical pair is not exclusively published");
    }
    const canonical = object(value, "canonicalObjects");
    const authority = object(canonical, "payloadAuthority");
    if (
        canonical.entryCount !== 25 || canonical.semanticCollisionCount !== 0 ||
        canonical.stableSeedCollisionCount !== 0 || canonical.mismatchCount !== 0 ||
        canonical.localIdCount !== 25_600 || canonical.localIdDuplicateCount !== 0 ||
        authority.payloadSchema !== 3 || authority.regionCount !== 25 ||
        authority.recordCount !== 25_600 || authority.copyCount !== 75 ||
        authority.recordCopyCount !== 25 || authority.identityCopyCount !== 25 ||
        authority.presentationCopyCount !== 25 || authority.readbackBytes !== 1_024_000 ||
        authority.chunkMismatchCount !== 0 ||
        authority.expectedIndexSha256 !== authority.observedIndexSha256
    ) fail("canonical object authority diverged");
    const grounding = object(value, "grounding");
    const boundaries = object(grounding, "boundaries");
    if (
        grounding.authority !== "arbitrary-q8" || grounding.groundingMode !== 2 ||
        grounding.groundDenominator !== 65_536 ||
        grounding.positionLatticeDenominator !== 512 || grounding.candidateCount !== 25_600 ||
        grounding.mismatchCount !== 0 || grounding.firstMismatch !== null ||
        boundaries.positionMismatchCount !== 0 || boundaries.groundMismatchCount !== 0 ||
        boundaries.firstMismatch !== null
    ) fail("canonical grounding diverged");
    const terrainQuery = object(value, "terrainQuery");
    const queryTriangles = object(terrainQuery, "triangles");
    if (
        terrainQuery.revision !== "exact-canonical-terrain-query-v1" ||
        terrainQuery.regionCount !== 25 || terrainQuery.sampleCount !== 76_800 ||
        terrainQuery.positionDenominator !== 512 || terrainQuery.heightDenominator !== 65_536 ||
        queryTriangles.first !== 25_600 || queryTriangles.diagonal !== 25_600 ||
        queryTriangles.second !== 25_600 || terrainQuery.oracleMismatchCount !== 0 ||
        terrainQuery.firstOracleMismatch !== null || terrainQuery.perQueryAllocationBytes !== 0 ||
        terrainQuery.sourceReadCount !== 0 || terrainQuery.gpuCopyCount !== 0 ||
        terrainQuery.gpuReadbackCount !== 0 || terrainQuery.fenceWaitCount !== 0 ||
        terrainQuery.synchronizationCount !== 0
    ) fail("canonical terrain query diverged");
    const terrainContactWitness = object(terrainQuery, "bodyContactWitness");
    const bodyClassifications = object(terrainContactWitness, "classifications");
    if (
        terrainContactWitness.revision !== "exact-terrain-body-contact-witness-v1" ||
        terrainContactWitness.bodyCount !== 225 ||
        terrainContactWitness.halfHeightNumerator !== 65_536 ||
        terrainContactWitness.heightDenominator !== 65_536 ||
        bodyClassifications.separated !== 75 || bodyClassifications.touching !== 75 ||
        bodyClassifications.penetrating !== 75 || terrainContactWitness.correctedCount !== 75 ||
        terrainContactWitness.oracleMismatchCount !== 0 ||
        terrainContactWitness.firstOracleMismatch !== null ||
        terrainContactWitness.perResolutionAllocationBytes !== 0 ||
        terrainContactWitness.sourceReadCount !== 0 || terrainContactWitness.gpuCopyCount !== 0 ||
        terrainContactWitness.gpuReadbackCount !== 0 ||
        terrainContactWitness.fenceWaitCount !== 0 ||
        terrainContactWitness.synchronizationCount !== 0
    ) fail("canonical terrain body-contact witness diverged");
    const simulation = object(value, "simulationSchedule");
    if (
        simulation.revision !== "deterministic-fixed-simulation-schedule-v1" ||
        simulation.tick !== 0 || simulation.remainderNumerator !== 0 ||
        simulation.remainderDenominator !== 1_000_000_000 ||
        simulation.stepsPerSecond !== 60 ||
        simulation.maximumElapsedNanoseconds !== 125_000_000 ||
        simulation.maximumStepsPerAdvance !== 8 ||
        simulation.successfulAdvanceCount !== 0 || simulation.emittedStepCount !== 0
    ) fail("canonical frame mutated the explicit simulation schedule");
    const terrain = object(value, "terrain");
    const global = object(terrain, "globalAddressing");
    const cpuEdges = object(terrain, "cpuEdges");
    const gpuEdges = object(terrain, "gpuEdges");
    if (
        global.entryCount !== 25 || global.duplicateGlobalCount !== 0 ||
        global.mismatchCount !== 0 || cpuEdges.mismatchCount !== 0 ||
        gpuEdges.mismatchCount !== 0
    ) fail("canonical terrain addressing or edges diverged");
    const lod = object(terrain, "lod");
    const oracle = object(lod, "oracle");
    const gpuLod = object(lod, "gpu");
    same(gpuLod.lodCounts, oracle.lodCounts, "terrain LOD counts");
    if (gpuLod.mismatchCount !== 0 || gpuLod.maxLodDelta !== 1) {
        fail("canonical terrain LOD seam evidence diverged");
    }
    const surface = object(value, "surface");
    const skeletal = object(surface, "skeletal");
    same(skeletal.gpu, skeletal.cpuOracle, "skeletal GPU/CPU oracle");
    if (
        surface.invalidPayloadCount !== 0 ||
        number(surface, "maximumSampleChannelDelta") > number(surface, "sampleChannelTolerance")
    ) fail("canonical surface resolve diverged");
    const occlusion = object(surface, "occlusion");
    if (
        occlusion.enabled !== true || occlusion.invalidQueries !== 0 || occlusion.overflow !== 0 ||
        occlusion.stableCompactionMismatchCount !== 0 || occlusion.hierarchyMismatchCount !== 0
    ) fail("canonical occlusion diverged");
    const shadow = object(surface, "shadow");
    if (
        shadow.enabled !== true || number(shadow, "mapSide") !== 1_024 ||
        shadow.format !== "D32_FLOAT" || number(shadow, "mapBytes") !== 4_194_304 ||
        number(shadow, "occupiedTexels") <= 0 || number(shadow, "clearTexels") <= 0 ||
        number(shadow, "casterCount") !== number(object(skeletal, "gpu"), "visible") ||
        number(shadow, "indirectDispatchCount") !== 1 ||
        number(shadow, "rootConstantDwords") !== 60 ||
        number(shadow, "descriptorCount") !== 98 ||
        number(shadow, "sampleMismatchCount") !== 0
    ) fail("canonical directional shadow diverged");
    if (
        value.clearCount !== 1 || value.fixedTerrainDispatches !== 4 ||
        value.fixedSkeletalDispatches !== 6
    ) fail("canonical fixed frame submission diverged");
}

export function stableEvidence(probeValue: Json, captureValue: Json): Json {
    const canonical = object(probeValue, "canonicalObjects");
    const grounding = object(probeValue, "grounding");
    const contact = object(probeValue, "contact");
    const surface = object(probeValue, "surface");
    const shadow = object(surface, "shadow");
    const skeletal = object(surface, "skeletal");
    const terrain = object(probeValue, "terrain");
    const terrainQuery = object(probeValue, "terrainQuery");
    const terrainContactWitness = object(terrainQuery, "bodyContactWitness");
    const simulation = object(probeValue, "simulationSchedule");
    const surfaceSamples = array(surface, "samples").map((value) => {
        const sample = value as Json;
        return {
            pixel: sample.pixel,
            primitiveIndex: sample.primitiveIndex,
            barycentrics: sample.barycentrics,
            stableIdentity: sample.stableIdentity,
            materialIndex: sample.materialIndex,
            mipLevel: sample.mipLevel,
            texel: sample.texel,
            expectedTexel: sample.expectedTexel,
            shadowed: sample.shadowed,
            expectedShadowed: sample.expectedShadowed,
            shadowTexel: sample.shadowTexel,
            expectedShadowTexel: sample.expectedShadowTexel,
            receiverShadowDepth: sample.receiverShadowDepth,
            storedShadowDepth: sample.storedShadowDepth,
            rgba8: sample.rgba8,
            expectedRgba8: sample.expectedRgba8,
            maximumChannelDelta: sample.maximumChannelDelta,
        };
    });
    return {
        objects: {
            identityKeyedSha256: canonical.identityKeyedSha256,
            presentationKeyedSha256: canonical.presentationKeyedSha256,
            stableKeySha256: canonical.stableKeySha256,
            stableSeedSha256: canonical.stableSeedSha256,
            entries: canonical.entries,
        },
        grounding: {
            positionSha256: grounding.identityKeyedPositionSha256,
            groundSha256: grounding.identityKeyedGroundSha256,
            triangles: grounding.triangles,
            boundaries: grounding.boundaries,
        },
        contact: {
            selected: contact.identityKeyedSelectedSurfaceSha256,
            residual: contact.identityKeyedResidualSha256,
            lod: contact.ownerPatchLodCounts,
            negative: contact.negativeCount,
            zero: contact.zeroCount,
            positive: contact.positiveCount,
        },
        terrain: {
            content: object(terrain, "globalContent").contentSha256,
            projection: object(terrain, "canonicalProjection").projectionSha256,
            lod: terrain.lod,
        },
        terrainQuery: {
            resultSha256: terrainQuery.resultSha256,
            identityKeyedSha256: terrainQuery.identityKeyedSha256,
            minimumHeightNumerator: terrainQuery.minimumHeightNumerator,
            maximumHeightNumerator: terrainQuery.maximumHeightNumerator,
            triangles: terrainQuery.triangles,
        },
        terrainContactWitness: {
            resultSha256: terrainContactWitness.resultSha256,
            identityKeyedSha256: terrainContactWitness.identityKeyedSha256,
            classifications: terrainContactWitness.classifications,
            correctedCount: terrainContactWitness.correctedCount,
        },
        simulationSchedule: simulation,
        skeletal: {
            settings: skeletal.settings,
            gpu: skeletal.gpu,
            cpuOracle: skeletal.cpuOracle,
            paletteWriteBytes: skeletal.paletteWriteBytes,
            importedGeometry: skeletal.importedGeometry,
            importedRig: skeletal.importedRig,
            meshletCatalogSha256: skeletal.meshletCatalogSha256,
            animationCatalogSha256: skeletal.animationCatalogSha256,
        },
        surface: {
            settings: surface.settings,
            stats: surface.stats,
            samples: surfaceSamples,
            maximumSampleChannelDelta: surface.maximumSampleChannelDelta,
            surfaceCatalogSha256: surface.surfaceCatalogSha256,
            importedMaterial: surface.importedMaterial,
            shadow: {
                revision: shadow.revision,
                enabled: shadow.enabled,
                direction: shadow.direction,
                lightViewProjectionSha256: shadow.lightViewProjectionSha256,
                mapSide: shadow.mapSide,
                format: shadow.format,
                mapBytes: shadow.mapBytes,
                receiverBias: shadow.receiverBias,
                depthSha256: shadow.depthSha256,
                occupiedTexels: shadow.occupiedTexels,
                clearTexels: shadow.clearTexels,
                minimumOccupiedDepth: shadow.minimumOccupiedDepth,
                maximumOccupiedDepth: shadow.maximumOccupiedDepth,
                casterCount: shadow.casterCount,
                indirectDispatchCount: shadow.indirectDispatchCount,
                rootConstantDwords: shadow.rootConstantDwords,
                descriptorCount: shadow.descriptorCount,
                sampleShadowedCount: shadow.sampleShadowedCount,
                sampleLitCount: shadow.sampleLitCount,
                sampleMismatchCount: shadow.sampleMismatchCount,
            },
        },
        capture: captureValue,
    };
}

export async function frame(id: string, collection: string, allowPending = false): Promise<Json> {
    const probeValue = await warmProbe(allowPending);
    const captureValue = await capture(id, collection);
    return {
        probe: probeValue,
        capture: captureValue,
        stable: stableEvidence(probeValue, captureValue),
    };
}

export async function setPosition(position: Coord): Promise<void> {
    await event("camera.set_pose", {
        position: [position[0], 6, position[1]],
        target: [position[0], 1, position[1] - 3],
        vertical_fov_degrees: 60,
    });
}

export async function setAliasCamera(localX: number, localZ = 64): Promise<void> {
    const x = (localX - 64) * 16;
    const z = (localZ - 64) * 16;
    await event("camera.set_pose", {
        position: [9 + x, 6, 12 + z],
        target: [x, 1, -3 + z],
        vertical_fov_degrees: 60,
    });
}

export async function holdPair(
    gate: string,
    config: GlobalConfig,
    before: Json,
    collection: string,
): Promise<Json> {
    await event(`${gate}.arm`);
    const scheduled = await event("canonical.schedule", config);
    const token = number(scheduled, "token");
    await event("workbench.resume");
    const objectGate = gate.includes("objects");
    const held = await waitStatus(`${gate} hold`, (value) => {
        if (!value.pending) return false;
        const pending = object(value, "pending");
        return objectGate
            ? pending.terrainStage === "staged" && pending.instanceStage === "in-flight"
            : pending.instanceStage === "staged" && pending.terrainStage === "in-flight";
    });
    await event("workbench.pause");
    const heldFrame = await frame(`${gate.replaceAll(".", "-")}-held`, collection, true);
    same(heldFrame.stable, before.stable, `${gate} old frame`);
    await event(`${gate}.release`);
    await event("workbench.resume");
    const completed = await waitStatus(
        `${gate} release`,
        (value) =>
            value.pending === null && value.published !== null &&
            number(object(value, "published"), "token") === token,
    );
    await event("workbench.pause");
    return { held, heldFrame, completed };
}

export async function failedPair(
    config: GlobalConfig,
    before: Json,
    collection: string,
    label: string,
): Promise<Json> {
    const current = await event("canonical.status");
    const publishedToken = number(object(current, "published"), "token");
    const scheduled = await event("canonical.schedule", config);
    const token = number(scheduled, "token");
    await event("workbench.resume");
    const failed = await waitStatus(
        `${label} rollback`,
        (value) =>
            value.pending === null && value.lastFailure !== null &&
            number(object(value, "lastFailure"), "token") === token,
    );
    if (number(object(failed, "published"), "token") !== publishedToken) {
        fail(`${label} replaced the published pair`);
    }
    await event("workbench.pause");
    const heldFrame = await frame(`${label}-rollback`, collection);
    same(heldFrame.stable, before.stable, `${label} rollback frame`);
    return { scheduled, failed, heldFrame };
}

export async function traversalSweep(base: Coord, prepared: boolean): Promise<Json> {
    await setPosition([0, 0]);
    await event("canonical.traversal.enable");
    if (prepared) await event("canonical.prefetch.enable");
    await event("workbench.resume");
    await sleep(30);
    const initial = await event("canonical.status");
    const traversal = object(initial, "traversal");
    const basePublications = number(traversal, "automaticPublicationCount");
    const prefetchBase = prepared ? number(object(traversal, "prefetch"), "completionCount") : 0;
    const samples: Json[] = [];
    for (let offset = 1; offset <= 32; offset += 1) {
        const expected = target([base[0] + offset, base[1]], 64 + offset);
        const meters = (offset - 1) * 16;
        let preparation: Json | null = null;
        if (prepared) {
            await setPosition([meters + 5, 0]);
            preparation = await waitStatus(`prepared crossing ${offset}`, (value) => {
                if (value.pending !== null) return false;
                const state = object(object(value, "traversal"), "prefetch");
                return number(state, "completionCount") === prefetchBase + offset &&
                    targetMatches(state.lastCompleted, expected);
            });
        }
        await setPosition([meters + 9, 0]);
        const published = await waitStatus(`crossing ${offset}`, (value) => {
            if (value.pending !== null || !targetMatches(value.published, expected)) return false;
            return number(object(value, "traversal"), "automaticPublicationCount") ===
                basePublications + offset;
        });
        const evidence = await probe();
        samples.push({ offset, preparation, published, probe: stableProbeSummary(evidence) });
    }
    await event("workbench.pause");
    return { prepared, sampleCount: samples.length, samples };
}

function stableProbeSummary(value: Json): Json {
    const canonical = object(value, "canonicalObjects");
    const grounding = object(value, "grounding");
    const terrainQuery = object(value, "terrainQuery");
    const terrainContactWitness = object(terrainQuery, "bodyContactWitness");
    const simulation = object(value, "simulationSchedule");
    const pair = object(object(value, "pair"), "published");
    return {
        token: pair.token,
        globalConfig: pair.globalConfig,
        identityKeyedSha256: canonical.identityKeyedSha256,
        groundSha256: grounding.identityKeyedGroundSha256,
        terrainQueryResultSha256: terrainQuery.resultSha256,
        terrainQueryIdentitySha256: terrainQuery.identityKeyedSha256,
        terrainQueryMismatchCount: terrainQuery.oracleMismatchCount,
        terrainContactWitnessResultSha256: terrainContactWitness.resultSha256,
        terrainContactWitnessIdentitySha256: terrainContactWitness.identityKeyedSha256,
        terrainContactWitnessMismatchCount: terrainContactWitness.oracleMismatchCount,
        simulationTick: simulation.tick,
        simulationRemainderNumerator: simulation.remainderNumerator,
        simulationAdvanceCount: simulation.successfulAdvanceCount,
        mismatchCount: grounding.mismatchCount,
        combinedGpuMs: object(value, "timing").combinedGpuMs,
    };
}

export type ProcessSample = {
    handleCount: number;
    privateBytes: number;
    threadCount: number;
};

export async function sampleProcess(processId: number): Promise<ProcessSample> {
    const script =
        `$p=Get-Process -Id ${processId} -ErrorAction Stop; @{handleCount=$p.HandleCount;privateBytes=$p.PrivateMemorySize64;threadCount=$p.Threads.Count}|ConvertTo-Json -Compress`;
    const output = await new Deno.Command("pwsh", {
        args: ["-NoProfile", "-Command", script],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail(`failed to sample workbench process ${processId}`);
    const value = JSON.parse(decoder.decode(output.stdout).trim()) as Json;
    return {
        handleCount: number(value, "handleCount"),
        privateBytes: number(value, "privateBytes"),
        threadCount: number(value, "threadCount"),
    };
}

async function settleProcess(
    processId: number,
    label: string,
): Promise<{ sample: ProcessSample; samples: Json[] }> {
    const samples: Json[] = [];
    let previous = await sampleProcess(processId);
    let stableSamples = 0;
    for (let elapsedSeconds = 10; elapsedSeconds <= 180; elapsedSeconds += 10) {
        await sleep(10_000);
        const sample = await sampleProcess(processId);
        samples.push({ elapsedSeconds, ...sample });
        stableSamples = sample.handleCount === previous.handleCount ? stableSamples + 1 : 0;
        previous = sample;
        if (stableSamples >= 6) return { sample, samples };
    }
    fail(`${label} handle count did not settle: ${JSON.stringify(samples)}`);
}

export async function resourcePlateau(base: Coord): Promise<Json> {
    const workbench = await status();
    const processId = number(workbench, "processId");
    const warmPublications = 32;
    for (let index = 1; index <= warmPublications; index += 1) {
        const center: Coord = [base[0] + 40 + (index % 2), base[1]];
        await publish(target(center));
        await probe();
    }
    await warmProbe();
    const quiescentBefore = await settleProcess(processId, "warm-up");
    const samples: Json[] = [];
    for (let index = 1; index <= 64; index += 1) {
        const center: Coord = [base[0] + 40 + (index % 2), base[1]];
        await publish(target(center));
        await probe();
        if (index % 8 === 0) {
            const sample = await sampleProcess(processId);
            samples.push({ publication: index, ...sample });
        }
    }
    const handleCounts = samples.map((value) => number(value, "handleCount"));
    const minimumActiveHandleCount = Math.min(...handleCounts);
    const peakHandleCount = Math.max(...handleCounts);
    const activeBaseline = samples[0];
    const final = samples.at(-1) as Json;
    const activeHandleLimit = number(activeBaseline, "handleCount") + 1;
    const activeHandleOverflow = peakHandleCount > activeHandleLimit;
    if (number(final, "privateBytes") > number(activeBaseline, "privateBytes") + 16 * 1024 * 1024) {
        fail("active private bytes exceeded the 16 MiB plateau allowance");
    }
    const quiescentAfter = await settleProcess(processId, "post-workload");
    if (activeHandleOverflow) {
        fail(
            `active handles exceeded the initial checkpoint allowance: ${
                JSON.stringify({ activeBaseline, activeHandleLimit, peakHandleCount, samples })
            }`,
        );
    }
    if (quiescentAfter.sample.handleCount > quiescentBefore.sample.handleCount) {
        fail(
            `post-workload handles exceeded the quiescent baseline: ${
                JSON.stringify({ quiescentBefore, quiescentAfter })
            }`,
        );
    }
    if (
        quiescentAfter.sample.privateBytes >
            quiescentBefore.sample.privateBytes + 16 * 1024 * 1024
    ) {
        fail("post-workload private bytes exceeded the 16 MiB recovery allowance");
    }
    return {
        processId,
        warmPublications,
        quiescentBefore,
        activeBaseline,
        activeHandleLimit,
        minimumActiveHandleCount,
        peakHandleCount,
        samples,
        quiescentAfter,
    };
}

export async function assertStopped(processId?: number): Promise<Json> {
    const value = await invoke(["status"]);
    const runtime = object(value, "runtime");
    if (runtime.running !== false || array(runtime, "pids").length !== 0) {
        fail(`${sidecarConfig} runtime remained active`);
    }
    for (const raw of array(value, "targets")) {
        const target = raw as Json;
        if (target.running !== false || array(target, "pids").length !== 0) {
            fail(`${sidecarConfig} target remained active`);
        }
    }
    if (processId !== undefined) {
        const script = `(Get-Process -Id ${processId} -ErrorAction SilentlyContinue) -eq $null`;
        const output = await new Deno.Command("pwsh", {
            args: ["-NoProfile", "-Command", script],
            cwd: root,
            stdout: "piped",
            stderr: "inherit",
        }).output();
        if (!output.success || decoder.decode(output.stdout).trim() !== "True") {
            fail(`workbench process ${processId} survived stop`);
        }
    }
    return value;
}

export async function lifecycleCycles(
    terrain: string,
    objects: string,
    config: GlobalConfig,
): Promise<Json> {
    const cycles: Json[] = [];
    for (let cycle = 1; cycle <= 16; cycle += 1) {
        await lifecycle("start");
        const idle = await status();
        if (object(idle, "workload").mode !== "idle-shell") {
            fail(`lifecycle ${cycle} did not start in the idle shell`);
        }
        const processId = number(idle, "processId");
        await openSources(terrain, objects);
        const publication = await publish(config);
        const evidence = await probe();
        await lifecycle("stop");
        const stopped = await assertStopped(processId);
        cycles.push({
            cycle,
            processId,
            token: object(publication, "published").token,
            evidence: stableProbeSummary(evidence),
            stopped,
        });
    }
    return { cycleCount: cycles.length, cycles };
}
