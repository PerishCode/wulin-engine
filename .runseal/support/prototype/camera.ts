import { array, fail, type Json, number, object, same } from "../canonical-runtime.ts";
import { presentationInvariant } from "./presentation.ts";

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
    const actualOrbitIndex = number(rig, "orbitIndex");
    if (actualOrbitIndex !== expectedOrbitIndex) {
        fail(
            `prototype camera orbit index diverged: expected ${expectedOrbitIndex}, got ${actualOrbitIndex}`,
        );
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

function nativeCameraRepeatInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const startup = object(launch, "startupNativeInput");
    const sequence = object(object(launch, "postReadinessInput"), "sequence");
    const expectedStartupKeys = [{ key: "E", virtualKey: 69, down: true }];
    const expectedSequenceKeys = [
        { key: "E", virtualKey: 69, down: true },
        { key: "W", virtualKey: 87, down: true },
    ];
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const exitInterval = number(sequence, "exitIntervalMilliseconds");
    if (
        startup.schema !== "prototype-native-window-action-v4" ||
        startup.action !== "input" ||
        startup.processId !== processId ||
        startup.requiredVisible !== true ||
        startup.windowWasVisible !== true ||
        JSON.stringify(startup.keys) !== JSON.stringify(expectedStartupKeys) ||
        JSON.stringify(startup.messages) !==
            JSON.stringify(["WM_SETFOCUS", "WM_KEYDOWN:E"]) ||
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.windowHandle !== startup.windowHandle ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !== JSON.stringify(expectedSequenceKeys) ||
        JSON.stringify(sequence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:E",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        number(sequence, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700
    ) fail("prototype native held-camera-repeat evidence diverged");
    return {
        exactProcessWindow: true,
        initialPress: startup.messages,
        repeatedHeldPress: sequence.messages,
        exitIntervalMilliseconds: exitInterval,
    };
}

export function cameraRepeatSessionInvariant(launch: Json, session: Json): Json {
    const camera = cameraDriverInvariant(launch, 1);
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");
    const readyPosition = object(readyBody, "position");
    const finalPosition = object(finalBody, "position");
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype held-camera-repeat actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype held-camera-repeat actor region",
    );
    if (
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype held-camera-repeat vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 >= 0 || deltaXQ9 % 32 !== 0 || deltaZQ9 !== 0) {
        fail("prototype held-camera-repeat did not retain orbit-one locomotion");
    }
    const horizontalSteps = -deltaXQ9 / 32;
    if (horizontalSteps < 1 || horizontalSteps > 43) {
        fail("prototype held-camera-repeat horizontal step count diverged");
    }
    const presentation = presentationInvariant(
        object(finalActor, "presentation"),
        1,
        32_768,
        "prototype held-camera-repeat Walk",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype held-camera-repeat did not commit the Walk transition");

    const readyClock = object(object(readiness, "simulation_driver"), "clock");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype held-camera-repeat clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeCameraRepeatInvariant(launch),
        readinessCamera: camera,
        heldRepeatSuppressed: true,
        retainedOrbitIndex: 1,
        horizontalSteps,
        deltaXQ9,
        deltaZQ9,
        presentation,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}

function nativeInvalidKeyInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const sequence = object(object(launch, "postReadinessInput"), "sequence");
    const expectedKeys = [
        { key: "OutOfRangeE", virtualKey: 0x145, down: true },
        { key: "W", virtualKey: 0x57, down: true },
    ];
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const exitInterval = number(sequence, "exitIntervalMilliseconds");
    if (
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !== JSON.stringify(expectedKeys) ||
        JSON.stringify(sequence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:OutOfRangeE",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 1 ||
        number(sequence, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700
    ) fail("prototype native out-of-range camera-key evidence diverged");
    return {
        exactProcessWindow: true,
        virtualKey: 0x145,
        truncatedAliasVirtualKey: 0x45,
        messages: sequence.messages,
        keyPostIntervalMilliseconds: intervals[0],
        exitIntervalMilliseconds: exitInterval,
    };
}

export function invalidKeySessionInvariant(launch: Json, session: Json): Json {
    const camera = cameraDriverInvariant(launch, 0);
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");
    const readyPosition = object(readyBody, "position");
    const finalPosition = object(finalBody, "position");
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype invalid-key actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype invalid-key actor region",
    );
    if (
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype invalid-key vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 !== 0 || deltaZQ9 >= 0 || deltaZQ9 % 32 !== 0) {
        fail("prototype invalid-key input did not retain orbit-zero locomotion");
    }
    const horizontalSteps = -deltaZQ9 / 32;
    if (horizontalSteps < 1 || horizontalSteps > 43) {
        fail("prototype invalid-key horizontal step count diverged");
    }
    const presentation = presentationInvariant(
        object(finalActor, "presentation"),
        1,
        49_152,
        "prototype invalid-key Walk",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype invalid-key input did not commit the Walk transition");

    const readyClock = object(object(readiness, "simulation_driver"), "clock");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype invalid-key clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeInvalidKeyInvariant(launch),
        readinessCamera: camera,
        checkedRangeRejected: true,
        retainedOrbitIndex: 0,
        truncationWouldAlias: "E",
        horizontalSteps,
        deltaXQ9,
        deltaZQ9,
        presentation,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}

function nativeOppositeCameraInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const sequence = object(object(launch, "postReadinessInput"), "sequence");
    const expectedKeys = [
        { key: "Q", virtualKey: 0x51, down: true },
        { key: "E", virtualKey: 0x45, down: true },
        { key: "W", virtualKey: 0x57, down: true },
    ];
    const intervals = sequence.keyPostIntervalsMilliseconds;
    const exitInterval = number(sequence, "exitIntervalMilliseconds");
    if (
        sequence.schema !== "prototype-native-window-action-v4" ||
        sequence.action !== "input" ||
        sequence.processId !== processId ||
        sequence.requiredVisible !== true ||
        sequence.windowWasVisible !== true ||
        JSON.stringify(sequence.keys) !== JSON.stringify(expectedKeys) ||
        JSON.stringify(sequence.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Q",
                "WM_KEYDOWN:E",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(sequence.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0, 0]) ||
        !Array.isArray(intervals) ||
        intervals.length !== 2 ||
        intervals.some((interval) => typeof interval !== "number" || interval < 0) ||
        sequence.atomicBatch !== true ||
        !Number.isSafeInteger(sequence.batchThreadId) ||
        number(sequence, "batchThreadId") <= 0 ||
        number(sequence, "batchSpanMilliseconds") < 0 ||
        number(sequence, "batchSpanMilliseconds") > 50 ||
        number(sequence, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700
    ) fail("prototype native opposite-camera evidence diverged");
    same(sequence, object(launch, "exitInput"), "prototype opposite-camera exit input");
    return {
        exactProcessWindow: true,
        messages: sequence.messages,
        atomicBatch: true,
        batchThreadId: sequence.batchThreadId,
        batchSpanMilliseconds: sequence.batchSpanMilliseconds,
        keyPostIntervalsMilliseconds: intervals,
        exitIntervalMilliseconds: exitInterval,
    };
}

export function oppositeCameraSessionInvariant(launch: Json, session: Json): Json {
    const camera = cameraDriverInvariant(launch, 0);
    const readiness = object(launch, "readiness");
    const completion = object(launch, "completion");
    const readyActor = object(object(readiness, "actor"), "state");
    const finalActor = object(object(completion, "actor"), "state");
    const readyMotion = object(readyActor, "motion");
    const finalMotion = object(finalActor, "motion");
    const readyBody = object(readyMotion, "body");
    const finalBody = object(finalMotion, "body");
    const readyPosition = object(readyBody, "position");
    const finalPosition = object(finalBody, "position");
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype opposite-camera actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype opposite-camera actor region",
    );
    if (
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype opposite-camera vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 !== 0 || deltaZQ9 >= 0 || deltaZQ9 % 32 !== 0) {
        fail("prototype opposite camera edges did not cancel to orbit-zero locomotion");
    }
    const horizontalSteps = -deltaZQ9 / 32;
    if (horizontalSteps < 1 || horizontalSteps > 43) {
        fail("prototype opposite-camera horizontal step count diverged");
    }
    const presentation = presentationInvariant(
        object(finalActor, "presentation"),
        1,
        49_152,
        "prototype opposite-camera Walk",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype opposite camera edges did not commit the Walk transition");

    const readyClock = object(object(readiness, "simulation_driver"), "clock");
    const finalClock = object(completion, "clock");
    if (
        finalClock.suspended !== false ||
        finalClock.hasBaseline !== true ||
        number(finalClock, "suspendCount") !== number(readyClock, "suspendCount") ||
        number(finalClock, "resumeCount") !== number(readyClock, "resumeCount") ||
        number(finalClock, "resetCount") !== number(readyClock, "resetCount") ||
        number(finalClock, "stallCount") !== number(readyClock, "stallCount") ||
        number(finalClock, "readyCount") <= number(readyClock, "readyCount") ||
        number(finalClock, "sampleCount") <= number(readyClock, "sampleCount") ||
        number(object(completion, "frames"), "renderBlockCount") !== 0
    ) fail("prototype opposite-camera clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeOppositeCameraInvariant(launch),
        readinessCamera: camera,
        oppositePressEdgesRetained: true,
        cameraCandidateCancelled: true,
        retainedOrbitIndex: 0,
        horizontalSteps,
        deltaXQ9,
        deltaZQ9,
        presentation,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}
