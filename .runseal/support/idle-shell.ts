import { array, event, fail, type Json, number, object, string } from "./canonical-runtime.ts";

export async function idleShellGates(collection: string, idleStatus: Json): Promise<Json> {
    if (object(idleStatus, "workload").mode !== "idle-shell") {
        fail("workbench did not start in the clear-only idle shell");
    }
    const spatial = object(idleStatus, "spatial");
    if (
        spatial.revision !== "canonical-camera-space-v1" ||
        JSON.stringify(Object.keys(spatial).sort()) !==
            JSON.stringify(["camera", "coordinateSystem", "depth", "revision"])
    ) fail("idle spatial status retained calibration scene state");

    const capture = await event("perception.capture", {
        id: "idle-shell",
        collection,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }, { x: 1279, y: 719 }],
    });
    if (capture.lastError !== null || object(capture, "renderer").deviceRemovedReason !== null) {
        fail("idle-shell capture reported a renderer failure");
    }
    if (object(capture, "workload").mode !== "idle-shell") {
        fail("idle-shell capture changed runtime mode");
    }
    const image = object(capture, "image");
    if (number(image, "differentPixelCount") !== 0) {
        fail("idle shell rendered pixels beyond its clear color");
    }
    const perception = object(capture, "perception");
    const evidence = object(perception, "evidence");
    const fullFrame = object(evidence, "fullFrame");
    if (
        number(fullFrame, "backgroundPixelCount") !== number(fullFrame, "pixelCount") ||
        array(fullFrame, "objects").length !== 0 || array(evidence, "unknownIds").length !== 0
    ) fail("idle semantic attachment was not uniformly zero");
    for (const sample of array(evidence, "samples") as Json[]) {
        if (number(sample, "id") !== 0 || sample.name !== null || sample.kind !== null) {
            fail("idle semantic sample was not background");
        }
    }

    return {
        status: idleStatus,
        capture: {
            colorSha256: string(image, "pixelSha256"),
            pngSha256: string(image, "pngSha256"),
            differentPixelCount: number(image, "differentPixelCount"),
            referencePixelRgba: array(image, "referencePixelRgba"),
            semanticSha256: string(perception, "rawSha256"),
            semanticValueCount: number(perception, "rawValueCount"),
            backgroundPixelCount: number(fullFrame, "backgroundPixelCount"),
            visibleSemanticCount: array(fullFrame, "objects").length,
            unknownSemanticCount: array(evidence, "unknownIds").length,
        },
    };
}
