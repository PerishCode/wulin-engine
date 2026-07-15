import { type Coord, fail, type Json, number, object, same } from "../canonical-runtime.ts";

import { presentationInvariant } from "./presentation.ts";

export function actorInvariant(launch: Json, center: Coord): Json {
    const readiness = object(launch, "readiness");
    const actorAuthority = object(readiness, "actor");
    if (number(actorAuthority, "capacity") !== 1 || number(actorAuthority, "liveCount") !== 1) {
        fail("prototype readiness actor cardinality diverged");
    }
    const current = object(actorAuthority, "state");
    const batch = object(
        object(object(readiness, "simulation_driver"), "advance"),
        "actor",
    );
    const initial = object(batch, "input");
    same(current, object(batch, "output"), "prototype current actor authority");
    if (number(object(initial, "handle"), "generation") !== 1) {
        fail("prototype initial actor generation diverged");
    }
    const presentation = presentationInvariant(
        object(initial, "presentation"),
        0,
        0,
        "prototype initial actor",
    );
    const initialEpoch = number(initial, "animationEpochTick");
    if (initialEpoch < 0 || initialEpoch >= 31_002_560) {
        fail("prototype initial actor animation epoch is outside the clock period");
    }
    const motion = object(initial, "motion");
    const body = object(motion, "body");
    const position = object(body, "position");
    const region = object(position, "region");
    if (
        number(region, "x") !== center[0] || number(region, "z") !== center[1] ||
        number(position, "localXQ9") !== 0 || number(position, "localZQ9") !== 0
    ) fail("prototype initial actor position diverged");
    if (
        number(body, "halfHeightNumerator") !== 65_536 ||
        number(motion, "stepVelocityQ16") !== 0
    ) fail("prototype initial actor grounding diverged");
    return {
        capacity: 1,
        liveCount: 1,
        generation: 1,
        presentation,
        animationEpochTick: initialEpoch,
        initialAtCenter: true,
        currentMatchesAdvance: true,
    };
}
