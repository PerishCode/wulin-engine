import { fail, type Json, number, object } from "../../canonical-runtime.ts";

const ACTION_RADIUS_Q9 = 512;
const ACKNOWLEDGEMENT_FRAME_COUNT = 12;

function facingRule(driver: Json): Json {
    const rule = object(driver, "facingRule");
    if (
        rule.domain !== "committed-eight-way-yaw" ||
        rule.nonCoincidentDot !== "positive" ||
        rule.coincidentEligible !== true
    ) fail("prototype object interaction facing rule diverged");
    return rule;
}

export function idleInteractionInvariant(launch: Json): Json {
    const driver = object(object(launch, "readiness"), "object_interaction_driver");
    const status = object(driver, "status");
    const suppression = object(driver, "suppression");
    if (
        driver.revision !== "live-prototype-object-rejected-feedback-v3" ||
        driver.input !== "Enter" ||
        number(driver, "maxDistanceQ9") !== ACTION_RADIUS_Q9 ||
        number(driver, "acknowledgementFrameCount") !== ACKNOWLEDGEMENT_FRAME_COUNT ||
        "attempt" in driver ||
        "completion" in driver ||
        status.pending !== false ||
        status.acknowledgement !== null ||
        number(status, "committedCount") !== 0 ||
        number(status, "ineligibleCount") !== 0 ||
        number(driver, "activatedFrameCount") !== 0 ||
        number(driver, "rejectedFrameCount") !== 0 ||
        status.consumed !== null ||
        driver.nearestExclusion !== null ||
        suppression.submitted !== null ||
        suppression.projected !== null ||
        number(suppression, "projectedFrameCount") !== 0 ||
        driver.copiedObjectState !== false
    ) fail("prototype idle object interaction driver diverged");
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
        rejectedFrameCount: 0,
        consumed: null,
        suppressionProjectedFrameCount: 0,
        copiedObjectState: false,
    };
}
