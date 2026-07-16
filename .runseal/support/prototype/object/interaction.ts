import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";

const ACTION_RADIUS_Q9 = 512;
const ACKNOWLEDGEMENT_FRAME_COUNT = 12;

function facingRule(driver: Json): Json {
    const rule = object(driver, "facingRule");
    if (
        rule.domain !== "committed-eight-way-yaw" ||
        rule.nonCoincidentDot !== "positive" || rule.coincidentEligible !== true
    ) fail("prototype object interaction facing rule diverged");
    return rule;
}

export function idleInteractionInvariant(launch: Json): Json {
    const driver = object(object(launch, "readiness"), "object_interaction_driver");
    const status = object(driver, "status");
    if (
        driver.revision !== "live-prototype-object-facing-v1" ||
        driver.input !== "Enter" ||
        number(driver, "maxDistanceQ9") !== ACTION_RADIUS_Q9 ||
        number(driver, "acknowledgementFrameCount") !== ACKNOWLEDGEMENT_FRAME_COUNT ||
        driver.attempt !== null || driver.completion !== null || status.pending !== false ||
        status.acknowledgement !== null || number(status, "committedCount") !== 0 ||
        number(status, "ineligibleCount") !== 0 || number(driver, "activatedFrameCount") !== 0 ||
        status.consumed !== null || driver.nearestExclusion !== null ||
        object(driver, "suppression").submitted !== null ||
        object(driver, "suppression").projected !== null ||
        number(object(driver, "suppression"), "projectedFrameCount") !== 0 ||
        driver.copiedObjectState !== false
    ) fail("prototype idle object interaction driver diverged");
    facingRule(driver);
    return {
        revision: driver.revision,
        input: driver.input,
        maxDistanceQ9: ACTION_RADIUS_Q9,
        facingRule: facingRule(driver),
        acknowledgementFrameCount: ACKNOWLEDGEMENT_FRAME_COUNT,
        pending: false,
        acknowledgement: null,
        committedCount: 0,
        ineligibleCount: 0,
        activatedFrameCount: 0,
        consumed: null,
        suppressionProjectedFrameCount: 0,
        copiedObjectState: false,
    };
}

export function interactionInvariant(launch: Json): Json {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "object_interaction_driver");
    const attempt = object(driver, "attempt");
    const completion = object(driver, "completion");
    const status = object(driver, "status");
    const feedback = object(attempt, "feedback");
    const proximity = object(attempt, "proximity");
    const facing = object(attempt, "facing");
    const observation = object(object(readiness, "object_observation_driver"), "observation");
    const nearest = object(object(observation, "query"), "nearest");
    const identity = object(object(nearest, "object"), "identity");
    const acknowledgement = object(status, "acknowledgement");

    if (
        driver.revision !== "live-prototype-object-facing-v1" ||
        driver.input !== "Enter" ||
        number(driver, "maxDistanceQ9") !== ACTION_RADIUS_Q9 ||
        number(driver, "acknowledgementFrameCount") !== ACKNOWLEDGEMENT_FRAME_COUNT ||
        attempt.outcome !== "eligible" ||
        feedback.kind !== "activated" ||
        completion.applied !== true ||
        status.pending !== false ||
        number(status, "committedCount") !== 1 ||
        number(status, "ineligibleCount") !== 0 ||
        number(acknowledgement, "remainingFrames") !== ACKNOWLEDGEMENT_FRAME_COUNT - 1 ||
        number(driver, "activatedFrameCount") !== 1 ||
        object(driver, "suppression").submitted !== null ||
        object(driver, "suppression").projected !== null ||
        number(object(driver, "suppression"), "projectedFrameCount") !== 0 ||
        driver.copiedObjectState !== false
    ) fail("prototype object interaction driver diverged");
    facingRule(driver);

    same(object(feedback, "identity"), identity, "prototype object action retained identity");
    same(object(completion, "feedback"), feedback, "prototype object action frame completion");
    same(
        object(acknowledgement, "identity"),
        identity,
        "prototype object action acknowledgement identity",
    );
    same(object(status, "consumed"), identity, "prototype consumed object identity");
    same(driver.nearestExclusion, identity, "prototype nearest exclusion identity");
    same(
        proximity,
        {
            deltaXQ9: number(nearest, "deltaXQ9"),
            deltaZQ9: number(nearest, "deltaZQ9"),
            distanceSquaredQ18: number(nearest, "distanceSquaredQ18"),
            terrainPosition: object(nearest, "terrainPosition"),
        },
        "prototype object action exact proximity",
    );
    const actorPresentation = object(
        object(
            object(object(object(readiness, "simulation_driver"), "advance"), "actor"),
            "output",
        ),
        "presentation",
    );
    if (
        number(facing, "yawQ16") !== number(actorPresentation, "yawQ16") ||
        number(facing, "directionX") !== 1 || number(facing, "directionZ") !== 0 ||
        number(facing, "dotQ9") !== number(proximity, "deltaXQ9") ||
        number(facing, "dotQ9") <= 0
    ) fail("prototype object action exact committed facing diverged");

    return {
        revision: driver.revision,
        input: driver.input,
        maxDistanceQ9: ACTION_RADIUS_Q9,
        facingRule: facingRule(driver),
        acknowledgementFrameCount: ACKNOWLEDGEMENT_FRAME_COUNT,
        attempt,
        completion,
        status,
        activatedFrameCount: 1,
        exactConsumedIdentity: true,
        nearestExclusionCommitted: true,
        suppressionDeferredUntilAcknowledged: true,
        exactRetainedIdentity: true,
        exactCommittedOriginProximity: true,
        exactCommittedFacing: true,
        projectedFrameCommit: true,
        copiedObjectState: false,
    };
}

export function sideFacingInteractionInvariant(launch: Json): Json {
    const driver = object(object(launch, "readiness"), "object_interaction_driver");
    const attempt = object(driver, "attempt");
    const status = object(driver, "status");
    const suppression = object(driver, "suppression");
    if (
        driver.revision !== "live-prototype-object-facing-v1" ||
        driver.input !== "Enter" ||
        number(driver, "maxDistanceQ9") !== ACTION_RADIUS_Q9 ||
        number(driver, "acknowledgementFrameCount") !== ACKNOWLEDGEMENT_FRAME_COUNT ||
        attempt.outcome !== "ineligible" || attempt.reason !== "outside-facing" ||
        driver.completion !== null || status.pending !== false ||
        status.acknowledgement !== null || number(status, "committedCount") !== 0 ||
        number(status, "ineligibleCount") !== 1 || status.consumed !== null ||
        number(driver, "activatedFrameCount") !== 0 || driver.nearestExclusion !== null ||
        suppression.submitted !== null || suppression.projected !== null ||
        number(suppression, "projectedFrameCount") !== 0 || driver.copiedObjectState !== false
    ) fail("prototype side-facing object interaction driver diverged");
    return {
        revision: driver.revision,
        input: driver.input,
        maxDistanceQ9: ACTION_RADIUS_Q9,
        facingRule: facingRule(driver),
        attempt,
        pending: false,
        committedCount: 0,
        ineligibleCount: 1,
        consumed: null,
        activatedFrameCount: 0,
        nearestExclusion: null,
        exactSideRejection: true,
        copiedObjectState: false,
    };
}
