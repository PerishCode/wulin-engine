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
import {
    holdPrototypeForwardKey,
    pressPrototypeCameraClockwise,
    pressPrototypeJump,
} from "./input.ts";
import { JUMP_VELOCITY_DELTA_Q16, jumpMotionInvariant, jumpPolicyInvariant } from "./jump.ts";
import { actorInvariant } from "./actor.ts";
import { BOUNDARY_HOLD_MILLISECONDS, boundarySurvival } from "./boundary.ts";
import { cameraDriverInvariant } from "./camera.ts";
import { presentationInvariant } from "./presentation.ts";
import { escapeExit, readinessLine } from "./process.ts";
import { cameraOrbitTraversalInvariant, traversalInvariant } from "./traversal.ts";

const CONFIG = "out/cooked/bootstrap/runtime.json";
const SIDECAR = "sidecar.prototype.toml";
const EXECUTABLE = "target/debug/prototype.exe";
const decoder = new TextDecoder();

function document(terrain: string, objects: string, center: Coord): Json {
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

async function writeDocument(value: Json): Promise<void> {
    await Deno.mkdir(`${root}/out/cooked/bootstrap`, { recursive: true });
    await Deno.writeTextFile(`${root}/${CONFIG}`, `${JSON.stringify(value, null, 2)}\n`);
}

async function failedStart(label: string): Promise<Json> {
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
    if (stdout.includes('"role":"prototype"')) {
        fail(`prototype ${label} emitted readiness before failing`);
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

type StartupInput = "camera-clockwise" | "forward" | "jump";

async function capturedReady(label: string, startupInput?: StartupInput): Promise<Json> {
    const started = performance.now();
    const child = new Deno.Command(EXECUTABLE, {
        args: [`--bootstrap=${CONFIG}`],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).spawn();
    const stderr = new Response(child.stderr).text();
    const reader = child.stdout
        .pipeThrough(new TextDecoderStream())
        .getReader();
    let value: Json;
    let nativeInput: Json | null = null;
    try {
        if (startupInput === "forward") nativeInput = await holdPrototypeForwardKey(child.pid);
        if (startupInput === "camera-clockwise") {
            nativeInput = await pressPrototypeCameraClockwise(child.pid);
        }
        if (startupInput === "jump") nativeInput = await pressPrototypeJump(child.pid);
        const line = await readinessLine(reader);
        value = JSON.parse(line) as Json;
        if (value.role !== "prototype") fail(`${label} emitted the wrong readiness role`);
        const startup = object(value, "startup");
        if (
            startup.mode !== "canonical-bootstrap" ||
            number(startup, "readyFrameIndex") < 1 ||
            number(startup, "elapsedMs") <= 0
        ) fail(`${label} emitted incomplete canonical readiness`);
    } finally {
        await reader.cancel();
        child.kill();
    }
    const status = await child.status;
    return {
        label,
        processId: Number(string(value, "instance_id")),
        elapsedMs: performance.now() - started,
        forcedEvidenceExitCode: status.code,
        stderr: (await stderr).trim().slice(-4_096),
        nativeInput,
        readiness: value,
    };
}

function startupInvariant(launch: Json): Json {
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

type ExpectedCommand = {
    deltaXQ9: number;
    deltaZQ9: number;
    stepUpLimitQ16: number;
    initialVelocityDeltaQ16: number;
    groundedAfterBatch: boolean;
    animationClip: number;
    yawQ16: number;
};

const STATIONARY_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: 0,
    stepUpLimitQ16: 32_768,
    initialVelocityDeltaQ16: 0,
    groundedAfterBatch: true,
    animationClip: 0,
    yawQ16: 0,
};
const FORWARD_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: -32,
    stepUpLimitQ16: 32_768,
    initialVelocityDeltaQ16: 0,
    groundedAfterBatch: true,
    animationClip: 1,
    yawQ16: 49_152,
};
const JUMP_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: 0,
    stepUpLimitQ16: 32_768,
    initialVelocityDeltaQ16: JUMP_VELOCITY_DELTA_Q16,
    groundedAfterBatch: false,
    animationClip: 0,
    yawQ16: 0,
};

function simulationDriverInvariant(launch: Json, expected: ExpectedCommand): Json {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "simulation_driver");
    if (driver.revision !== "live-prototype-locomotion-driver-v7") {
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

async function sidecarStatus(): Promise<Json> {
    const output = await new Deno.Command("sidecar", {
        args: ["status", "--config", SIDECAR, "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail(`prototype Sidecar status failed with ${output.code}`);
    return JSON.parse(decoder.decode(output.stdout).trim()) as Json;
}

function prototypePids(status: Json): number[] {
    const targets = array(status, "targets");
    if (targets.length !== 1) fail("prototype Sidecar target count diverged");
    const target = targets[0] as Json;
    if (target.name !== "prototype") fail("prototype Sidecar target identity diverged");
    return array(target, "pids").map((value) => {
        if (typeof value !== "number") fail("prototype Sidecar PID must be numeric");
        return value;
    });
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
    const invalid = document(terrain, objects, base);
    invalid.fallback = true;
    await writeDocument(invalid);
    const invalidDocument = await failedStart("invalid document");

    await writeDocument(document(terrain, "out/cooked/bootstrap/missing.wlr", base));
    const missingSource = await failedStart("missing source");

    const corruptCenter: Coord = [base[0] + 70, base[1]];
    await writeDocument(document(terrain, corruptObjects, corruptCenter));
    const corruptPayload = await failedStart("corrupt payload");

    await writeDocument(document(terrain, objects, base));
    const first = await capturedReady("prototype first process");
    const restarted = await capturedReady("prototype restarted process");
    const forward = await capturedReady("prototype forward locomotion", "forward");
    const cameraOrbit = await capturedReady("prototype clockwise camera orbit", "camera-clockwise");
    const jump = await capturedReady("prototype committed jump", "jump");
    const escape = await escapeExit(EXECUTABLE, CONFIG, "prototype Escape press exit");
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
    const firstTraversal = traversalInvariant(first, base);
    same(
        traversalInvariant(restarted, base),
        firstTraversal,
        "prototype restart traversal activation",
    );
    same(startupInvariant(forward), startupInvariant(first), "prototype locomotion configuration");
    same(startupInvariant(escape), startupInvariant(first), "prototype Escape configuration");
    same(
        jumpPolicyInvariant(escape, true),
        jumpPolicyInvariant(first, true),
        "prototype Escape jump policy",
    );
    same(
        actorInvariant(forward, base),
        actorInvariant(first, base),
        "prototype locomotion initial actor authority",
    );
    const forwardTraversal = traversalInvariant(forward, base);
    same(forwardTraversal, firstTraversal, "prototype locomotion traversal activation");
    const forwardInvariant = {
        simulation: simulationDriverInvariant(forward, FORWARD_COMMAND),
        jump: jumpPolicyInvariant(forward, true),
        camera: cameraDriverInvariant(forward),
        traversal: forwardTraversal,
    };
    same(
        startupInvariant(cameraOrbit),
        startupInvariant(first),
        "prototype camera-orbit configuration",
    );
    same(
        actorInvariant(cameraOrbit, base),
        actorInvariant(first, base),
        "prototype camera-orbit initial actor authority",
    );
    const cameraOrbitInvariant = {
        simulation: simulationDriverInvariant(cameraOrbit, STATIONARY_COMMAND),
        jump: jumpPolicyInvariant(cameraOrbit, true),
        camera: cameraDriverInvariant(cameraOrbit, 1),
        traversal: cameraOrbitTraversalInvariant(cameraOrbit, base),
    };
    same(startupInvariant(jump), startupInvariant(first), "prototype jump configuration");
    same(
        actorInvariant(jump, base),
        actorInvariant(first, base),
        "prototype jump initial actor authority",
    );
    const jumpInvariant = {
        simulation: simulationDriverInvariant(jump, JUMP_COMMAND),
        jump: jumpPolicyInvariant(jump, false),
        camera: cameraDriverInvariant(jump),
        traversal: traversalInvariant(jump, base),
    };
    same(startupInvariant(boundary), startupInvariant(first), "prototype boundary configuration");
    same(
        actorInvariant(boundary, base),
        actorInvariant(first, base),
        "prototype boundary initial actor authority",
    );
    const boundaryInvariant = {
        simulation: simulationDriverInvariant(boundary, FORWARD_COMMAND),
        jump: jumpPolicyInvariant(boundary, true),
        camera: cameraDriverInvariant(boundary),
        traversal: traversalInvariant(boundary, base),
        minimumHoldMilliseconds: BOUNDARY_HOLD_MILLISECONDS,
        processRemainedLive: boundary.processRemainedLive,
    };

    await lifecycle("start");
    const firstSidecar = await sidecarStatus();
    const firstPids = prototypePids(firstSidecar);
    if (firstPids.length === 0) fail("prototype Sidecar start retained no process");
    await lifecycle("restart");
    const restartedSidecar = await sidecarStatus();
    const restartedPids = prototypePids(restartedSidecar);
    if (
        restartedPids.length === 0 ||
        JSON.stringify(restartedPids) === JSON.stringify(firstPids)
    ) fail("prototype Sidecar restart did not replace its process set");
    await lifecycle("stop");
    const stopped = await sidecarStatus();
    if (prototypePids(stopped).length !== 0 || object(stopped, "runtime").running !== false) {
        fail("prototype Sidecar stop left an owned process");
    }
    useSidecar("sidecar.toml");

    return {
        configPath: CONFIG,
        invalidDocument,
        missingSource,
        corruptPayload,
        first,
        restarted,
        forward,
        escape,
        forwardInvariant,
        cameraOrbit,
        cameraOrbitInvariant,
        jump,
        jumpInvariant,
        boundary,
        boundaryInvariant,
        sidecar: { first: firstSidecar, restarted: restartedSidecar, stopped },
    };
}
