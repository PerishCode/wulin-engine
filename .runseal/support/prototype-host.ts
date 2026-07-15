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
} from "./canonical-runtime.ts";

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

async function capturedReady(label: string): Promise<Json> {
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
    try {
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
    const actorAuthority = object(object(launch, "readiness"), "actor");
    if (number(actorAuthority, "capacity") !== 1 || number(actorAuthority, "liveCount") !== 1) {
        fail("prototype readiness actor cardinality diverged");
    }
    const terrain = object(actorAuthority, "terrain");
    const actor = object(actorAuthority, "state");
    if (number(object(actor, "handle"), "generation") !== 1) {
        fail("prototype initial actor generation diverged");
    }
    const presentation = object(actor, "presentation");
    if (
        number(presentation, "archetype") !== 7 || number(presentation, "material") !== 63 ||
        number(presentation, "yawQ16") !== 0 || number(presentation, "animation") !== 1
    ) fail("prototype initial actor presentation diverged");
    const motion = object(actor, "motion");
    const body = object(motion, "body");
    const position = object(body, "position");
    const region = object(position, "region");
    if (
        number(region, "x") !== center[0] || number(region, "z") !== center[1] ||
        number(position, "localXQ9") !== 0 || number(position, "localZQ9") !== 0
    ) fail("prototype initial actor position diverged");
    const halfHeight = number(body, "halfHeightNumerator");
    const terrainHeight = number(terrain, "heightNumerator");
    if (
        number(terrain, "heightDenominator") !== 65_536 || halfHeight !== 65_536 ||
        number(body, "centerHeightNumerator") !== terrainHeight + halfHeight ||
        number(motion, "stepVelocityQ16") !== 0
    ) fail("prototype initial actor grounding diverged");
    return actorAuthority;
}

function simulationDriverInvariant(launch: Json): Json {
    const readiness = object(launch, "readiness");
    const driver = object(readiness, "simulation_driver");
    if (driver.revision !== "live-prototype-actor-driver-v1") {
        fail("prototype simulation driver revision diverged");
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
    for (
        const field of ["deltaXQ9", "deltaZQ9", "stepUpLimitQ16", "stepAccelerationQ16"]
    ) {
        if (number(command, field) !== 0) fail(`prototype simulation command ${field} diverged`);
    }
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
    const initial = object(object(readiness, "actor"), "state");
    same(object(actor, "input"), initial, "prototype live actor input");
    same(object(actor, "output"), initial, "prototype live actor output");
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
        clockActive: true,
        boundedStepCount: true,
        tickStartsAtZero: true,
        actorStable: true,
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
        simulationDriverInvariant(restarted),
        simulationDriverInvariant(first),
        "prototype restart simulation driver",
    );

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
        sidecar: { first: firstSidecar, restarted: restartedSidecar, stopped },
    };
}
