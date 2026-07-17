import { fail, type Json, number, object, same } from "../../canonical-runtime.ts";
import { cameraDriverInvariant } from "../camera.ts";
import { presentationInvariant } from "../presentation.ts";

function nativeOppositionInvariant(launch: Json): Json {
    const processId = number(launch, "processId");
    const startup = object(launch, "startupNativeInput");
    const startupIntervals = startup.keyPostIntervalsMilliseconds;
    if (
        startup.schema !== "prototype-native-window-action-v3" ||
        startup.action !== "input" ||
        startup.processId !== processId ||
        startup.requiredVisible !== true ||
        startup.windowWasVisible !== true ||
        JSON.stringify(startup.keys) !== JSON.stringify([
                { key: "Shift", virtualKey: 0x10, down: true },
                { key: "W", virtualKey: 0x57, down: true },
                { key: "S", virtualKey: 0x53, down: true },
            ]) ||
        JSON.stringify(startup.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYDOWN:Shift",
                "WM_KEYDOWN:W",
                "WM_KEYDOWN:S",
            ]) ||
        JSON.stringify(startup.delaysBeforeKeysMilliseconds) !== JSON.stringify([0, 0, 0]) ||
        !Array.isArray(startupIntervals) ||
        startupIntervals.length !== 2 ||
        startupIntervals.some((interval) =>
            typeof interval !== "number" || interval < 0 || interval > 50
        ) ||
        startup.atomicBatch !== true ||
        typeof startup.batchThreadId !== "number" ||
        !Number.isSafeInteger(startup.batchThreadId) ||
        startup.batchThreadId <= 0 ||
        typeof startup.batchSpanMilliseconds !== "number" ||
        startup.batchSpanMilliseconds < 0 ||
        startup.batchSpanMilliseconds > 50 ||
        startup.exitAfterLastMilliseconds !== 0 ||
        startup.exitIntervalMilliseconds !== null
    ) fail("prototype native opposite locomotion startup evidence diverged");

    const release = object(object(launch, "postReadinessInput"), "sequence");
    const exitInterval = number(release, "exitIntervalMilliseconds");
    if (
        release.schema !== "prototype-native-window-action-v3" ||
        release.action !== "input" ||
        release.processId !== processId ||
        release.requiredVisible !== true ||
        release.windowWasVisible !== true ||
        JSON.stringify(release.keys) !== JSON.stringify([
                { key: "S", virtualKey: 0x53, down: false },
            ]) ||
        JSON.stringify(release.messages) !== JSON.stringify([
                "WM_SETFOCUS",
                "WM_KEYUP:S",
                "WM_KEYDOWN:Escape",
            ]) ||
        JSON.stringify(release.delaysBeforeKeysMilliseconds) !== JSON.stringify([0]) ||
        !Array.isArray(release.keyPostIntervalsMilliseconds) ||
        release.keyPostIntervalsMilliseconds.length !== 0 ||
        release.atomicBatch !== false ||
        release.batchThreadId !== null ||
        release.batchSpanMilliseconds !== null ||
        number(release, "exitAfterLastMilliseconds") !== 200 ||
        exitInterval < 200 ||
        exitInterval > 700
    ) fail("prototype native opposite locomotion release evidence diverged");
    same(release, object(launch, "exitInput"), "prototype opposite locomotion exit input");
    return {
        exactProcessWindow: true,
        atomicWindowThreadBatch: true,
        batchThreadId: startup.batchThreadId,
        batchSpanMilliseconds: startup.batchSpanMilliseconds,
        orderedStartupMessages: startup.messages,
        orderedReleaseMessages: release.messages,
        exitIntervalMilliseconds: exitInterval,
    };
}

export function locomotionOppositionSessionInvariant(launch: Json, session: Json): Json {
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
    const readyPresentation = presentationInvariant(
        object(readyActor, "presentation"),
        0,
        0,
        "prototype opposite locomotion stationary readiness",
    );
    same(
        object(finalActor, "handle"),
        object(readyActor, "handle"),
        "prototype opposite locomotion actor handle",
    );
    same(
        object(finalPosition, "region"),
        object(readyPosition, "region"),
        "prototype opposite locomotion actor region",
    );
    if (
        number(readyPosition, "localXQ9") !== 0 ||
        number(readyPosition, "localZQ9") !== 0 ||
        number(finalBody, "halfHeightNumerator") !==
            number(readyBody, "halfHeightNumerator") ||
        number(finalMotion, "stepVelocityQ16") !== 0
    ) fail("prototype opposite locomotion stationary or vertical state diverged");

    const deltaXQ9 = number(finalPosition, "localXQ9") -
        number(readyPosition, "localXQ9");
    const deltaZQ9 = number(finalPosition, "localZQ9") -
        number(readyPosition, "localZQ9");
    if (deltaXQ9 !== 0 || deltaZQ9 >= 0 || deltaZQ9 % 64 !== 0) {
        fail("prototype opposite locomotion release did not readmit retained Run");
    }
    const runStepCount = -deltaZQ9 / 64;
    if (runStepCount < 1 || runStepCount > 512) {
        fail("prototype opposite locomotion Run step bound diverged");
    }
    const finalPresentation = presentationInvariant(
        object(finalActor, "presentation"),
        2,
        49_152,
        "prototype opposite locomotion released Run",
    );
    if (
        number(finalActor, "animationEpochTick") <=
            number(readyActor, "animationEpochTick")
    ) fail("prototype opposite locomotion did not commit the Run transition");

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
    ) fail("prototype opposite locomotion clock continuity diverged");

    return {
        ...session,
        nativeInput: nativeOppositionInvariant(launch),
        readinessCamera: camera,
        oppositeAxisCancelled: true,
        stationarySurveyReadiness: true,
        releasedBackwardInput: true,
        retainedForwardRunReadmitted: true,
        deltaXQ9,
        deltaZQ9,
        runStepCount,
        readyPresentation,
        finalPresentation,
        clock: {
            ready: readyClock,
            final: finalClock,
            discontinuity: false,
        },
    };
}
