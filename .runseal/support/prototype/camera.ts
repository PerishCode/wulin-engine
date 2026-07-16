import { array, fail, type Json, number, object, same } from "../canonical-runtime.ts";

const CAMERA_RIGS = [
    { position: [9, 4, 12], target: [0, -1, -3] },
    { position: [12, 4, -9], target: [-3, -1, 0] },
    { position: [-9, 4, -12], target: [0, -1, 3] },
    { position: [-12, 4, 9], target: [3, -1, 0] },
];

function numericArray(value: Json, key: string): number[] {
    return array(value, key).map((entry) => {
        if (typeof entry !== "number" || !Number.isFinite(entry)) {
            fail(`prototype camera ${key} must contain finite numbers`);
        }
        return entry;
    });
}

export function cameraDriverInvariant(launch: Json, expectedOrbitIndex = 0): Json {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "camera_driver");
    if (driver.revision !== "live-prototype-actor-camera-v2") {
        fail("prototype camera driver revision diverged");
    }
    const actor = object(object(readiness, "actor"), "state");
    same(object(driver, "actor"), object(actor, "handle"), "prototype camera actor handle");
    const rig = object(driver, "rig");
    if (number(rig, "orbitIndex") !== expectedOrbitIndex) {
        fail("prototype camera orbit index diverged");
    }
    const expectedRig = CAMERA_RIGS[expectedOrbitIndex];
    if (expectedRig === undefined) fail("prototype expected camera orbit index is invalid");
    same(
        numericArray(rig, "positionOffset"),
        expectedRig.position,
        "prototype camera position rig",
    );
    same(
        numericArray(rig, "targetOffset"),
        expectedRig.target,
        "prototype camera target rig",
    );
    if (number(rig, "verticalFovDegrees") !== 60) {
        fail("prototype camera field of view diverged");
    }
    const centerHeightQ16 = number(
        object(object(actor, "motion"), "body"),
        "centerHeightNumerator",
    );
    const position = object(object(object(actor, "motion"), "body"), "position");
    const actorX = number(position, "localXQ9") / 512;
    const actorZ = number(position, "localZQ9") / 512;
    const camera = object(driver, "camera");
    const anchorY = centerHeightQ16 / 65_536;
    same(
        numericArray(camera, "position"),
        [
            actorX + expectedRig.position[0],
            anchorY + expectedRig.position[1],
            actorZ + expectedRig.position[2],
        ],
        "prototype anchored camera position",
    );
    same(
        numericArray(camera, "target"),
        [
            actorX + expectedRig.target[0],
            anchorY + expectedRig.target[1],
            actorZ + expectedRig.target[2],
        ],
        "prototype anchored camera target",
    );
    if (
        number(camera, "verticalFovDegrees") !== 60 ||
        number(camera, "nearPlaneMeters") !== Math.fround(0.1)
    ) fail("prototype anchored camera lens diverged");
    const liveFrames = number(driver, "liveFrameCount");
    if (
        liveFrames !== number(object(readiness, "simulation_driver"), "liveFrameCount") ||
        number(driver, "anchorCount") !== liveFrames || liveFrames < 1
    ) fail("prototype camera/frame ordering diverged");
    return {
        revision: driver.revision,
        actor: driver.actor,
        orbitIndex: expectedOrbitIndex,
        rig,
        camera,
        anchorPerLiveFrame: true,
    };
}
