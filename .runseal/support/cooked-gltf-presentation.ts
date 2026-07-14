import {
    type Coord,
    event,
    fail,
    frame,
    type Json,
    number,
    object,
    publish,
    same,
    string,
    target,
} from "./canonical-runtime.ts";
import { presentationInvariant } from "./temporal-presentation.ts";

const JSON_SHA = "6ddcabf511c0257b87dedf6ac51f1bdb6f21e570eee5fa7c4fa6162d055cb002";
const BIN_SHA = "c7d0d8de28a84d5b25623037f88e063e1502495a2ee6c55f182c61161ad12f80";
const TEXTURE_SHA = "61c8b109ee7f8bf262791933380fafb1465f7b51cbe6472c2d21eff0b31f83a1";

function assertImportedMetadata(stable: Json): void {
    const skeletal = object(stable, "skeletal");
    const geometry = object(skeletal, "importedGeometry");
    if (
        string(geometry, "revision") !== "cooked-gltf-geometry-v2-skin" ||
        string(geometry, "sourceJsonSha256") !== JSON_SHA ||
        string(geometry, "sourceBinSha256") !== BIN_SHA ||
        string(geometry, "sourceTextureSha256") !== TEXTURE_SHA ||
        string(geometry, "cookedSha256") !==
            "b0eb4940ee63a34e0b64569774ade165b767a458d5806b0239cf90dcf759c077" ||
        string(geometry, "bindingSha256") !==
            "de8831585bbb3a13504a049d106258c8819fb990e3908408239b03554baff319" ||
        number(geometry, "archetype") !== 7 ||
        number(geometry, "vertexCount") !== 434 ||
        number(geometry, "bindingCount") !== 434 ||
        number(geometry, "sourceJointCount") !== 24 ||
        number(geometry, "maximumJointDepth") !== 7 ||
        JSON.stringify(geometry.lodTriangleCounts) !== JSON.stringify([576, 288, 144]) ||
        string(skeletal, "meshletCatalogSha256") !==
            "af535c808e73edbfe84e8df316b21b9d07849fd35da6e68ef80fda1ae11bab60"
    ) fail("imported geometry/skin source metadata diverged");

    const rig = object(skeletal, "importedRig");
    if (
        string(rig, "revision") !== "cooked-gltf-skeletal-animation-v1" ||
        number(rig, "rig") !== 1 ||
        string(rig, "sourceJsonSha256") !== JSON_SHA ||
        string(rig, "sourceBinSha256") !== BIN_SHA ||
        string(rig, "cookedSha256") !==
            "fea223a83fc8d799c6ef794358f98aa5b524a8a0b7d92a80d9ca4c8fa0429ec1" ||
        string(rig, "fixtureRigSha256") !==
            "bf4eb3fddf98f18eb191f2d5ed3a4a5b4dcb9efe399f6375d843faf62fee80e8" ||
        string(rig, "importedRigSha256") !==
            "1ca9897100f0f1b5909dcc0cb892f827483b87f924dfcd325d516cd5cc645b71" ||
        number(rig, "sourceJointCount") !== 24 ||
        number(rig, "maximumJointDepth") !== 7 ||
        number(rig, "poseKeyCapacity") !== 1024 ||
        JSON.stringify(rig.sourceClipNames) !== JSON.stringify(["Survey", "Walk", "Run"]) ||
        JSON.stringify(rig.sourceClipDurationUnits) !==
            JSON.stringify([16400, 3400, 5560]) ||
        JSON.stringify(rig.sourceClipKeyCounts) !== JSON.stringify([83, 18, 25]) ||
        JSON.stringify(rig.clipAliases) !== JSON.stringify([0, 1, 2, 0, 1, 2, 0, 1]) ||
        JSON.stringify(rig.clipDurationUnits) !==
            JSON.stringify([16400, 3400, 5560, 16400, 3400, 5560, 16400, 3400]) ||
        number(rig, "rootConstantDwords") !== 60 ||
        number(rig, "clockFramePeriod") !== 31_002_560 ||
        number(rig, "timeUnitsPerFrame") !== 80 ||
        number(rig, "timeUnitsPerSecond") !== 4_800 ||
        string(skeletal, "animationCatalogSha256") !==
            "4201d5d51820957df83700e7fbc22631e41b3fa8fca6ec076bf727bc61558f82"
    ) fail("imported skeletal animation metadata diverged");
}

function assertImportedMaterial(stable: Json): void {
    const surface = object(stable, "surface");
    const material = object(surface, "importedMaterial");
    if (
        string(surface, "surfaceCatalogSha256") !==
            "4267365c1d71e96beaff2ece04d6a94c450fd86131ebe65c2447c7b95cb8c15d" ||
        string(material, "revision") !== "cooked-gltf-material-v1" ||
        string(material, "sourceJsonSha256") !== JSON_SHA ||
        string(material, "sourceTextureSha256") !== TEXTURE_SHA ||
        string(material, "cookedSha256") !==
            "5c18b4a6c9f13f79d5b6714ece3d0ef3e4ee20c181b1a169c7eb6a8392e41f0c" ||
        string(material, "fixtureTextureSha256") !==
            "3f6256268867bf270268e7478145e12ab7e3216612b550cd1a412bf440357c8b" ||
        number(material, "materialIndex") !== 63 ||
        number(material, "textureLayer") !== 63 ||
        number(material, "textureSide") !== 64 ||
        number(material, "catalogGpuBytes") !== 1_500_416 ||
        JSON.stringify(material.sourceSize) !== JSON.stringify([1024, 1024]) ||
        JSON.stringify(material.mipSizes) !== JSON.stringify([16384, 4096, 1024, 256, 64, 16, 4])
    ) fail("imported material source/cook metadata diverged");
    const stats = object(surface, "stats");
    if (
        number(stats, "observedMaterialCount") !== 1 ||
        JSON.stringify(stats.observedMaterialMask) !== JSON.stringify([0, 2147483648])
    ) fail("imported presentation did not select only authored material 63");
    if (!Array.isArray(surface.samples)) fail("imported surface samples are not an array");
    const visible = surface.samples.filter((sample) =>
        typeof sample === "object" && sample !== null && (sample as Json).materialIndex !== null
    ) as Json[];
    if (visible.length === 0 || visible.some((sample) => sample.materialIndex !== 63)) {
        fail("imported surface samples did not use authored material 63");
    }
}

function assertImportedGpu(
    frameValue: Json,
    activePoses = 64,
    expectedPhase: number | null = null,
): void {
    const stable = object(frameValue, "stable");
    const skeletal = object(stable, "skeletal");
    const gpu = object(skeletal, "gpu");
    if (
        number(gpu, "observedArchetypeMask") !== 128 ||
        number(gpu, "animated") !== number(gpu, "visible") ||
        number(gpu, "staticCount") !== 0 ||
        number(gpu, "activePoses") !== activePoses
    ) fail("imported authored animation did not become bounded rig-1 GPU work");
    const raw = object(object(object(frameValue, "probe"), "surface"), "skeletal");
    const sample = object(raw, "paletteSample");
    if (
        number(sample, "rig") !== 1 || number(sample, "clip") !== 1 ||
        number(sample, "variant") !== 0 || number(sample, "maximumAbsoluteDelta") > 0.00002
    ) fail("imported GPU palette sample diverged from the CPU rig oracle");
    if (expectedPhase !== null && number(sample, "phase") !== expectedPhase) {
        fail("imported GPU palette phase diverged from source-duration time");
    }
    for (
        const dispatch of [
            "resetDispatchCount",
            "cullDispatchCount",
            "poseCompactDispatchCount",
            "indirectPoseDispatchCount",
            "indirectMeshDispatchCount",
        ]
    ) {
        if (number(raw, dispatch) !== 1) fail(`imported rig changed fixed ${dispatch}`);
    }
}

function durationRepeatEvidence(frameValue: Json): Json {
    const stable = object(frameValue, "stable");
    const skeletal = object(stable, "skeletal");
    return {
        invariant: presentationInvariant(stable),
        objects: object(stable, "objects"),
        skeletal: {
            gpu: skeletal.gpu,
            cpuOracle: skeletal.cpuOracle,
            paletteWriteBytes: skeletal.paletteWriteBytes,
            importedGeometry: skeletal.importedGeometry,
            importedRig: skeletal.importedRig,
            meshletCatalogSha256: skeletal.meshletCatalogSha256,
            animationCatalogSha256: skeletal.animationCatalogSha256,
        },
        surface: stable.surface,
        capture: stable.capture,
    };
}

export async function sourceDurationGates(
    baseline: Json,
    objectPath: string,
    base: Coord,
    collection: string,
): Promise<Json> {
    await event("source.objects.open", { path: objectPath });
    const publication = await publish(target(base));
    const objects = object(object(publication, "published"), "objects");
    if (
        number(objects, "uploadedRegionCount") !== 25 ||
        number(objects, "identityCopyCount") !== 25 ||
        number(objects, "presentationCopyCount") !== 25
    ) fail("source-duration object triple copy count diverged");

    const captures: Record<string, Json> = {};
    for (const [tick, expectedPhase] of [[0, 0], [42, 63], [43, 0], [85, 0]]) {
        await event("canonical.time.set", { tick });
        const value = await frame(`source-duration-${tick}`, collection);
        const stable = object(value, "stable");
        same(
            presentationInvariant(stable),
            presentationInvariant(object(baseline, "stable")),
            `source-duration tick ${tick} spatial/identity invariants`,
        );
        if (number(object(object(stable, "skeletal"), "settings"), "timeTick") !== tick) {
            fail(`source-duration tick ${tick} was not observed by the renderer`);
        }
        assertImportedMetadata(stable);
        assertImportedMaterial(stable);
        assertImportedGpu(value, 1, expectedPhase);
        captures[String(tick)] = value;
    }
    same(
        durationRepeatEvidence(captures["43"]),
        durationRepeatEvidence(captures["0"]),
        "source-duration first sampled Walk loop",
    );
    same(
        durationRepeatEvidence(captures["85"]),
        durationRepeatEvidence(captures["0"]),
        "source-duration second sampled Walk loop",
    );
    const zeroCapture = object(object(captures["0"], "stable"), "capture");
    const endCapture = object(object(captures["42"], "stable"), "capture");
    if (zeroCapture.color === endCapture.color || zeroCapture.png === endCapture.png) {
        fail("source-duration Walk end pose did not change rendered evidence");
    }
    await event("canonical.time.set", { tick: 0 });
    return {
        publication,
        tickZero: captures["0"],
        tick42: captures["42"],
        tick43: captures["43"],
        tick85: captures["85"],
    };
}

export async function importedPresentationGates(
    baseFrame: Json,
    objectPath: string,
    base: Coord,
    collection: string,
): Promise<Json> {
    await event("source.objects.open", { path: objectPath });
    const publication = await publish(target(base));
    const objects = object(object(publication, "published"), "objects");
    if (
        number(objects, "uploadedRegionCount") !== 25 ||
        number(objects, "identityCopyCount") !== 25 ||
        number(objects, "presentationCopyCount") !== 25
    ) fail("imported object triple copy count diverged");
    await event("canonical.time.set", { tick: 0 });
    const tickZero = await frame("presentation-imported-tick-00", collection);
    const baseStable = object(baseFrame, "stable");
    const zeroStable = object(tickZero, "stable");
    same(
        presentationInvariant(zeroStable),
        presentationInvariant(baseStable),
        "imported presentation spatial/identity invariants",
    );
    assertImportedMetadata(zeroStable);
    assertImportedMaterial(zeroStable);
    assertImportedGpu(tickZero);

    await event("canonical.time.set", { tick: 16 });
    let tickSixteen: Json;
    try {
        tickSixteen = await frame("presentation-imported-tick-16", collection);
    } finally {
        await event("canonical.time.set", { tick: 0 });
    }
    const sixteenStable = object(tickSixteen, "stable");
    same(
        presentationInvariant(sixteenStable),
        presentationInvariant(zeroStable),
        "imported articulated tick spatial/identity invariants",
    );
    assertImportedMetadata(sixteenStable);
    assertImportedMaterial(sixteenStable);
    assertImportedGpu(tickSixteen);
    const zeroCapture = object(zeroStable, "capture");
    const sixteenCapture = object(sixteenStable, "capture");
    if (
        sixteenCapture.color === zeroCapture.color ||
        sixteenCapture.png === zeroCapture.png ||
        sixteenCapture.objectId === zeroCapture.objectId
    ) fail("imported source animation did not change color and silhouette attachments");
    const baseCapture = object(baseStable, "capture");
    if (zeroCapture.color === baseCapture.color && zeroCapture.png === baseCapture.png) {
        fail("imported presentation did not change rendered color evidence");
    }
    return { publication, tickZero, tickSixteen };
}
