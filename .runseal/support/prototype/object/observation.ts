import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { objectNearestOracle } from "../../object/nearest.ts";
import { traversalObservationOrder } from "./observation_order.ts";

const OBSERVATION_RADIUS_Q9 = 512;

export async function observationInvariant(
    launch: Json,
    source: string,
    windowCenter: [number, number],
    expectedCompleted: boolean,
    expectedFeedbackKind: "activated" | "rejected" | "selected" = "selected",
): Promise<Json> {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "object_observation_driver");
    const status = object(driver, "status");
    if (
        driver.revision !== "live-prototype-object-target-v4" ||
        number(driver, "maxDistanceQ9") !== OBSERVATION_RADIUS_Q9 ||
        status.pending !== false ||
        driver.completed !== expectedCompleted
    ) {
        fail(
            `prototype object observation driver diverged: ${
                JSON.stringify({
                    revision: driver.revision,
                    maxDistanceQ9: driver.maxDistanceQ9,
                    pending: status.pending,
                    expectedCompleted,
                    completed: driver.completed,
                })
            }`,
        );
    }

    if (!expectedCompleted) {
        const feedback = object(driver, "frameFeedback");
        if (
            driver.observation !== null || status.target !== null ||
            feedback.submitted !== null || feedback.projected !== null ||
            number(feedback, "submittedFrameCount") !== 0 ||
            feedback.copiedObjectState !== false
        ) {
            fail("prototype retained an object observation or target without an intent");
        }
        return {
            revision: driver.revision,
            maxDistanceQ9: OBSERVATION_RADIUS_Q9,
            pending: false,
            completed: false,
            observation: null,
            target: null,
        };
    }
    const nativeInput = object(launch, "nativeInput");
    const nativeKeys = nativeInput.keys;
    const nativeIntervals = nativeInput.keyPostIntervalsMilliseconds;
    const advance = object(object(readiness, "simulation_driver"), "advance");
    const simulation = object(advance, "simulation");
    const expectedNativeKeys = [
        { key: "F", virtualKey: 70, down: true },
        { key: "Enter", virtualKey: 13, down: true },
    ];
    const stepCount = number(simulation, "stepCount");
    if (
        nativeInput.atomicBatch !== true ||
        !Number.isSafeInteger(nativeInput.batchThreadId) ||
        number(nativeInput, "batchThreadId") <= 0 ||
        number(nativeInput, "batchSpanMilliseconds") < 0 ||
        number(nativeInput, "batchSpanMilliseconds") > 50 ||
        !Array.isArray(nativeKeys) ||
        JSON.stringify(nativeKeys) !== JSON.stringify(expectedNativeKeys) ||
        !Array.isArray(nativeIntervals) ||
        nativeIntervals.length !== Math.max(0, expectedNativeKeys.length - 1) ||
        stepCount < 1 ||
        stepCount > 8
    ) {
        fail(
            `prototype object action did not use one bounded atomic native input batch: ${
                JSON.stringify({
                    expectedFeedbackKind,
                    expectedNativeKeys,
                    nativeKeys,
                    nativeIntervals,
                    atomicBatch: nativeInput.atomicBatch,
                    batchThreadId: nativeInput.batchThreadId,
                    batchSpanMilliseconds: nativeInput.batchSpanMilliseconds,
                    stepCount: simulation.stepCount,
                })
            }`,
        );
    }
    const observation = object(driver, "observation");
    const origin = object(observation, "origin");
    const actorOutput = object(
        object(object(object(readiness, "simulation_driver"), "advance"), "actor"),
        "output",
    );
    const committedPosition = object(object(object(actorOutput, "motion"), "body"), "position");
    same(origin, committedPosition, "prototype object observation committed origin");
    const originRegion = object(origin, "region");
    const oracle = await objectNearestOracle(
        source,
        {
            region: [number(originRegion, "x"), number(originRegion, "z")],
            localXQ9: number(origin, "localXQ9"),
            localZQ9: number(origin, "localZQ9"),
            maxDistanceQ9: OBSERVATION_RADIUS_Q9,
        },
        windowCenter,
    );
    const query = object(observation, "query");
    same(query, oracle, "prototype object observation source oracle");
    const nearest = object(query, "nearest");
    const observedIdentity = object(object(nearest, "object"), "identity");
    const snapshot = object(observation, "snapshot");
    const publicationToken = number(snapshot, "publicationToken");
    if (
        !Number.isSafeInteger(publicationToken) || publicationToken < 1 ||
        snapshot.sourceNamespace !== observedIdentity.sourceNamespace
    ) fail("prototype object observation snapshot diverged from its qualified result");
    const target = object(status, "target");
    if (target.availability !== "resolved") {
        fail("prototype newly observed target was not resolved");
    }
    same(object(target, "identity"), observedIdentity, "prototype retained target identity");
    const feedback = object(driver, "frameFeedback");
    const submitted = object(feedback, "submitted");
    const projected = object(feedback, "projected");
    same(
        object(submitted, "identity"),
        observedIdentity,
        "prototype frame object-target input",
    );
    same(projected, submitted, "prototype projected object-target feedback");
    if (
        submitted.kind !== expectedFeedbackKind ||
        number(feedback, "submittedFrameCount") < 1 || feedback.copiedObjectState !== false
    ) {
        fail(
            `prototype did not forward exact immutable target feedback: ${
                JSON.stringify({
                    expectedFeedbackKind,
                    submittedKind: submitted.kind,
                    submittedFrameCount: feedback.submittedFrameCount,
                    copiedObjectState: feedback.copiedObjectState,
                    nativeIntervals,
                    nativeKeys,
                    stepCount,
                    yawQ16: object(actorOutput, "presentation").yawQ16,
                    deltaXQ9: nearest.deltaXQ9,
                    deltaZQ9: nearest.deltaZQ9,
                    distanceSquaredQ18: nearest.distanceSquaredQ18,
                    interactionStatus: object(
                        object(readiness, "object_interaction_driver"),
                        "status",
                    ),
                })
            }`,
        );
    }
    const targetSnapshot = object(target, "snapshot");
    const targetToken = number(targetSnapshot, "publicationToken");
    if (targetSnapshot.sourceNamespace !== snapshot.sourceNamespace) {
        fail("prototype target validation changed source without clearing the target");
    }
    const traversal = object(readiness, "traversal");
    const traversalOrder = traversalObservationOrder({
        automaticPublicationCount: number(traversal, "automaticPublicationCount"),
        lastPublishedToken: traversal.lastPublished === null
            ? null
            : number(object(traversal, "lastPublished"), "token"),
        publicationToken,
        targetToken,
    });
    if ("object" in target || "position" in target || "presentation" in target) {
        fail("prototype retained copied canonical object content in target state");
    }
    const distanceSquaredQ18 = number(nearest, "distanceSquaredQ18");
    if (
        number(query, "candidateCount") !== 25_600 ||
        distanceSquaredQ18 > OBSERVATION_RADIUS_Q9 * OBSERVATION_RADIUS_Q9
    ) fail("prototype object observation bound diverged");

    return {
        revision: driver.revision,
        maxDistanceQ9: OBSERVATION_RADIUS_Q9,
        pending: false,
        completed: true,
        origin,
        query,
        target,
        exactCommittedOrigin: true,
        independentSourceOracle: true,
        snapshotGatedTarget: true,
        ...traversalOrder,
        copiedObjectState: false,
        nativeInput: {
            atomicBatch: true,
            batchThreadId: nativeInput.batchThreadId,
            batchSpanMilliseconds: nativeInput.batchSpanMilliseconds,
            keyPostIntervalsMilliseconds: nativeIntervals,
            stepCount,
            maximumBatchGeometryInvariant: true,
        },
        frameFeedback: feedback,
    };
}
