import {
    array,
    type Coord,
    fail,
    type Json,
    lifecycle,
    number,
    object,
    root,
    same,
    string,
    useSidecar,
} from "../canonical-runtime.ts";
import { jumpMotionInvariant, jumpPolicyInvariant } from "./jump.ts";
import { actorInvariant } from "./actor.ts";
import { BOUNDARY_HOLD_MILLISECONDS, boundarySurvival } from "./boundary.ts";
import { cameraDriverInvariant } from "./camera.ts";
import { presentationInvariant } from "./presentation.ts";
import { objectFeedbackGates, restartObservation } from "./object/gates.ts";
import {
    capturedReady as captureReady,
    objectFeedbackSession,
    sustainedCapacitySession,
} from "./sessions/mod.ts";
import { sessionGates } from "./sessions/gates.ts";
import { type ExpectedCommand, STATIONARY_COMMAND } from "./simulation.ts";
import { traversalInvariant } from "./traversal.ts";

export const CONFIG = "out/cooked/bootstrap/runtime.json";
export const SIDECAR = "sidecar.prototype.toml";
const EXECUTABLE = "target/debug/prototype.exe";
const decoder = new TextDecoder();

export async function sidecarStatus(config: string): Promise<Json> {
    const output = await new Deno.Command("sidecar", {
        args: ["status", "--config", config, "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail(`prototype Sidecar status failed with ${output.code}`);
    return JSON.parse(decoder.decode(output.stdout).trim()) as Json;
}

export function prototypePids(status: Json): number[] {
    const targets = array(status, "targets");
    if (targets.length !== 1) fail("prototype Sidecar target count diverged");
    const target = targets[0] as Json;
    if (target.name !== "prototype") fail("prototype Sidecar target identity diverged");
    return array(target, "pids").map((value) => {
        if (typeof value !== "number") fail("prototype Sidecar PID must be numeric");
        return value;
    });
}

export function document(terrain: string, objects: string, center: Coord): Json {
    return {
        schemaVersion: 2,
        terrain,
        objects,
        globalOrigin: { x: center[0], z: center[1] },
        globalCenter: { x: center[0], z: center[1] },
        activeRadius: 2,
        playableRegionBounds: {
            minimum: { x: center[0], z: center[1] },
            maximum: { x: center[0], z: center[1] },
        },
    };
}

export async function writeDocument(value: Json): Promise<void> {
    await Deno.mkdir(`${root}/out/cooked/bootstrap`, { recursive: true });
    await Deno.writeTextFile(`${root}/${CONFIG}`, `${JSON.stringify(value, null, 2)}\n`);
}

export async function failedStart(label: string): Promise<Json> {
    const started = performance.now();
    const output = await new Deno.Command(EXECUTABLE, {
        args: [`--bootstrap=${CONFIG}`],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    const stderr = decoder.decode(output.stderr).trim();
    if (output.success) fail(`prototype ${label} unexpectedly succeeded`);
    if (
        stdout.includes('"role":"prototype"') ||
        stdout.includes('"role":"prototype-session-completion"')
    ) {
        fail(`prototype ${label} emitted successful session output before failing`);
    }
    return {
        label,
        code: output.code,
        elapsedMs: performance.now() - started,
        stdout: stdout.slice(-4_096),
        stderr: stderr.slice(-4_096),
        readinessEmitted: false,
    };
}

export async function capturedReady(label: string): Promise<Json> {
    return await captureReady(EXECUTABLE, CONFIG, label);
}

export function startupInvariant(launch: Json): Json {
    const startup = object(object(launch, "readiness"), "startup");
    return {
        revision: startup.revision,
        mode: startup.mode,
        configPath: startup.configPath,
        configBytes: startup.configBytes,
        configSha256: startup.configSha256,
        terrainPath: startup.terrainPath,
        objectPath: startup.objectPath,
        globalConfig: startup.globalConfig,
        playableRegionBounds: startup.playableRegionBounds,
    };
}

export function simulationDriverInvariant(launch: Json, expected: ExpectedCommand): Json {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "simulation_driver");
    if (driver.revision !== "live-prototype-locomotion-driver-v8") {
        fail("prototype simulation driver revision diverged");
    }
    if (number(driver, "renderBlockCount") !== 0) {
        fail("prototype normal readiness encountered render backpressure");
    }
    const sample = object(driver, "sample");
    const elapsed = number(sample, "elapsedNanoseconds");
    if (sample.outcome !== "ready" || elapsed < 0 || elapsed > 125_000_000) {
        fail("prototype simulation driver sample diverged");
    }
    const clock = object(driver, "clock");
    const sampleCount = number(clock, "sampleCount");
    if (
        clock.suspended !== false || clock.hasBaseline !== true ||
        number(clock, "resetCount") < 1 || number(clock, "readyCount") < 1 ||
        sampleCount !== number(clock, "resetCount") + number(clock, "readyCount") +
                number(clock, "stallCount") + number(clock, "suspendedSampleCount")
    ) fail("prototype simulation driver clock status diverged");
    const command = object(driver, "command");
    if (
        number(command, "deltaXQ9") !== expected.deltaXQ9 ||
        number(command, "deltaZQ9") !== expected.deltaZQ9 ||
        number(command, "stepUpLimitQ16") !== expected.stepUpLimitQ16 ||
        number(command, "initialStepVelocityDeltaQ16") !== expected.initialVelocityDeltaQ16
    ) {
        fail(
            `prototype simulation locomotion command diverged: expected=${
                JSON.stringify(expected)
            } actual=${JSON.stringify(command)}`,
        );
    }
    if (number(command, "stepAccelerationQ16") !== -179) {
        fail("prototype gravity command diverged");
    }
    const commandPresentation = presentationInvariant(
        object(command, "presentation"),
        expected.animationClip,
        expected.yawQ16,
        "prototype simulation command",
    );
    const advance = object(driver, "advance");
    const simulation = object(advance, "simulation");
    const stepCount = number(simulation, "stepCount");
    if (
        number(simulation, "elapsedNanoseconds") !== elapsed ||
        number(simulation, "startTick") !== 0 || stepCount < 1 || stepCount > 8 ||
        number(simulation, "endTick") !== stepCount ||
        number(simulation, "remainderDenominator") !== 1_000_000_000
    ) fail("prototype live simulation advance diverged");
    const actor = object(advance, "actor");
    if (
        number(actor, "stepCount") !== stepCount ||
        number(actor, "terrainQueryCount") !== stepCount ||
        actor.lastStepGrounded !== expected.groundedAfterBatch
    ) fail("prototype live actor batch diverged");
    const initial = object(actor, "input");
    const output = object(actor, "output");
    const inputPresentation = presentationInvariant(
        object(initial, "presentation"),
        0,
        0,
        "prototype simulation input",
    );
    const outputPresentation = presentationInvariant(
        object(output, "presentation"),
        expected.animationClip,
        expected.yawQ16,
        "prototype simulation output",
    );
    const inputEpoch = number(initial, "animationEpochTick");
    const outputEpoch = number(output, "animationEpochTick");
    let jumpMotion: Json | null = null;
    if (expected.initialVelocityDeltaQ16 !== 0) {
        jumpMotion = jumpMotionInvariant(initial, output, stepCount);
    } else if (expected.deltaXQ9 === 0 && expected.deltaZQ9 === 0) {
        same(output, initial, "prototype stationary actor output");
    } else {
        same(object(output, "handle"), object(initial, "handle"), "prototype moved actor handle");
        const initialMotion = object(initial, "motion");
        const outputMotion = object(output, "motion");
        const initialBody = object(initialMotion, "body");
        const outputBody = object(outputMotion, "body");
        if (
            number(outputBody, "halfHeightNumerator") !==
                number(initialBody, "halfHeightNumerator") ||
            number(outputMotion, "stepVelocityQ16") !== 0
        ) fail("prototype moved actor vertical state diverged");
        const initialPosition = object(initialBody, "position");
        const outputPosition = object(outputBody, "position");
        same(
            object(outputPosition, "region"),
            object(initialPosition, "region"),
            "prototype moved actor region",
        );
        if (
            number(outputPosition, "localXQ9") !==
                number(initialPosition, "localXQ9") + expected.deltaXQ9 * stepCount ||
            number(outputPosition, "localZQ9") !==
                number(initialPosition, "localZQ9") + expected.deltaZQ9 * stepCount
        ) fail("prototype moved actor horizontal displacement diverged");
    }
    const bootstrapFrames = number(driver, "bootstrapFrameCount");
    const liveFrames = number(driver, "liveFrameCount");
    if (
        bootstrapFrames !== number(object(readiness, "startup"), "readyFrameIndex") ||
        liveFrames < 1 || number(driver, "totalFrameCount") !== bootstrapFrames + liveFrames
    ) fail("prototype simulation readiness frame ordering diverged");
    const expectedOutputEpoch = expected.animationClip === 0
        ? inputEpoch
        : (inputEpoch + liveFrames - 1) % 31_002_560;
    if (outputEpoch !== expectedOutputEpoch) {
        fail("prototype simulation output animation epoch diverged");
    }
    return {
        revision: driver.revision,
        outcome: sample.outcome,
        command,
        presentation: {
            command: commandPresentation,
            input: inputPresentation,
            output: outputPresentation,
            inputEpoch,
            outputEpoch,
            startsAtLocalPhaseZero: true,
        },
        clockActive: true,
        boundedStepCount: true,
        renderBlockCount: 0,
        tickStartsAtZero: true,
        exactHorizontalDisplacement: true,
        groundedAfterBatch: expected.groundedAfterBatch,
        jumpMotion,
        queryPerStep: true,
        readinessAfterFrame: true,
    };
}

export async function prototypeHostGates(
    terrain: string,
    objects: string,
    corruptObjects: string,
    base: Coord,
): Promise<Json> {
    console.log("==> thin non-diagnostic prototype host gates");
    useSidecar(SIDECAR);
    await lifecycle("stop");
    await writeDocument(document(terrain, "out/cooked/bootstrap/missing.wlr", base));
    const missingSource = await failedStart("missing source");
    const corruptCenter: Coord = [base[0] + 70, base[1]];
    await writeDocument(document(terrain, corruptObjects, corruptCenter));
    const corruptPayload = await failedStart("corrupt payload");
    await writeDocument(document(terrain, objects, base));
    const first = await capturedReady("prototype first process");
    const restarted = await capturedReady("prototype restarted process");
    const objectActionActivated = await objectFeedbackSession(
        EXECUTABLE,
        CONFIG,
        "prototype invariant activated object action",
    );
    const sustained = await sustainedCapacitySession(EXECUTABLE, CONFIG);
    const objectActionCenter: Coord = [base[0] + 4, base[1]];
    await writeDocument(document(terrain, objects, objectActionCenter));
    const objectActionBaseline = await capturedReady(
        "prototype invariant object action baseline",
    );
    const objectActionRejected = await objectFeedbackSession(
        EXECUTABLE,
        CONFIG,
        "prototype invariant rejected object action",
    );
    await writeDocument(document(terrain, objects, base));
    const sessions = await sessionGates(
        EXECUTABLE,
        CONFIG,
        first,
        sustained,
        first,
        startupInvariant,
        (launch) => jumpPolicyInvariant(launch, true),
        objects,
        base,
    );
    const boundary = await boundarySurvival(EXECUTABLE, CONFIG);
    if (number(first, "processId") === number(restarted, "processId")) {
        fail("prototype evidence restart reused the process identity");
    }
    same(startupInvariant(restarted), startupInvariant(first), "prototype restart configuration");
    same(
        actorInvariant(restarted, base),
        actorInvariant(first, base),
        "prototype restart actor authority",
    );
    same(
        simulationDriverInvariant(restarted, STATIONARY_COMMAND),
        simulationDriverInvariant(first, STATIONARY_COMMAND),
        "prototype restart simulation driver",
    );
    same(
        cameraDriverInvariant(restarted),
        cameraDriverInvariant(first),
        "prototype restart camera driver",
    );
    same(
        jumpPolicyInvariant(restarted, true),
        jumpPolicyInvariant(first, true),
        "prototype restart jump policy",
    );
    restartObservation(restarted, first);
    const firstTraversal = traversalInvariant(first, base);
    same(
        traversalInvariant(restarted, base),
        firstTraversal,
        "prototype restart traversal activation",
    );
    const objectFeedbackInvariant = await objectFeedbackGates(
        objectActionActivated,
        objectActionRejected,
        first,
        objectActionBaseline,
        objects,
        base,
        objectActionCenter,
        startupInvariant,
        (launch) => simulationDriverInvariant(launch, STATIONARY_COMMAND),
        (launch) => simulationDriverInvariant(launch, STATIONARY_COMMAND),
    );
    same(startupInvariant(boundary), startupInvariant(first), "prototype boundary configuration");
    same(
        actorInvariant(boundary, base),
        actorInvariant(first, base),
        "prototype boundary initial actor authority",
    );
    const boundaryInvariant = {
        simulation: simulationDriverInvariant(boundary, STATIONARY_COMMAND),
        jump: jumpPolicyInvariant(boundary, true),
        camera: cameraDriverInvariant(boundary),
        traversal: traversalInvariant(boundary, base),
        minimumHoldMilliseconds: BOUNDARY_HOLD_MILLISECONDS,
        processRemainedLive: boundary.processRemainedLive,
        actionAfterReadiness: boundary.actionAfterReadiness,
    };

    await lifecycle("start");
    const firstSidecar = await sidecarStatus(SIDECAR);
    const firstPids = prototypePids(firstSidecar);
    if (firstPids.length === 0) fail("prototype Sidecar start retained no process");
    await lifecycle("restart");
    const restartedSidecar = await sidecarStatus(SIDECAR);
    const restartedPids = prototypePids(restartedSidecar);
    if (
        restartedPids.length === 0 ||
        JSON.stringify(restartedPids) === JSON.stringify(firstPids)
    ) fail("prototype Sidecar restart did not replace its process set");
    await lifecycle("stop");
    const stopped = await sidecarStatus(SIDECAR);
    if (prototypePids(stopped).length !== 0 || object(stopped, "runtime").running !== false) {
        fail("prototype Sidecar stop left an owned process");
    }
    useSidecar("sidecar.toml");

    return {
        configPath: CONFIG,
        missingSource,
        corruptPayload,
        first,
        restarted,
        sessions,
        objectActionBaseline,
        objectActionActivated,
        objectActionRejected,
        objectFeedbackInvariant,
        boundary,
        boundaryInvariant,
        sidecar: { first: firstSidecar, restarted: restartedSidecar, stopped },
    };
}
