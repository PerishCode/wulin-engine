import { type Coord, fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { objectNearestOracle } from "../../object/nearest.ts";
import { actorInvariant } from "../actor.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { jumpPolicyInvariant } from "../jump.ts";
import { traversalInvariant } from "../traversal.ts";
import { observationInvariant } from "./observation.ts";
import {
    idleInteractionInvariant,
    interactionInvariant,
    sideFacingInteractionInvariant,
} from "./interaction.ts";

type StartupInvariant = (launch: Json) => Json;
type SimulationInvariant = (launch: Json) => Json;

export async function restartObservation(
    restarted: Json,
    first: Json,
    objects: string,
    base: Coord,
): Promise<void> {
    same(
        await observationInvariant(restarted, objects, base, false),
        await observationInvariant(first, objects, base, false),
        "prototype restart object observation policy",
    );
    same(
        idleInteractionInvariant(restarted),
        idleInteractionInvariant(first),
        "prototype restart object interaction policy",
    );
}

export async function facingRejectionGates(
    launch: Json,
    baseline: Json,
    objects: string,
    base: Coord,
    startupInvariant: StartupInvariant,
    simulationInvariant: SimulationInvariant,
): Promise<Json> {
    same(
        startupInvariant(launch),
        startupInvariant(baseline),
        "prototype side-facing action configuration",
    );
    same(
        actorInvariant(launch, base),
        actorInvariant(baseline, base),
        "prototype side-facing action initial actor authority",
    );
    return {
        simulation: simulationInvariant(launch),
        observation: await observationInvariant(launch, objects, base, true, "rejected"),
        interaction: sideFacingInteractionInvariant(launch),
        jump: jumpPolicyInvariant(launch, true),
        camera: cameraDriverInvariant(launch),
        traversal: traversalInvariant(launch, base),
    };
}

export async function objectFacingGates(
    admitted: Json,
    rejected: Json,
    baseline: Json,
    objects: string,
    base: Coord,
    startupInvariant: StartupInvariant,
    admittedSimulation: SimulationInvariant,
    rejectedSimulation: SimulationInvariant,
): Promise<Json> {
    return {
        admitted: await observationGates(
            admitted,
            baseline,
            objects,
            base,
            startupInvariant,
            admittedSimulation,
        ),
        rejected: await facingRejectionGates(
            rejected,
            baseline,
            objects,
            base,
            startupInvariant,
            rejectedSimulation,
        ),
    };
}

export async function sustainedCapacityInvariant(
    launch: Json,
    session: Json,
    source: string,
    windowCenter: Coord,
): Promise<Json> {
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyInteraction = object(
        object(readiness, "object_interaction_driver"),
        "status",
    );
    const finalInteraction = object(completion, "object_interaction");
    const finalObservation = object(completion, "object_observation");
    const consumed = object(readyInteraction, "consumed");
    if (
        number(readyInteraction, "committedCount") !== 1 ||
        number(readyInteraction, "ineligibleCount") !== 0 ||
        finalInteraction.pending !== false ||
        finalInteraction.acknowledgement !== null ||
        number(finalInteraction, "committedCount") !== 1 ||
        number(finalInteraction, "ineligibleCount") !== 1 ||
        finalObservation.pending !== false ||
        finalObservation.target === null
    ) fail("prototype sustained capacity-one state diverged");
    same(
        object(finalInteraction, "consumed"),
        consumed,
        "prototype sustained consumed identity",
    );
    same(
        object(finalInteraction, "nearestExclusion"),
        consumed,
        "prototype sustained nearest exclusion",
    );
    const finalTarget = object(finalObservation, "target");
    if (
        finalTarget.availability !== "resolved" ||
        JSON.stringify(object(finalTarget, "identity")) === JSON.stringify(consumed)
    ) fail("prototype sustained capacity rejection did not retain a different resolved target");

    const frames = object(completion, "frames");
    if (
        number(frames, "liveFrameCount") <=
            number(object(readiness, "simulation_driver"), "liveFrameCount") ||
        number(frames, "activatedFrameCount") < 1 ||
        number(frames, "rejectedFrameCount") !== 12 ||
        number(frames, "suppressionProjectedFrameCount") < 1
    ) fail("prototype sustained session did not project exact capacity rejection and suppression");
    const readyPosition = object(
        object(
            object(object(object(readiness, "actor"), "state"), "motion"),
            "body",
        ),
        "position",
    );
    const finalPosition = object(
        object(
            object(object(object(completion, "actor"), "state"), "motion"),
            "body",
        ),
        "position",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "sustained actor region",
    );
    if (number(finalPosition, "localXQ9") <= number(readyPosition, "localXQ9")) {
        fail("prototype sustained actor did not advance after readiness");
    }
    const finalRegion = object(finalPosition, "region");
    const expected = await objectNearestOracle(
        source,
        {
            region: [number(finalRegion, "x"), number(finalRegion, "z")],
            localXQ9: number(finalPosition, "localXQ9"),
            localZQ9: number(finalPosition, "localZQ9"),
            maxDistanceQ9: 512,
            excludedIdentity: consumed,
        },
        windowCenter,
    );
    const expectedIdentity = object(object(object(expected, "nearest"), "object"), "identity");
    same(
        object(finalTarget, "identity"),
        expectedIdentity,
        "prototype sustained exclusion-aware second target",
    );

    const postReadiness = object(launch, "postReadinessInput");
    const transitions = postReadiness.keys;
    if (
        !Array.isArray(transitions) ||
        transitions.length !== 5 ||
        JSON.stringify(transitions) !== JSON.stringify([
                { key: "D", virtualKey: 68, down: false },
                { key: "F", virtualKey: 70, down: false },
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: false },
                { key: "Enter", virtualKey: 13, down: true },
            ])
    ) fail("prototype sustained capacity-rejection input evidence diverged");

    return {
        ...session,
        consumedIdentity: consumed,
        rejectedTargetIdentity: expectedIdentity,
        committedCount: 1,
        postReadinessIneligibleCount: 1,
        acknowledgement: null,
        capacityRejectedFrameCount: 12,
        suppressionProjectedFrameCount: number(frames, "suppressionProjectedFrameCount"),
        actorAdvancedAfterReadiness: true,
        postReadinessCapacityRejection: postReadiness,
        independentExclusionOracle: true,
        exactCapacityOneRollback: true,
    };
}

export async function observationGates(
    launch: Json,
    baseline: Json,
    objects: string,
    base: Coord,
    startupInvariant: StartupInvariant,
    simulationInvariant: SimulationInvariant,
): Promise<Json> {
    same(
        startupInvariant(launch),
        startupInvariant(baseline),
        "prototype object observation configuration",
    );
    same(
        actorInvariant(launch, base),
        actorInvariant(baseline, base),
        "prototype object observation initial actor authority",
    );
    return {
        simulation: simulationInvariant(launch),
        observation: await observationInvariant(launch, objects, base, true, "activated"),
        interaction: interactionInvariant(launch),
        jump: jumpPolicyInvariant(launch, true),
        camera: cameraDriverInvariant(launch),
        traversal: traversalInvariant(launch, base),
    };
}
