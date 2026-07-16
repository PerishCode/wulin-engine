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

export async function setObjectSuppression(identity: Json): Promise<Json> {
    const region = object(identity, "region");
    const value = await event("canonical.objects.suppression.set", {
        source_namespace: string(identity, "sourceNamespace"),
        region_x: number(region, "x"),
        region_z: number(region, "z"),
        authored_local_id: number(identity, "authoredLocalId"),
    });
    same(value.objectSuppression, identity, "workbench object suppression input");
    return value;
}

export async function clearObjectSuppression(): Promise<Json> {
    const value = await event("canonical.objects.suppression.clear");
    if (value.objectSuppression !== null) fail("workbench object suppression did not clear");
    return value;
}

export async function invalidObjectSuppressionGate(identity: Json): Promise<Json> {
    const region = object(identity, "region");
    const value = await rejectedEvent("canonical.objects.suppression.set", {
        source_namespace: string(identity, "sourceNamespace"),
        region_x: number(region, "x"),
        region_z: number(region, "z"),
        authored_local_id: 1_024,
    });
    if (
        typeof value.error !== "string" ||
        !value.error.startsWith("invalid_object_suppression: ")
    ) fail("out-of-range workbench object suppression was not rejected");
    return value;
}

export function assertSuppressedFrame(
    value: Json,
    identity: Json,
    activeIndex: number,
    baseline: Json,
    label: string,
): void {
    const surface = object(object(value, "probe"), "surface");
    const skeletal = object(surface, "skeletal");
    same(
        skeletal.objectSuppression,
        {
            activeIndex,
            authoredLocalId: number(identity, "authoredLocalId"),
        },
        `${label} projected suppression`,
    );
    const baselineSurface = object(object(baseline, "probe"), "surface");
    const baselineSkeletal = object(baselineSurface, "skeletal");
    const gpu = object(skeletal, "gpu");
    const baselineGpu = object(baselineSkeletal, "gpu");
    if (
        number(gpu, "visible") !== number(baselineGpu, "visible") - 1 ||
        number(gpu, "rejected") !== number(baselineGpu, "rejected") + 1
    ) fail(`${label} did not cull exactly one streamed object`);
    same(object(skeletal, "cpuOracle"), gpu, `${label} skeletal CPU/GPU suppression oracle`);

    const shadow = object(surface, "shadow");
    const baselineShadow = object(baselineSurface, "shadow");
    if (number(shadow, "casterCount") !== number(baselineShadow, "casterCount") - 1) {
        fail(`${label} shadow path did not consume the suppressed cull result`);
    }
    const occlusion = object(surface, "occlusion");
    const baselineOcclusion = object(baselineSurface, "occlusion");
    const sourceVisible = number(occlusion, "sourceVisible");
    if (sourceVisible !== number(baselineOcclusion, "sourceVisible") - 1) {
        fail(`${label} occlusion source did not consume the suppressed cull result`);
    }
    if (occlusion.historyQueried === true) {
        if (
            number(occlusion, "tested") !== sourceVisible ||
            number(occlusion, "bypassed") !== 0
        ) fail(`${label} compatible occlusion history did not test the suppressed source exactly`);
    } else if (
        occlusion.historyQueried !== false || number(occlusion, "tested") !== 0 ||
        number(occlusion, "bypassed") !== sourceVisible
    ) fail(`${label} suppression transition did not bypass invalid occlusion history exactly`);

    const capture = object(value, "capture");
    const baselineCapture = object(baseline, "capture");
    if (string(capture, "color") === string(baselineCapture, "color")) {
        fail(`${label} did not remove the exact object from resolved color`);
    }
    same(capture.objectId, baselineCapture.objectId, `${label} semantic attachment`);
}

export function assertUnprojectedSuppression(value: Json, label: string): void {
    const skeletal = object(object(object(value, "probe"), "surface"), "skeletal");
    if (skeletal.objectSuppression !== null) {
        fail(`${label} projected an unavailable object suppression`);
    }
}

export async function objectSuppressionLifecycle(
    selected: VisibleObjectTarget,
    baseline: Json,
    objectPath: string,
    replacementObjectPath: string,
    center: Coord,
    collection: string,
): Promise<Json> {
    const suppressionSet = await setObjectSuppression(selected.identity);
    const suppressed = await frame(
        "suppression-before-source-replacement",
        collection,
        false,
        false,
    );
    assertSuppressedFrame(
        suppressed,
        selected.identity,
        selected.activeIndex,
        baseline,
        "suppression before source replacement",
    );

    await event("source.objects.open", { path: replacementObjectPath });
    const replacementPublication = await publish(target(center));
    const sourceReplaced = await frame("suppression-source-replaced", collection, false, false);
    assertUnprojectedSuppression(sourceReplaced, "source-replaced suppression frame");
    same(sourceReplaced.stable, baseline.stable, "source-replaced suppression baseline");

    await event("source.objects.open", { path: objectPath });
    const revisitPublication = await publish(target(center));
    const sourceRevisited = await frame("suppression-source-revisited", collection, false, false);
    assertSuppressedFrame(
        sourceRevisited,
        selected.identity,
        selected.activeIndex,
        baseline,
        "source-revisited suppression frame",
    );
    same(sourceRevisited.stable, suppressed.stable, "source-revisited suppression replay");

    const departureBase = await publish(target([center[0] + 40, center[1]]));
    const departure = await publish(target([center[0] + 41, center[1] + 1]));
    const departed = await frame("suppression-departed", collection, false, false);
    assertUnprojectedSuppression(departed, "same-source departed suppression frame");

    const returnedPublication = await publish(target(center));
    const returned = await frame("suppression-returned", collection, false, false);
    assertSuppressedFrame(
        returned,
        selected.identity,
        selected.activeIndex,
        baseline,
        "same-source returned suppression frame",
    );
    same(returned.stable, suppressed.stable, "same-source returned suppression replay");

    const suppressionCleared = await clearObjectSuppression();
    const clearWarm = await frame("suppression-lifecycle-clear-warm", collection, false, false);
    const restored = await frame("suppression-lifecycle-restored", collection, false, false);
    same(restored.stable, baseline.stable, "suppression lifecycle baseline restoration");
    return {
        suppressionSet,
        suppressed,
        replacementPublication,
        sourceReplaced,
        revisitPublication,
        sourceRevisited,
        departureBase,
        departure,
        departed,
        returnedPublication,
        returned,
        suppressionCleared,
        clearWarm,
        restored,
    };
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
