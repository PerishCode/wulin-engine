import { fail, type Json, number, object } from "../../canonical-runtime.ts";

const OBSERVATION_RADIUS_Q9 = 512;

export function idleObservationInvariant(launch: Json): Json {
    const driver = object(object(launch, "readiness"), "object_observation_driver");
    const status = object(driver, "status");
    const feedback = object(driver, "frameFeedback");
    if (
        driver.revision !== "live-prototype-object-target-v4" ||
        number(driver, "maxDistanceQ9") !== OBSERVATION_RADIUS_Q9 ||
        status.pending !== false ||
        driver.completed !== false ||
        driver.observation !== null ||
        status.target !== null ||
        feedback.submitted !== null ||
        feedback.projected !== null ||
        number(feedback, "submittedFrameCount") !== 0 ||
        feedback.copiedObjectState !== false
    ) fail("prototype idle object observation driver diverged");
    return {
        revision: driver.revision,
        maxDistanceQ9: OBSERVATION_RADIUS_Q9,
        pending: false,
        completed: false,
        observation: null,
        target: null,
        copiedObjectState: false,
    };
}
