import { fail, type Json, number, object } from "../../canonical-runtime.ts";

export function capacityRejectionInputInvariant(
    evidence: Json,
    processId: number,
): Json {
    const motion = object(evidence, "motion");
    const action = object(evidence, "action");
    if (
        evidence.revision !== "prototype-capacity-rejection-input-v1" ||
        number(evidence, "requestedMotionHoldMilliseconds") !== 250 ||
        number(evidence, "motionHoldMilliseconds") < 250 ||
        number(motion, "processId") !== processId ||
        JSON.stringify(motion.keys) !== JSON.stringify([
                { key: "D", virtualKey: 68, down: true },
            ]) ||
        number(action, "processId") !== processId ||
        JSON.stringify(action.keys) !== JSON.stringify([
                { key: "D", virtualKey: 68, down: false },
                { key: "F", virtualKey: 70, down: false },
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: false },
                { key: "Enter", virtualKey: 13, down: true },
            ])
    ) fail("prototype sustained capacity-rejection input evidence diverged");
    return {
        revision: evidence.revision,
        requestedMotionHoldMilliseconds: evidence.requestedMotionHoldMilliseconds,
        motionHoldMilliseconds: evidence.motionHoldMilliseconds,
        exactProcessWindow: true,
        exactMotionKeys: true,
        exactActionKeys: true,
        motionThenStationaryAction: true,
    };
}
