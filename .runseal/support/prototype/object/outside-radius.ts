import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";

const TERRAIN_REGION_SPAN_Q9 = 8_192;

export function outsideRadiusActorInvariant(readyActor: Json, finalActor: Json): Json {
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype outside-radius actor handle",
    );
    same(
        object(finalActor, "presentation"),
        object(readyActor, "presentation"),
        "prototype outside-radius final Survey presentation",
    );
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");
    if (
        number(finalMotion, "stepVelocityQ16") !== 0 ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator")
    ) fail("prototype outside-radius final body diverged");
    const translation = exactDelta(
        object(readyBody, "position"),
        object(finalBody, "position"),
    );
    if (
        number(translation, "deltaXQ9") < 10 * 32 ||
        number(translation, "deltaXQ9") % 32 !== 0 ||
        number(translation, "deltaZQ9") !== 0
    ) fail("prototype outside-radius actor translation diverged");
    return {
        translation,
        readyAnimationEpochTick: readyActor.animationEpochTick,
        finalAnimationEpochTick: finalActor.animationEpochTick,
        finalPositionValidated: true,
        finalCenterHeightNumerator: finalBody.centerHeightNumerator,
    };
}

export function exactObjectProximity(origin: Json, target: Json): Json {
    const delta = exactDelta(origin, target);
    const deltaX = BigInt(number(delta, "deltaXQ9"));
    const deltaZ = BigInt(number(delta, "deltaZQ9"));
    const squared = deltaX * deltaX + deltaZ * deltaZ;
    const distanceSquaredQ18 = Number(squared);
    if (!Number.isSafeInteger(distanceSquaredQ18)) {
        fail("prototype exact object proximity exceeded the JSON-safe domain");
    }
    return { ...delta, distanceSquaredQ18 };
}

function exactDelta(origin: Json, target: Json): Json {
    const axis = (name: "x" | "z", local: "localXQ9" | "localZQ9"): number => {
        const originRegion = number(object(origin, "region"), name);
        const targetRegion = number(object(target, "region"), name);
        const delta = (BigInt(targetRegion) - BigInt(originRegion)) *
                BigInt(TERRAIN_REGION_SPAN_Q9) +
            BigInt(number(target, local) - number(origin, local));
        const value = Number(delta);
        if (!Number.isSafeInteger(value)) {
            fail(`prototype exact ${name.toUpperCase()} delta exceeded the JSON-safe domain`);
        }
        return value;
    };
    return {
        deltaXQ9: axis("x", "localXQ9"),
        deltaZQ9: axis("z", "localZQ9"),
    };
}
