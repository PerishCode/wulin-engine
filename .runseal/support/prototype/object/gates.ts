import { type Coord, fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { objectNearestOracle } from "../../object/nearest.ts";
import { actorInvariant } from "../actor.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { jumpPolicyInvariant } from "../jump.ts";
import { gracefulCompletionInvariant } from "../sessions/mod.ts";
import { traversalInvariant } from "../traversal.ts";
import {
    missingTargetInputInvariant,
    nativeSelectionInvariant,
    objectRecoveryInputInvariant,
    outsideRadiusInputInvariant,
} from "./input-gates.ts";
import { idleInteractionInvariant } from "./interaction.ts";
import { idleObservationInvariant } from "./observation.ts";
import { exactObjectProximity, outsideRadiusActorInvariant } from "./outside-radius.ts";

type StartupInvariant = (launch: Json) => Json;
type SimulationInvariant = (launch: Json) => Json;

const ACTION_RADIUS_Q9 = 512;

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
    admittedStartup: Json,
    rejectedStartup: Json,
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
            admittedStartup,
            objects,
            admittedBase,
            "activated",
            startupInvariant,
            admittedSimulation,
        ),
        rejected: await feedbackSessionInvariant(
            rejected,
            rejectedStartup,
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
    expectedStartup: Json,
    source: string,
    windowCenter: Coord,
    expectedKind: "activated" | "rejected",
    startupInvariant: StartupInvariant,
    simulationInvariant: SimulationInvariant,
): Promise<Json> {
    same(
        startupInvariant(launch),
        expectedStartup,
        `prototype post-ready ${expectedKind} configuration`,
    );
    actorInvariant(launch, windowCenter);
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyPosition = object(object(object(readyActor, "motion"), "body"), "position");
    const finalPosition = object(object(object(finalActor, "motion"), "body"), "position");
    const readyRegion = object(readyPosition, "region");
    const expected = await objectNearestOracle(
        source,
        {
            region: [number(readyRegion, "x"), number(readyRegion, "z")],
            localXQ9: number(readyPosition, "localXQ9"),
            localZQ9: number(readyPosition, "localZQ9"),
            maxDistanceQ9: ACTION_RADIUS_Q9,
        },
        windowCenter,
    );
    const expectedNearest = object(expected, "nearest");
    const expectedIdentity = object(object(expectedNearest, "object"), "identity");
    const finalTargetProximity = exactObjectProximity(
        finalPosition,
        object(expectedNearest, "terrainPosition"),
    );
    let actorTransition: Json | null = null;
    if (expectedKind === "activated") {
        same(finalActor, readyActor, "prototype post-ready Activated stationary actor");
        if (number(finalTargetProximity, "distanceSquaredQ18") > ACTION_RADIUS_Q9 ** 2) {
            fail("prototype post-ready Activated target left the action radius");
        }
    } else {
        actorTransition = outsideRadiusActorInvariant(readyActor, finalActor);
        if (number(finalTargetProximity, "distanceSquaredQ18") <= ACTION_RADIUS_Q9 ** 2) {
            fail("prototype post-ready Rejected target remained inside the action radius");
        }
    }
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
            number(interaction, "ineligibleCount") !== 1 ||
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
            number(interaction, "ineligibleCount") !== 2 ||
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

    const launchInput = object(launch, "nativeInput");
    const processId = number(launch, "processId");
    let nativeInput: Json;
    let focusRecovery: Json | null = null;
    if (expectedKind === "activated") {
        const focus = nativeObjectFocusInvariant(
            launch,
            launchInput,
            processId,
        );
        focusRecovery = object(focus, "focusRecovery");
        nativeInput = object(focus, "nativeInput");
    } else {
        const initialRejection = object(launchInput, "initialRejection");
        nativeInput = {
            initialRejection: nativeSelectionInvariant(
                initialRejection,
                processId,
            ),
            rangeMotion: outsideRadiusInputInvariant(
                launchInput,
                processId,
                initialRejection.windowHandle,
            ),
        };
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
        nativeInput,
        ...(focusRecovery === null ? {} : { focusRecovery }),
        expectedKind,
        exactSourceIdentity: true,
        finalTargetProximity,
        exactCommittedOriginProximity: true,
        exactCommittedFacing: true,
        ...(actorTransition === null ? { stationaryActor: true } : { actorTransition }),
        acknowledgementFrameCount: 12,
    };
}

function nativeObjectFocusInvariant(
    launch: Json,
    postReadiness: Json,
    processId: number,
): Json {
    const suspended = object(postReadiness, "suspended");
    const resumed = object(postReadiness, "resumed");
    const missingTarget = object(postReadiness, "missingTarget");
    const sequence = object(postReadiness, "sequence");
    if (
        suspended.schema !== "prototype-native-window-action-v4" ||
        suspended.action !== "suspend" ||
        number(suspended, "processId") !== processId ||
        suspended.activated !== true ||
        suspended.closeRequested !== false ||
        suspended.requiredVisible !== true ||
        suspended.windowWasVisible !== true ||
        JSON.stringify(suspended.keys) !== JSON.stringify([
                { key: "F", virtualKey: 70, down: true },
                { key: "Enter", virtualKey: 13, down: true },
            ]) ||
        JSON.stringify(suspended.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:F",
                "WM_KEYDOWN:Enter",
                "WM_KILLFOCUS",
            ]) ||
        JSON.stringify(suspended.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0]) ||
        !Array.isArray(suspended.keyPostIntervalsMilliseconds) ||
        suspended.keyPostIntervalsMilliseconds.length !== 1 ||
        typeof suspended.keyPostIntervalsMilliseconds[0] !== "number" ||
        suspended.keyPostIntervalsMilliseconds[0] < 0 ||
        suspended.keyPostIntervalsMilliseconds[0] > 50 ||
        suspended.atomicBatch !== true ||
        number(suspended, "atomicPrefixLength") !== 2 ||
        !Number.isSafeInteger(suspended.batchThreadId) ||
        number(suspended, "batchThreadId") <= 0 ||
        number(suspended, "batchSpanMilliseconds") < 0 ||
        number(suspended, "batchSpanMilliseconds") > 50 ||
        number(suspended, "exitAfterLastMilliseconds") !== 0 ||
        suspended.exitIntervalMilliseconds !== null ||
        resumed.schema !== "prototype-native-window-action-v4" ||
        resumed.action !== "resume" ||
        number(resumed, "processId") !== processId ||
        resumed.windowHandle !== suspended.windowHandle ||
        resumed.activated !== true ||
        resumed.closeRequested !== false ||
        resumed.requiredVisible !== true ||
        resumed.windowWasVisible !== true ||
        !Array.isArray(resumed.keys) ||
        resumed.keys.length !== 0 ||
        JSON.stringify(resumed.messages) !== JSON.stringify(["WM_SETFOCUS"]) ||
        number(postReadiness, "requestedMissingHoldMilliseconds") !== 250 ||
        number(postReadiness, "missingHoldMilliseconds") < 250 ||
        missingTarget.windowHandle !== suspended.windowHandle ||
        sequence.windowHandle !== suspended.windowHandle
    ) fail("prototype native object focus-readmission evidence diverged");

    const readyClock = object(object(object(launch, "readiness"), "simulation_driver"), "clock");
    const completion = object(launch, "completion");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") + 1 ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") + 1 ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") + 1 ||
        number(finalClock, "suspendedSampleCount") <=
            number(readyClock, "suspendedSampleCount") ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype object focus-readmission clock recovery diverged");

    const nativeInput = objectRecoveryInputInvariant(
        sequence,
        processId,
        suspended.windowHandle,
    );
    return {
        nativeInput,
        focusRecovery: {
            exactProcessWindow: true,
            exactSuspendedMessageOrder: true,
            exactResumedMessageOrder: true,
            atomicCancelledIntents: {
                threadId: suspended.batchThreadId,
                spanMilliseconds: suspended.batchSpanMilliseconds,
            },
            missingTarget: missingTargetInputInvariant(
                missingTarget,
                processId,
                suspended.windowHandle,
            ),
            missingHoldMilliseconds: postReadiness.missingHoldMilliseconds,
            clock: {
                continuityValidated: true,
                exactSuspendResumeCount: 1,
                postResumeResetCount: 1,
                elapsedBacklog: false,
            },
        },
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

    const launchInput = object(launch, "nativeInput");
    if (
        launchInput.revision !== "prototype-post-ready-consumption-capacity-input-v1" ||
        number(launchInput, "requestedConsumptionHoldMilliseconds") !== 250 ||
        number(launchInput, "consumptionHoldMilliseconds") < 250
    ) fail("prototype sustained post-ready consumption timing diverged");
    const consumptionInput = nativeSelectionInvariant(
        object(launchInput, "consumption"),
        number(launch, "processId"),
    );
    const capacityInput = capacityRejectionInputInvariant(
        object(launchInput, "capacity"),
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
        exactConsumedIdentity: true,
        exactRejectedTargetIdentity: true,
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
        revision: evidence.revision,
        requestedMotionHoldMilliseconds: evidence.requestedMotionHoldMilliseconds,
        motionHoldMilliseconds: evidence.motionHoldMilliseconds,
        exactProcessWindow: true,
        exactMotionKeys: true,
        exactActionKeys: true,
        motionThenStationaryAction: true,
    };
}
