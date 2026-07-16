import { type Coord, type Json, same } from "../../canonical-runtime.ts";
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
        observation: await observationInvariant(launch, objects, base, true, "selected"),
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
