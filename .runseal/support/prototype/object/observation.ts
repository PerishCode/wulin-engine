import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { objectNearestOracle } from "../../object/nearest.ts";

const OBSERVATION_RADIUS_Q9 = 512;

export async function observationInvariant(
    launch: Json,
    source: string,
    windowCenter: [number, number],
    expectedCompleted: boolean,
): Promise<Json> {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "object_observation_driver");
    const status = object(driver, "status");
    if (
        driver.revision !== "live-prototype-object-observation-v1" ||
        number(driver, "maxDistanceQ9") !== OBSERVATION_RADIUS_Q9 ||
        status.pending !== false ||
        driver.completed !== expectedCompleted
    ) fail("prototype object observation driver diverged");

    if (!expectedCompleted) {
        if (driver.observation !== null) {
            fail("prototype emitted an object observation without an intent");
        }
        return {
            revision: driver.revision,
            maxDistanceQ9: OBSERVATION_RADIUS_Q9,
            pending: false,
            completed: false,
            observation: null,
        };
    }
    const observation = object(driver, "observation");
    const origin = object(observation, "origin");
    const actorOutput = object(
        object(object(object(readiness, "simulation_driver"), "advance"), "actor"),
        "output",
    );
    const committedPosition = object(object(object(actorOutput, "motion"), "body"), "position");
    same(origin, committedPosition, "prototype object observation committed origin");
    const originRegion = object(origin, "region");
    const oracle = await objectNearestOracle(
        source,
        {
            region: [number(originRegion, "x"), number(originRegion, "z")],
            localXQ9: number(origin, "localXQ9"),
            localZQ9: number(origin, "localZQ9"),
            maxDistanceQ9: OBSERVATION_RADIUS_Q9,
        },
        windowCenter,
    );
    const query = object(observation, "query");
    same(query, oracle, "prototype object observation source oracle");
    const nearest = object(query, "nearest");
    const distanceSquaredQ18 = number(nearest, "distanceSquaredQ18");
    if (
        number(query, "candidateCount") !== 25_600 ||
        distanceSquaredQ18 > OBSERVATION_RADIUS_Q9 * OBSERVATION_RADIUS_Q9
    ) fail("prototype object observation bound diverged");

    return {
        revision: driver.revision,
        maxDistanceQ9: OBSERVATION_RADIUS_Q9,
        pending: false,
        completed: true,
        origin,
        query,
        exactCommittedOrigin: true,
        independentSourceOracle: true,
    };
}
