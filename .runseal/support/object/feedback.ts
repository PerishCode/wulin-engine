import {
    array,
    assertObjectCopies,
    type Coord,
    event,
    fail,
    frame,
    type Json,
    number,
    object,
    publish,
    rejectedEvent,
    same,
    string,
    target,
} from "../canonical-runtime.ts";

export type VisibleObjectTarget = {
    identity: Json;
    activeIndex: number;
    semanticRegion: number;
};

export type ObjectFeedbackKind = "activated" | "selected";

export function visibleObjectTarget(
    value: Json,
    sourceNamespace: string,
    center: [number, number],
): VisibleObjectTarget {
    const surface = object(object(value, "probe"), "surface");
    for (const raw of array(surface, "samples")) {
        const sample = raw as Json;
        if (typeof sample.candidateIndex !== "number" || sample.candidateIndex >= 25_600) {
            continue;
        }
        const stableIdentity = array(sample, "stableIdentity");
        const authoredLocalId = stableIdentity[1];
        if (
            !Number.isSafeInteger(authoredLocalId) || (authoredLocalId as number) < 0 ||
            (authoredLocalId as number) >= 1_024
        ) {
            fail("visible target sample has an invalid authored local ID");
        }
        const activeIndex = Math.floor(sample.candidateIndex / 1_024);
        const offsetX = activeIndex % 5 - 2;
        const offsetZ = Math.floor(activeIndex / 5) - 2;
        return {
            identity: {
                authoredLocalId,
                region: { x: center[0] + offsetX, z: center[1] + offsetZ },
                sourceNamespace,
            },
            activeIndex,
            semanticRegion: (64 + offsetZ) * 128 + 64 + offsetX,
        };
    }
    fail("canonical surface samples contain no visible streamed object");
}

export async function setObjectTarget(
    identity: Json,
    feedbackKind: ObjectFeedbackKind = "selected",
): Promise<Json> {
    const region = object(identity, "region");
    const payload = {
        source_namespace: string(identity, "sourceNamespace"),
        region_x: number(region, "x"),
        region_z: number(region, "z"),
        authored_local_id: number(identity, "authoredLocalId"),
        feedback_kind: feedbackKind,
    };
    const value = await event("canonical.objects.target.set", payload);
    const actual = object(value, "objectTargetFeedback");
    const actualIdentity = object(actual, "identity");
    same(
        {
            identity: {
                authoredLocalId: number(actualIdentity, "authoredLocalId"),
                region: object(actualIdentity, "region"),
                sourceNamespace: string(actualIdentity, "sourceNamespace"),
            },
            kind: string(actual, "kind"),
        },
        {
            identity: {
                authoredLocalId: number(identity, "authoredLocalId"),
                region: object(identity, "region"),
                sourceNamespace: string(identity, "sourceNamespace"),
            },
            kind: feedbackKind,
        },
        "workbench object target input",
    );
    return value;
}

export async function invalidObjectTargetGate(identity: Json): Promise<Json> {
    const region = object(identity, "region");
    const value = await rejectedEvent("canonical.objects.target.set", {
        source_namespace: string(identity, "sourceNamespace"),
        region_x: number(region, "x"),
        region_z: number(region, "z"),
        authored_local_id: 1_024,
        feedback_kind: "selected",
    });
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("invalid_object_target: ")
    ) fail("out-of-range workbench object target was not rejected");
    return value;
}

export async function invalidObjectFeedbackGate(identity: Json): Promise<Json> {
    const region = object(identity, "region");
    const value = await rejectedEvent("canonical.objects.target.set", {
        source_namespace: string(identity, "sourceNamespace"),
        region_x: number(region, "x"),
        region_z: number(region, "z"),
        authored_local_id: number(identity, "authoredLocalId"),
        feedback_kind: "pulsing",
    });
    if (typeof value.error !== "string" || !value.error.startsWith("invalid_payload: ")) {
        fail("unknown workbench object feedback kind was not rejected");
    }
    return value;
}

export async function clearObjectTarget(): Promise<Json> {
    const value = await event("canonical.objects.target.clear");
    if (value.objectTargetFeedback !== null) fail("workbench object target did not clear");
    return value;
}

export function assertTargetedFrame(
    value: Json,
    identity: Json,
    activeIndex: number,
    semanticRegion: number,
    baseline: Json,
    label: string,
    feedbackKind: ObjectFeedbackKind = "selected",
): number {
    const stats = object(object(object(value, "probe"), "surface"), "stats");
    same(
        object(stats, "objectTarget"),
        {
            activeIndex,
            authoredLocalId: number(identity, "authoredLocalId"),
            kind: feedbackKind,
            semanticRegion,
        },
        `${label} projected object target`,
    );
    const pixels = number(stats, "targetedPixels");
    if (!Number.isSafeInteger(pixels) || pixels <= 0) {
        fail(`${label} did not emphasize any exact target pixels`);
    }
    const capture = object(value, "capture");
    const baselineCapture = object(baseline, "capture");
    if (string(capture, "color") === string(baselineCapture, "color")) {
        fail(`${label} did not change the resolved color`);
    }
    same(capture.objectId, baselineCapture.objectId, `${label} semantic attachment`);
    if (typeof capture.diagnostic === "string" && typeof baselineCapture.diagnostic === "string") {
        same(capture.diagnostic, baselineCapture.diagnostic, `${label} semantic diagnostic`);
    }
    return pixels;
}

export function assertUntargetedFrame(value: Json, label: string): void {
    const stats = object(object(object(value, "probe"), "surface"), "stats");
    if (stats.objectTarget !== null || number(stats, "targetedPixels") !== 0) {
        fail(`${label} retained object-target feedback`);
    }
}

export async function beginTargetLifecycle(
    selected: VisibleObjectTarget,
    baseline: Json,
    objectPath: string,
    center: Coord,
    collection: string,
): Promise<Json> {
    await event("source.objects.open", { path: objectPath });
    await publish(target(center));
    const targetSet = await setObjectTarget(selected.identity);
    const rendered = await frame("target-before-source-replacement", collection, false, false);
    const pixels = assertTargetedFrame(
        rendered,
        selected.identity,
        selected.activeIndex,
        selected.semanticRegion,
        baseline,
        "target before source replacement",
    );
    return { targetSet, rendered, pixels };
}

export async function confirmTargetRevisit(
    selected: VisibleObjectTarget,
    baseline: Json,
    expectedPixels: number,
    collection: string,
): Promise<Json> {
    const rendered = await frame("order-a-revisit-targeted", collection, false, false);
    const pixels = assertTargetedFrame(
        rendered,
        selected.identity,
        selected.activeIndex,
        selected.semanticRegion,
        baseline,
        "source-revisited target frame",
    );
    same(pixels, expectedPixels, "source-revisited target pixel count");
    return { rendered, pixels, cleared: await clearObjectTarget() };
}

export async function targetDepartureReturn(
    selected: VisibleObjectTarget,
    baseline: Json,
    expectedPixels: number,
    center: Coord,
    collection: string,
): Promise<Json> {
    const targetSet = await setObjectTarget(selected.identity);
    const departureBase = await publish(target([center[0] + 40, center[1]]));
    assertObjectCopies(departureBase, 25, "diagonal cold base");
    const departure = await publish(target([center[0] + 41, center[1] + 1]));
    assertObjectCopies(departure, 9, "diagonal publication");
    const departed = await frame("diagonal", collection, false, false);
    assertUntargetedFrame(departed, "same-source departed target frame");
    const returnedPublication = await publish(target(center));
    const returnedTargeted = await frame("returned-targeted", collection, false, false);
    const pixels = assertTargetedFrame(
        returnedTargeted,
        selected.identity,
        selected.activeIndex,
        selected.semanticRegion,
        baseline,
        "same-source returned target frame",
    );
    same(pixels, expectedPixels, "same-source returned target pixel count");
    const cleared = await clearObjectTarget();
    const returned = await frame("returned", collection, false, false);
    same(returned.stable, baseline.stable, "movement revisit");
    return {
        targetSet,
        departureBase,
        departure,
        departed,
        returnedPublication,
        returnedTargeted,
        pixels,
        cleared,
        returned,
    };
}
