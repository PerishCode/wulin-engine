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
import { holdPrototypeForwardKey } from "./input.ts";
import { presentationInvariant } from "./presentation.ts";
import { traversalInvariant } from "./traversal.ts";

const CONFIG = "out/cooked/bootstrap/runtime.json";
const SIDECAR = "sidecar.prototype.toml";
const EXECUTABLE = "target/debug/prototype.exe";
const decoder = new TextDecoder();

function document(terrain: string, objects: string, center: Coord): Json {
    return {
        schemaVersion: 1,
        terrain,
        objects,
        globalOrigin: { x: center[0], z: center[1] },
        globalCenter: { x: center[0], z: center[1] },
        activeRadius: 2,
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

async function readinessLine(reader: ReadableStreamDefaultReader<string>): Promise<string> {
    const deadline = performance.now() + 30_000;
    let buffered = "";
    while (performance.now() < deadline) {
        const remaining = Math.max(1, deadline - performance.now());
        const result = await Promise.race([
            reader.read(),
            new Promise<{ done: true; value: undefined }>((resolve) =>
                setTimeout(() => resolve({ done: true, value: undefined }), remaining)
            ),
        ]);
        if (result.done) {
            if (buffered.trim()) return buffered.trim();
            fail("prototype exited or timed out before readiness");
        }
        buffered += result.value;
        const newline = buffered.indexOf("\n");
        if (newline >= 0) return buffered.slice(0, newline).trim();
    }
    fail("prototype readiness timeout expired");
}

async function capturedReady(label: string, holdForward = false): Promise<Json> {
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
        if (holdForward) nativeInput = await holdPrototypeForwardKey(child.pid);
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
    };
}

function actorInvariant(launch: Json, center: Coord): Json {
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
        initialAtCenter: true,
        currentMatchesAdvance: true,
    };
}

type ExpectedCommand = {
    deltaXQ9: number;
    deltaZQ9: number;
    stepUpLimitQ16: number;
    animationClip: number;
    yawQ16: number;
};

const STATIONARY_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: 0,
    stepUpLimitQ16: 32_768,
    animationClip: 0,
    yawQ16: 0,
};
const FORWARD_COMMAND: ExpectedCommand = {
    deltaXQ9: 0,
    deltaZQ9: -32,
    stepUpLimitQ16: 32_768,
    animationClip: 1,
    yawQ16: 49_152,
};

function simulationDriverInvariant(launch: Json, expected: ExpectedCommand): Json {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "simulation_driver");
    if (driver.revision !== "live-prototype-locomotion-driver-v2") {
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
        number(command, "stepUpLimitQ16") !== expected.stepUpLimitQ16
    ) fail("prototype simulation locomotion command diverged");
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
        number(actor, "terrainQueryCount") !== stepCount
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
    if (expected.deltaXQ9 === 0 && expected.deltaZQ9 === 0) {
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
    return {
        revision: driver.revision,
        outcome: sample.outcome,
        command,
        presentation: {
            command: commandPresentation,
            input: inputPresentation,
            output: outputPresentation,
        },
        clockActive: true,
        boundedStepCount: true,
        renderBlockCount: 0,
        tickStartsAtZero: true,
        exactHorizontalDisplacement: true,
        groundedAfterBatch: true,
        queryPerStep: true,
        readinessAfterFrame: true,
    };
}

function numericArray(value: Json, key: string): number[] {
    return array(value, key).map((entry) => {
        if (typeof entry !== "number" || !Number.isFinite(entry)) {
            fail(`prototype camera ${key} must contain finite numbers`);
        }
        return entry;
    });
}

function cameraDriverInvariant(launch: Json): Json {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "camera_driver");
    if (driver.revision !== "live-prototype-actor-camera-v1") {
        fail("prototype camera driver revision diverged");
    }
    const actor = object(object(readiness, "actor"), "state");
    same(object(driver, "actor"), object(actor, "handle"), "prototype camera actor handle");
    const rig = object(driver, "rig");
    same(numericArray(rig, "positionOffset"), [9, 4, 12], "prototype camera position rig");
    same(numericArray(rig, "targetOffset"), [0, -1, -3], "prototype camera target rig");
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
        [actorX + 9, anchorY + 4, actorZ + 12],
        "prototype anchored camera position",
    );
    same(
        numericArray(camera, "target"),
        [actorX, anchorY - 1, actorZ - 3],
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
        rig,
        camera,
        anchorPerLiveFrame: true,
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
    const forward = await capturedReady("prototype forward locomotion", true);
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
    const firstTraversal = traversalInvariant(first, base);
    same(
        traversalInvariant(restarted, base),
        firstTraversal,
        "prototype restart traversal activation",
    );
    same(startupInvariant(forward), startupInvariant(first), "prototype locomotion configuration");
    same(
        actorInvariant(forward, base),
        actorInvariant(first, base),
        "prototype locomotion initial actor authority",
    );
    const forwardTraversal = traversalInvariant(forward, base);
    same(forwardTraversal, firstTraversal, "prototype locomotion traversal activation");
    const forwardInvariant = {
        simulation: simulationDriverInvariant(forward, FORWARD_COMMAND),
        camera: cameraDriverInvariant(forward),
        traversal: forwardTraversal,
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
        forwardInvariant,
        sidecar: { first: firstSidecar, restarted: restartedSidecar, stopped },
    };
}
