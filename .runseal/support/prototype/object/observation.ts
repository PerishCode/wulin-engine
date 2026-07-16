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
        driver.revision !== "live-prototype-object-target-v2" ||
        number(driver, "maxDistanceQ9") !== OBSERVATION_RADIUS_Q9 ||
        status.pending !== false ||
        driver.completed !== expectedCompleted
    ) fail("prototype object observation driver diverged");

    if (!expectedCompleted) {
        if (driver.observation !== null || status.target !== null) {
            fail("prototype retained an object observation or target without an intent");
        }
        return {
            revision: driver.revision,
            maxDistanceQ9: OBSERVATION_RADIUS_Q9,
            pending: false,
            completed: false,
            observation: null,
            target: null,
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
    const observedIdentity = object(object(nearest, "object"), "identity");
    const snapshot = object(observation, "snapshot");
    const publicationToken = number(snapshot, "publicationToken");
    if (
        !Number.isSafeInteger(publicationToken) || publicationToken < 1 ||
        snapshot.sourceNamespace !== observedIdentity.sourceNamespace
    ) fail("prototype object observation snapshot diverged from its qualified result");
    const target = object(status, "target");
    if (target.availability !== "resolved") {
        fail("prototype newly observed target was not resolved");
    }
    same(object(target, "identity"), observedIdentity, "prototype retained target identity");
    const targetSnapshot = object(target, "snapshot");
    const targetToken = number(targetSnapshot, "publicationToken");
    if (targetSnapshot.sourceNamespace !== snapshot.sourceNamespace) {
        fail("prototype target validation changed source without clearing the target");
    }
    const traversal = object(readiness, "traversal");
    const traversalPublicationCount = number(traversal, "automaticPublicationCount");
    if (targetToken === publicationToken) {
        if (traversalPublicationCount !== 0 || traversal.lastPublished !== null) {
            fail("prototype target missed a completed traversal publication");
        }
    } else {
        const lastPublished = object(traversal, "lastPublished");
        if (
            targetToken <= publicationToken || traversalPublicationCount !== 1 ||
            number(lastPublished, "token") !== targetToken
        ) fail("prototype target validation snapshot diverged from traversal publication");
    }
    if ("object" in target || "position" in target || "presentation" in target) {
        fail("prototype retained copied canonical object content in target state");
    }
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
        target,
        exactCommittedOrigin: true,
        independentSourceOracle: true,
        snapshotGatedTarget: true,
        revalidatedAfterPublication: targetToken !== publicationToken,
        copiedObjectState: false,
    };
}
