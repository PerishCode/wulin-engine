import { fail, type Json, number, object, same, string } from "./canonical-runtime.ts";

const EXPECTED_CAPTURE = {
    color: "8b13d2146cd838cab9fee14049e4b2331b93127ee78ec07d5b50e12c99aa4135",
    png: "e96e44cc6c7cf05338433a05568e2a41e81f95f2f5ba8c52ce7baa26114450c6",
    objectId: "01951615d1b4645bdfba68991c75b8ea333482d312f31f39ed3b907ca479da5b",
    diagnostic: "5f6f2f195d9deadfc4db905692d22e805b4e7000f102537ad36a2e01bd319855",
};

const EXPECTED_SKELETAL_GPU = {
    activePoses: 968,
    animated: 7_905,
    emittedTriangles: 2_784_720,
    emittedVertices: 2_690_116,
    evaluatedBones: 61_952,
    lodCounts: [7_109, 3_429, 0],
    meshlets: 58_098,
    observedArchetypeMask: 255,
    rejected: 15_062,
    reusedPoses: 6_937,
    skinInfluences: 8_972_824,
    staticCount: 2_633,
    visible: 10_538,
};

const EXPECTED_SURFACE_STATS = {
    backgroundPixels: 288_171,
    observedMaterialCount: 64,
    observedMaterialMask: [4_294_967_295, 4_294_967_295],
    resolvedPixels: 921_600,
    visiblePixels: 633_429,
};

const EXPECTED_SHADOW = {
    lightViewProjectionSha256: "480ef3365b258ea2a93b21942a800bfdc21d8d1f6241c45ef36fd2d5fa41fd65",
    depthSha256: "34481150db654d8955b7efdae0eaf55eaca039ec50bfc679092068b0c4ae4ebd",
    occupiedTexels: 88_557,
    clearTexels: 960_019,
    minimumOccupiedDepth: 0.4330306351184845,
    maximumOccupiedDepth: 0.6074084639549255,
    casterCount: 10_538,
    sampleShadowedCount: 1,
    sampleLitCount: 5,
    sampleMismatchCount: 0,
};

const EXPECTED_OCCLUSION = {
    tested: 10_538,
    occluded: 2_084,
    survivors: 8_454,
    sourceVisible: 10_538,
    sourceMeshlets: 58_098,
    sourceTriangles: 2_784_720,
    sourceVertices: 2_690_116,
    sourceSkinInfluences: 8_972_824,
    submittedMeshlets: 47_182,
    submittedTriangles: 2_245_440,
    submittedVertices: 2_201_892,
    submittedSkinInfluences: 7_388_640,
    visibleRecordBytes: 56,
    filteredVisibleBytes: 1_433_656,
    orderReadbackBytes: 2_867_312,
    candidateMaskSha256: "f7cb07a3bfd9cf729d51b0074a2c530373241e90c0b426b2f377d46f681b23e7",
    hierarchySha256: "48752eeadc852541b33eae64c7887582713dda488988f54210a71c8652bee0a9",
};

export function assertCanonicalFrame(value: Json, label: string): Json {
    const stable = object(value, "stable");
    const capture = object(stable, "capture");
    same(
        {
            color: string(capture, "color"),
            png: string(capture, "png"),
            objectId: string(capture, "objectId"),
            diagnostic: string(capture, "diagnostic"),
        },
        EXPECTED_CAPTURE,
        `${label} capture`,
    );

    const skeletal = object(stable, "skeletal");
    same(object(skeletal, "gpu"), EXPECTED_SKELETAL_GPU, `${label} skeletal GPU evidence`);
    const surface = object(stable, "surface");
    same(object(surface, "stats"), EXPECTED_SURFACE_STATS, `${label} surface statistics`);

    const shadow = object(surface, "shadow");
    same(
        selectNumbersAndHashes(shadow, Object.keys(EXPECTED_SHADOW)),
        EXPECTED_SHADOW,
        `${label} shadow evidence`,
    );

    const probeSurface = object(object(value, "probe"), "surface");
    const occlusion = object(probeSurface, "occlusion");
    same(
        selectNumbersAndHashes(occlusion, Object.keys(EXPECTED_OCCLUSION)),
        EXPECTED_OCCLUSION,
        `${label} occlusion evidence`,
    );
    return stable;
}

export function assertCanonicalFrameReplay(first: Json, replay: Json): Json {
    const firstStable = assertCanonicalFrame(first, "canonical frame");
    const replayStable = assertCanonicalFrame(replay, "canonical frame replay");
    same(replayStable, firstStable, "canonical frame immediate replay");
    return firstStable;
}

function selectNumbersAndHashes(value: Json, keys: string[]): Json {
    const selected: Json = {};
    for (const key of keys) {
        const field = value[key];
        if (typeof field === "number") {
            selected[key] = number(value, key);
        } else if (typeof field === "string") {
            selected[key] = string(value, key);
        } else {
            fail(`${key} must be a number or string`);
        }
    }
    return selected;
}
