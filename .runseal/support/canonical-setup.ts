import {
    cookObjects,
    cookTerrain,
    type Coord,
    corruptObjects,
    corruptTerrain,
    fail,
    type Json,
    object,
    root,
    run,
    stopCanonicalProcesses,
    string,
} from "./canonical-runtime.ts";

export type CanonicalFramePaths = {
    terrain: string;
    objects: string;
    report: string;
};

export type CanonicalFrameSetup = {
    paths: CanonicalFramePaths;
    storage: Json;
};

export type PrototypePaths = {
    terrain: string;
    objects: string;
    objectsCorrupt: string;
    report: string;
};

export type PrototypeSetup = {
    paths: PrototypePaths;
    storage: Json;
};

export type CanonicalPaths = {
    terrain: string;
    objectsA: string;
    objectsB: string;
    objectsArchetype: string;
    objectsMaterial: string;
    objectsYaw: string;
    objectsAnimation: string;
    objectsImported: string;
    objectsImportedDuration: string;
    objectsCorrupt: string;
    terrainCorrupt: string;
    report: string;
};

export type CanonicalSetup = {
    paths: CanonicalPaths;
    storage: Json;
};

async function resetCaptureCollection(collection: string): Promise<void> {
    const path = `${root}/out/captures/${collection}`;
    try {
        await Deno.remove(path, { recursive: true });
    } catch (error) {
        if (!(error instanceof Deno.errors.NotFound)) throw error;
    }
    await Deno.mkdir(path, { recursive: true });
}

export async function prepareCanonicalFrameSetup(
    collection: string,
    centers: Coord[],
): Promise<CanonicalFrameSetup> {
    const directory = `out/cooked/${collection}`;
    const paths: CanonicalFramePaths = {
        terrain: `${directory}/terrain.wlt`,
        objects: `${directory}/objects.wlr`,
        report: `out/captures/${collection}/acceptance.json`,
    };
    await Deno.mkdir(`${root}/${directory}`, { recursive: true });
    await resetCaptureCollection(collection);
    const terrain = await cookTerrain(paths.terrain, centers);
    const objects = await cookObjects(paths.objects, centers, "a");
    return { paths, storage: { terrain, objects } };
}

export async function preparePrototypeSetup(
    collection: string,
    base: Coord,
): Promise<PrototypeSetup> {
    const directory = `out/cooked/${collection}`;
    const paths: PrototypePaths = {
        terrain: `${directory}/terrain.wlt`,
        objects: `${directory}/objects.wlr`,
        objectsCorrupt: `${directory}/objects-corrupt.wlr`,
        report: `out/captures/${collection}/acceptance.json`,
    };
    await Deno.mkdir(`${root}/${directory}`, { recursive: true });
    await resetCaptureCollection(collection);
    const traversalCenter: Coord = [base[0] + 1, base[1] + 1];
    const cameraOrbitCenter: Coord = [base[0] + 1, base[1] - 1];
    const objectActionCenter: Coord = [base[0] + 4, base[1]];
    const objectActionTraversalCenter: Coord = [base[0] + 5, base[1] + 1];
    const corruptCenter: Coord = [base[0] + 70, base[1]];
    const centers = [
        base,
        traversalCenter,
        cameraOrbitCenter,
        objectActionCenter,
        objectActionTraversalCenter,
        corruptCenter,
    ];
    const terrain = await cookTerrain(paths.terrain, centers);
    const objects = await cookObjects(paths.objects, centers, "a");
    await Deno.copyFile(`${root}/${paths.objects}`, `${root}/${paths.objectsCorrupt}`);
    const objectCorruption = await corruptObjects(paths.objectsCorrupt, corruptCenter);
    return { paths, storage: { terrain, objects, objectCorruption } };
}

export async function prepareCanonicalSetup(
    collection: string,
    base: Coord,
): Promise<CanonicalSetup> {
    const directory = `out/cooked/${collection}`;
    const paths: CanonicalPaths = {
        terrain: `${directory}/terrain.wlt`,
        objectsA: `${directory}/objects-a.wlr`,
        objectsB: `${directory}/objects-b.wlr`,
        objectsArchetype: `${directory}/objects-archetype.wlr`,
        objectsMaterial: `${directory}/objects-material.wlr`,
        objectsYaw: `${directory}/objects-yaw.wlr`,
        objectsAnimation: `${directory}/objects-animation.wlr`,
        objectsImported: `${directory}/objects-imported.wlr`,
        objectsImportedDuration: `${directory}/objects-imported-duration.wlr`,
        objectsCorrupt: `${directory}/objects-corrupt.wlr`,
        terrainCorrupt: `${directory}/terrain-corrupt.wlt`,
        report: `out/captures/${collection}/acceptance.json`,
    };
    await Deno.mkdir(`${root}/${directory}`, { recursive: true });
    await resetCaptureCollection(collection);
    await stopCanonicalProcesses();

    await run(
        "cargo",
        [
            "test",
            "--locked",
            "-p",
            "terrain-format",
            "-p",
            "region-format",
            "-p",
            "terrain-cooker",
            "-p",
            "region-cooker",
            "-p",
            "meshlet-catalog",
            "-p",
            "surface-catalog",
            "-p",
            "animation-catalog",
            "-p",
            "reference-host",
            "-p",
            "prototype",
            "-p",
            "engine-runtime",
        ],
        "canonical codec and cooker tests",
    );
    await run(
        "cargo",
        ["build", "--locked", "-p", "prototype"],
        "thin prototype host build",
    );

    const centers: Coord[] = [];
    for (let offset = -1; offset <= 80; offset += 1) {
        centers.push([base[0] + offset, base[1]]);
    }
    centers.push([base[0] + 1, base[1] + 1]);
    centers.push([base[0] + 1, base[1] - 1]);
    centers.push([base[0] + 41, base[1] + 1]);
    const terrain = await cookTerrain(paths.terrain, centers);
    const objectsA = await cookObjects(paths.objectsA, centers, "a");
    const objectsB = await cookObjects(paths.objectsB, centers, "b");
    const objectsArchetype = await cookObjects(paths.objectsArchetype, centers, "a", "archetype");
    const objectsMaterial = await cookObjects(paths.objectsMaterial, centers, "a", "material");
    const objectsYaw = await cookObjects(paths.objectsYaw, centers, "a", "yaw");
    const objectsAnimation = await cookObjects(paths.objectsAnimation, centers, "a", "animation");
    const objectsImported = await cookObjects(paths.objectsImported, centers, "a", "imported");
    const objectsImportedDuration = await cookObjects(
        paths.objectsImportedDuration,
        centers,
        "a",
        "imported-duration",
    );
    const metadataA = object(objectsA, "metadata");
    const metadataB = object(objectsB, "metadata");
    if (
        metadataA.payloadSchema !== 3 || metadataB.payloadSchema !== 3 ||
        metadataA.stableSeedNamespaceSha256 !== metadataB.stableSeedNamespaceSha256 ||
        metadataA.sourceNamespaceSha256 === metadataB.sourceNamespaceSha256 ||
        string(objectsA, "fileSha256") === string(objectsB, "fileSha256")
    ) fail("canonical object order/source identity gate failed");

    await Deno.copyFile(`${root}/${paths.objectsA}`, `${root}/${paths.objectsCorrupt}`);
    await Deno.copyFile(`${root}/${paths.terrain}`, `${root}/${paths.terrainCorrupt}`);
    const objectCorruption = await corruptObjects(paths.objectsCorrupt, [base[0] + 70, base[1]]);
    const terrainCorruption = await corruptTerrain(paths.terrainCorrupt, [base[0] + 75, base[1]]);
    return {
        paths,
        storage: {
            terrain,
            objectsA,
            objectsB,
            objectsArchetype,
            objectsMaterial,
            objectsYaw,
            objectsAnimation,
            objectsImported,
            objectsImportedDuration,
            objectCorruption,
            terrainCorruption,
        },
    };
}
