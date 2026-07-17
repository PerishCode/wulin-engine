import { type Coord, fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { objectNearestOracle } from "../../object/nearest.ts";
import { actorInvariant } from "../actor.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { jumpPolicyInvariant } from "../jump.ts";
import { gracefulCompletionInvariant } from "../sessions/mod.ts";
import { traversalInvariant } from "../traversal.ts";
import { idleInteractionInvariant } from "./interaction.ts";
import { idleObservationInvariant } from "./observation.ts";

type StartupInvariant = (launch: Json) => Json;
type SimulationInvariant = (launch: Json) => Json;

export function restartObservation(restarted: Json, first: Json): void {
    same(
        idleObservationInvariant(restarted),
        idleObservationInvariant(first),
        "prototype restart object observation policy",
    );
    same(
        idleInteractionInvariant(restarted),
        idleInteractionInvariant(first),
        "prototype restart object interaction policy",
    );
}

export async function objectFeedbackGates(
    admitted: Json,
    rejected: Json,
    admittedBaseline: Json,
    rejectedBaseline: Json,
    objects: string,
    admittedBase: Coord,
    rejectedBase: Coord,
    startupInvariant: StartupInvariant,
    admittedSimulation: SimulationInvariant,
    rejectedSimulation: SimulationInvariant,
): Promise<Json> {
    return {
        admitted: await feedbackSessionInvariant(
            admitted,
            admittedBaseline,
            objects,
            admittedBase,
            "activated",
            startupInvariant,
            admittedSimulation,
        ),
        rejected: await feedbackSessionInvariant(
            rejected,
            rejectedBaseline,
            objects,
            rejectedBase,
            "rejected",
            startupInvariant,
            rejectedSimulation,
        ),
    };
}

async function feedbackSessionInvariant(
    launch: Json,
    baseline: Json,
    source: string,
    windowCenter: Coord,
    expectedKind: "activated" | "rejected",
    startupInvariant: StartupInvariant,
    simulationInvariant: SimulationInvariant,
): Promise<Json> {
    same(
        startupInvariant(launch),
        startupInvariant(baseline),
        `prototype post-ready ${expectedKind} configuration`,
    );
    same(
        actorInvariant(launch, windowCenter),
        actorInvariant(baseline, windowCenter),
        `prototype post-ready ${expectedKind} initial actor authority`,
    );
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    same(finalActor, readyActor, `prototype post-ready ${expectedKind} stationary actor`);

    const finalPosition = object(object(object(finalActor, "motion"), "body"), "position");
    const region = object(finalPosition, "region");
    const expected = await objectNearestOracle(
        source,
        {
            region: [number(region, "x"), number(region, "z")],
            localXQ9: number(finalPosition, "localXQ9"),
            localZQ9: number(finalPosition, "localZQ9"),
            maxDistanceQ9: 512,
        },
        windowCenter,
    );
    const expectedIdentity = object(object(object(expected, "nearest"), "object"), "identity");
    const interaction = object(completion, "object_interaction");
    const observation = object(completion, "object_observation");
    const frames = object(completion, "frames");
    if (
        interaction.pending !== false ||
        interaction.acknowledgement !== null ||
        observation.pending !== false ||
        number(frames, "renderBlockCount") !== 0 ||
        number(frames, "liveFrameCount") <=
            number(object(readiness, "simulation_driver"), "liveFrameCount")
    ) fail(`prototype post-ready ${expectedKind} final state diverged`);

    if (expectedKind === "activated") {
        if (
            number(interaction, "committedCount") !== 1 ||
            number(interaction, "ineligibleCount") !== 0 ||
            observation.target !== null ||
            number(frames, "activatedFrameCount") !== 12 ||
            number(frames, "rejectedFrameCount") !== 0 ||
            number(frames, "suppressionProjectedFrameCount") < 1
        ) fail("prototype post-ready Activated completion diverged");
        same(
            object(interaction, "consumed"),
            expectedIdentity,
            "prototype post-ready Activated consumed identity",
        );
        same(
            object(interaction, "nearestExclusion"),
            expectedIdentity,
            "prototype post-ready Activated exclusion",
        );
    } else {
        if (
            number(interaction, "committedCount") !== 0 ||
            number(interaction, "ineligibleCount") !== 1 ||
            interaction.consumed !== null ||
            interaction.nearestExclusion !== null ||
            observation.target === null ||
            number(frames, "activatedFrameCount") !== 0 ||
            number(frames, "rejectedFrameCount") !== 12 ||
            number(frames, "suppressionProjectedFrameCount") !== 0
        ) fail("prototype post-ready Rejected completion diverged");
        const target = object(observation, "target");
        if (target.availability !== "resolved") {
            fail("prototype post-ready Rejected target became unavailable");
        }
        same(
            object(target, "identity"),
            expectedIdentity,
            "prototype post-ready Rejected target",
        );
    }

    return {
        ...gracefulCompletionInvariant(launch, "escape"),
        readiness: {
            simulation: simulationInvariant(launch),
            observation: idleObservationInvariant(launch),
            interaction: idleInteractionInvariant(launch),
            jump: jumpPolicyInvariant(launch, true),
            camera: cameraDriverInvariant(launch),
            traversal: traversalInvariant(launch, windowCenter),
        },
        nativeInput: nativeObjectActionInvariant(
            object(object(launch, "postReadinessInput"), "sequence"),
            number(launch, "processId"),
            true,
        ),
        expectedKind,
        exactSourceIdentity: expectedIdentity,
        actionAfterReadiness: true,
        exactCommittedOriginProximity: true,
        exactCommittedFacing: true,
        stationaryActor: true,
        acknowledgementFrameCount: 12,
        copiedObjectState: false,
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
    const finalInteraction = object(completion, "object_interaction");
    const finalObservation = object(completion, "object_observation");
    const consumed = object(finalInteraction, "consumed");
    if (
        finalInteraction.pending !== false ||
        finalInteraction.acknowledgement !== null ||
        number(finalInteraction, "committedCount") !== 1 ||
        number(finalInteraction, "ineligibleCount") !== 1 ||
        finalObservation.pending !== false ||
        finalObservation.target === null
    ) fail("prototype sustained capacity-one state diverged");
    same(
        object(finalInteraction, "nearestExclusion"),
        consumed,
        "prototype sustained nearest exclusion",
    );
    const finalTarget = object(finalObservation, "target");
    if (
        finalTarget.availability !== "resolved" ||
        JSON.stringify(object(finalTarget, "identity")) === JSON.stringify(consumed)
    ) fail("prototype sustained capacity rejection did not retain another resolved target");

    const frames = object(completion, "frames");
    if (
        number(frames, "liveFrameCount") <=
            number(object(readiness, "simulation_driver"), "liveFrameCount") ||
        number(frames, "activatedFrameCount") !== 12 ||
        number(frames, "rejectedFrameCount") !== 12 ||
        number(frames, "suppressionProjectedFrameCount") < 1 ||
        number(frames, "renderBlockCount") !== 0
    ) fail("prototype sustained session frame evidence diverged");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyPosition = object(object(object(readyActor, "motion"), "body"), "position");
    const finalPosition = object(object(object(finalActor, "motion"), "body"), "position");
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype sustained actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype sustained actor region",
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
    if (
        postReadiness.revision !== "prototype-post-ready-consumption-capacity-input-v1" ||
        number(postReadiness, "requestedConsumptionHoldMilliseconds") !== 250 ||
        number(postReadiness, "consumptionHoldMilliseconds") < 250
    ) fail("prototype sustained post-ready consumption timing diverged");
    const consumptionInput = nativeObjectActionInvariant(
        object(postReadiness, "consumption"),
        number(launch, "processId"),
        false,
    );
    const capacityInput = capacityRejectionInputInvariant(
        object(postReadiness, "capacity"),
        number(launch, "processId"),
    );

    return {
        ...session,
        readiness: {
            actor: actorInvariant(launch, windowCenter),
            observation: idleObservationInvariant(launch),
            interaction: idleInteractionInvariant(launch),
            jump: jumpPolicyInvariant(launch, true),
            camera: cameraDriverInvariant(launch),
            traversal: traversalInvariant(launch, windowCenter),
        },
        consumedIdentity: consumed,
        rejectedTargetIdentity: expectedIdentity,
        committedCount: 1,
        postReadinessIneligibleCount: 1,
        acknowledgement: null,
        activatedFrameCount: 12,
        capacityRejectedFrameCount: 12,
        suppressionProjectedFrameCount: number(frames, "suppressionProjectedFrameCount"),
        actorAdvancedAfterReadiness: true,
        postReadinessConsumption: consumptionInput,
        postReadinessCapacityRejection: capacityInput,
        independentExclusionOracle: true,
        exactCapacityOneRollback: true,
        copiedObjectState: false,
    };
}

function nativeObjectActionInvariant(
    evidence: Json,
    processId: number,
    delayedExit: boolean,
): Json {
    const intervals = evidence.keyPostIntervalsMilliseconds;
    const expectedMessages = [
        "WM_SETFOCUS",
        "WM_KEYDOWN:F",
        "WM_KEYDOWN:Enter",
        ...(delayedExit ? ["WM_KEYDOWN:Escape"] : []),
    ];
    if (
        evidence.schema !== "prototype-native-window-action-v4" ||
        evidence.action !== "input" ||
        number(evidence, "processId") !== processId ||
        evidence.requiredVisible !== true ||
        evidence.windowWasVisible !== true ||
        JSON.stringify(evidence.keys) !== JSON.stringify([
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: true },
            ]) ||
        JSON.stringify(evidence.messages) !== JSON.stringify(expectedMessages) ||
        JSON.stringify(evidence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        typeof intervals[0] !== "number" ||
        intervals[0] < 0 ||
        intervals[0] > 50 ||
        evidence.atomicBatch !== true ||
        number(evidence, "atomicPrefixLength") !== 2 ||
        !Number.isSafeInteger(evidence.batchThreadId) ||
        number(evidence, "batchThreadId") <= 0 ||
        number(evidence, "batchSpanMilliseconds") < 0 ||
        number(evidence, "batchSpanMilliseconds") > 50 ||
        number(evidence, "exitAfterLastMilliseconds") !== (delayedExit ? 200 : 0) ||
        (delayedExit
            ? number(evidence, "exitIntervalMilliseconds") < 200 ||
                number(evidence, "exitIntervalMilliseconds") > 700
            : evidence.exitIntervalMilliseconds !== null)
    ) fail("prototype post-ready native object action evidence diverged");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: evidence.batchThreadId,
        batchSpanMilliseconds: evidence.batchSpanMilliseconds,
        keyPostIntervalMilliseconds: intervals[0],
        orderedMessages: evidence.messages,
        delayedExit,
        exitIntervalMilliseconds: evidence.exitIntervalMilliseconds,
    };
}

function capacityRejectionInputInvariant(evidence: Json, processId: number): Json {
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
        ...evidence,
        exactProcessWindow: true,
        motionThenStationaryAction: true,
    };
}
