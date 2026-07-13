import {
    array,
    CANONICAL_MAPPING,
    CANONICAL_PAYLOAD,
    captureEvidence,
    collectEnvironment,
    distribution,
    fail,
    field,
    loadConfig,
    object,
    same,
    stableProbe,
    TERRAIN_REVISION,
    validateProbe,
} from "../support/terrain.ts";

async function sidecar(args: string[]): Promise<Record<string, unknown>> {
    for (let attempt = 0; attempt < 3; attempt += 1) {
        const output = await new Deno.Command("sidecar", {
            args: [...args, "--config", sidecarConfig, "--format", "json"],
            cwd: root,
            stdout: "piped",
            stderr: "piped",
        }).output();
        const stdout = decoder.decode(output.stdout).trim();
        if (stdout) {
            try {
                const value = JSON.parse(stdout) as Record<string, unknown>;
                if (!output.success || value.ok === false) {
                    fail(`sidecar ${args[0]} failed: ${JSON.stringify(value.error)}`);
                }
                return value;
            } catch (error) {
                if (attempt === 2) fail(`sidecar returned invalid JSON: ${error}: ${stdout}`);
            }
        }
        await new Promise((resolve) => setTimeout(resolve, 50));
    }
    fail("sidecar retry loop exhausted");
}

async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [verb, "--config", sidecarConfig],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`sidecar ${verb} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    return object(await sidecar(["inspect", "workbench", verb, JSON.stringify(payload)]), "data");
}

async function cook(path: string): Promise<Record<string, unknown>> {
    const output = await new Deno.Command("cargo", {
        args: ["run", "--locked", "--release", "-p", "terrain-cooker", "--", path],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("terrain cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim());
}

async function waitPublished(
    expected: Record<string, number>,
    minimumGeneration = 0,
): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const renderer = object(status, "renderer");
        const published = renderer.published;
        if (published && typeof published === "object") {
            const value = published as Record<string, unknown>;
            const config = object(value, "config");
            const stream = object(status, "stream");
            const transfer = object(renderer, "transfer");
            if (
                stream.pending === null && transfer.copyPending === null &&
                field<number>(value, "generation", "number") >= minimumGeneration &&
                config.worldRegionSide === expected.world_region_side &&
                config.activeCenterX === expected.active_center_x &&
                config.activeCenterZ === expected.active_center_z &&
                config.activeRadius === expected.active_radius
            ) return status;
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail("terrain publication timed out");
}

async function publish(workload: Record<string, number>): Promise<Record<string, unknown>> {
    const scheduled = await event("terrain.schedule", workload);
    await event("workbench.resume");
    const status = await waitPublished(workload);
    await event("workbench.pause");
    return { scheduled, published: object(object(status, "renderer"), "published") };
}

async function prepare(path: string): Promise<void> {
    await event("terrain.open", { path });
    await publish(loadConfig());
    await event("terrain.enable");
    await event("camera.reset");
    await event("workbench.pause");
}

async function probe(): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateProbe(value);
    return value;
}

async function capture(id: string): Promise<Record<string, unknown>> {
    return captureEvidence(
        await event("perception.capture", {
            id,
            collection: "0013-gpu-streamed-terrain",
            samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
        }),
    );
}

async function holdStage(kind: "io" | "copy", destination: Record<string, number>) {
    const before = await capture(`${kind}-before`);
    await event(`terrain.${kind}_gate.arm`);
    await event("terrain.schedule", destination);
    await event("workbench.resume");
    const deadline = Date.now() + 5_000;
    let heldStatus: Record<string, unknown> | undefined;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const renderer = object(status, "renderer");
        const stream = object(status, "stream");
        const reached = kind === "io"
            ? stream.pending && object(stream, "pending").stage === "reading"
            : object(renderer, "transfer").copyPending !== null;
        if (reached) {
            heldStatus = status;
            break;
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    if (!heldStatus) fail(`${kind} hold did not reach the expected stage`);
    await event("workbench.pause");
    const held = await capture(`${kind}-held`);
    same(held, before, `${kind} held frame`);
    await event(`terrain.${kind}_gate.release`);
    await event("workbench.resume");
    const published = await waitPublished(destination);
    await event("workbench.pause");
    return { before, held, heldStatus, published };
}

async function waitFailure(): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const stream = object(status, "stream");
        if (stream.pending === null && stream.lastFailure) return status;
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail("terrain corruption failure timed out");
}

async function measuredProbes(): Promise<Record<string, unknown>> {
    await event("workbench.resume");
    await new Promise((resolve) => setTimeout(resolve, WARMUP_MS));
    await event("workbench.pause");
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) samples.push(await probe());
    const evidence = stableProbe(samples[0]);
    for (const sample of samples.slice(1)) {
        same(stableProbe(sample), evidence, "terrain timing evidence");
    }
    const timing = (name: string) =>
        distribution(
            samples.map((sample) => field<number>(object(sample, "timing"), name, "number")),
        );
    return {
        sampleCount: samples.length,
        warmupMs: WARMUP_MS,
        evidence,
        seamMs: timing("seamMs"),
        rasterMs: timing("rasterMs"),
        totalMs: timing("totalMs"),
    };
}

function nonnegativeDistribution(values: number[]): Record<string, number> {
    if (values.some((value) => !Number.isFinite(value) || value < 0)) {
        fail("invalid nonnegative terrain timing sample");
    }
    const sorted = [...values].sort((left, right) => left - right);
    const at = (fraction: number) => sorted[Math.ceil(fraction * sorted.length) - 1];
    return {
        minimum: sorted[0],
        median: at(0.5),
        p95: at(0.95),
        p99: at(0.99),
        maximum: sorted.at(-1)!,
    };
}

function publishedTransaction(result: Record<string, unknown>): Record<string, unknown> {
    return object(object(result, "published"), "transaction");
}

function validateTransaction(
    report: Record<string, unknown>,
    expected: Record<string, number>,
): void {
    for (const [name, value] of Object.entries(expected)) {
        if (field<number>(report, name, "number") !== value) {
            fail(`terrain transaction ${name} mismatch`);
        }
    }
    const schedule = field<number>(report, "scheduleMs", "number");
    const copy = field<number>(report, "copyGpuMs", "number");
    const publication = field<number>(report, "copyToPublicationMs", "number");
    const pending = field<number>(report, "pendingMs", "number");
    if (schedule <= 0 || copy < 0 || publication <= 0 || pending <= 0) {
        fail("terrain transaction timing is invalid");
    }
    if (Math.abs(schedule + publication - pending) > 0.001) {
        fail("terrain transaction stage timings do not cover total pending time");
    }
}

function transactionDistributions(
    samples: Record<string, unknown>[],
): Record<string, unknown> {
    const uploadedSha256 = field<string>(samples[0], "uploadedSha256", "string");
    for (const sample of samples.slice(1)) {
        if (sample.uploadedSha256 !== uploadedSha256) {
            fail("terrain streaming sample upload hash changed");
        }
    }
    const timing = (owner: Record<string, unknown>, name: string) =>
        field<number>(owner, name, "number");
    const values = (name: string) => samples.map((sample) => timing(sample, name));
    const ioValues = (name: string) => samples.map((sample) => timing(object(sample, "io"), name));
    return {
        sampleCount: samples.length,
        uploadedSha256,
        scheduleMs: distribution(values("scheduleMs")),
        copyGpuMs: nonnegativeDistribution(values("copyGpuMs")),
        copyToPublicationMs: distribution(values("copyToPublicationMs")),
        pendingMs: distribution(values("pendingMs")),
        io: {
            payloadBytes: [...new Set(ioValues("payloadBytes"))],
            readMs: nonnegativeDistribution(ioValues("readMs")),
            verifyMs: nonnegativeDistribution(ioValues("verifyMs")),
            totalMs: nonnegativeDistribution(ioValues("totalMs")),
        },
    };
}

async function measuredStreaming(path: string): Promise<Record<string, unknown>> {
    const cached: Record<string, unknown>[] = [];
    for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        const report = publishedTransaction(await publish(loadConfig()));
        validateTransaction(report, {
            retainedRegionCount: 25,
            uploadedRegionCount: 0,
            evictedRegionCount: 0,
            residentRegionCount: 25,
            payloadBytes: 0,
        });
        cached.push(report);
    }

    const cold: Record<string, unknown>[] = [];
    let coldProbe: Record<string, unknown> | undefined;
    for (let index = 0; index < SAMPLE_COUNT; index += 1) {
        await lifecycle("restart");
        await prepare(path);
        const report = publishedTransaction(await publish(loadConfig(96, 96, 2)));
        validateTransaction(report, {
            retainedRegionCount: 0,
            uploadedRegionCount: 25,
            evictedRegionCount: 0,
            residentRegionCount: 50,
            payloadBytes: 102_400,
        });
        if (field<number>(report, "copyGpuMs", "number") <= 0) {
            fail("cold terrain copy timestamp did not advance");
        }
        const evidence = stableProbe(await probe());
        if (coldProbe) same(evidence, coldProbe, "cold terrain restart evidence");
        else coldProbe = evidence;
        cold.push(report);
    }
    return {
        cachedRevisit: transactionDistributions(cached),
        coldTeleport: {
            ...transactionDistributions(cold),
            evidence: coldProbe,
        },
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :terrain");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);
const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const output = "out/terrain/0013-gpu-streamed-terrain/terrain.wlt";
const corruptPath = "out/terrain/0013-gpu-streamed-terrain/terrain-corrupt.wlt";
const reportPath = `${root}/out/captures/0013-gpu-streamed-terrain/acceptance.json`;
const SAMPLE_COUNT = 32;
const WARMUP_MS = 250;
let sidecarConfig = "sidecar.toml";
let finalReport: Record<string, unknown> | undefined;

await lifecycle("stop");
const cooked = await cook(output);
const environment = await collectEnvironment(root);
try {
    await lifecycle("start");
    await prepare(output);
    const status = await event("workbench.status");
    const renderer = object(status, "renderer");
    const firstProcess = field<number>(status, "processId", "number");
    if (renderer.debugLayer !== true || renderer.deviceRemovedReason !== null) {
        fail("debug terrain capability gate failed");
    }
    const terrainStatus = await event("terrain.status");
    const transfer = object(object(terrainStatus, "renderer"), "transfer");
    if (
        transfer.defaultHeapAllocationBytes !== 3_276_800 ||
        transfer.payloadArenaBytes !== 204_800 ||
        transfer.copyTimestampBytes !== 16
    ) fail("terrain allocation contract mismatch");
    const canonicalProbe = await probe();
    if (
        canonicalProbe.activeMappingSha256 !== CANONICAL_MAPPING ||
        canonicalProbe.payloadSha256 !== CANONICAL_PAYLOAD
    ) fail("canonical terrain mapping or payload hash mismatch");
    const canonicalCapture = await capture("canonical");

    const cameras = [
        { name: "x-boundary", position: [8, 5, 18], target: [8, 0, -6] },
        { name: "z-boundary", position: [18, 5, 8], target: [-6, 0, 8] },
        { name: "four-corner", position: [16, 8, 20], target: [8, 0, 8] },
    ];
    const boundaries = [];
    for (const camera of cameras) {
        await event("camera.set_pose", {
            position: camera.position,
            target: camera.target,
            vertical_fov_degrees: 60,
        });
        boundaries.push({
            camera,
            probe: stableProbe(await probe()),
            capture: await capture(camera.name),
        });
    }
    await event("camera.reset");

    const radii = [];
    for (const radius of [0, 1, 2]) {
        const transaction = await publish(loadConfig(64, 64, radius));
        radii.push({ radius, transaction, probe: stableProbe(await probe()) });
    }
    const movement = [];
    for (const [x, z] of [[65, 64], [65, 65], [64, 64]]) {
        const transaction = await publish(loadConfig(x, z, 2));
        movement.push({ x, z, transaction, probe: stableProbe(await probe()) });
    }
    same(stableProbe(await probe()), stableProbe(canonicalProbe), "cached terrain revisit");

    const ioHold = await holdStage("io", loadConfig(65, 64, 2));
    const copyHold = await holdStage("copy", loadConfig(65, 65, 2));

    await Deno.copyFile(output, corruptPath);
    await event("terrain.open", { path: corruptPath });
    const beforeFailure = await capture("corruption-before");
    const bytes = await Deno.readFile(corruptPath);
    bytes[bytes.length - 1] ^= 1;
    await Deno.writeFile(corruptPath, bytes);
    await event("terrain.schedule", loadConfig(96, 96, 2));
    await event("workbench.resume");
    const failure = await waitFailure();
    await event("workbench.pause");
    const heldFailure = await capture("corruption-held");
    same(heldFailure, beforeFailure, "corrupt terrain rollback frame");
    bytes[bytes.length - 1] ^= 1;
    await Deno.writeFile(corruptPath, bytes);
    const retry = await publish(loadConfig(96, 96, 2));
    const retryProbe = await probe();

    await lifecycle("restart");
    await prepare(output);
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("terrain workbench process survived restart");
    const restartProbe = await probe();
    const restartCapture = await capture("restart");
    same(stableProbe(restartProbe), stableProbe(canonicalProbe), "terrain restart probe");
    same(restartCapture, canonicalCapture, "terrain restart capture");

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await prepare(output);
    const benchmarkStatus = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkStatus, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release terrain capability gate failed");
    }
    const benchmark = await measuredProbes();
    benchmark.streaming = await measuredStreaming(output);
    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: TERRAIN_REVISION,
        environment,
        cooked,
        capability: { correctness: renderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        resources: transfer,
        canonical: { probe: canonicalProbe, capture: canonicalCapture },
        boundaries,
        sweeps: { radii },
        movement,
        holds: { io: ioHold, copy: copyHold },
        corruption: { failure, beforeFailure, heldFailure, retry, retryProbe },
        restart: { probe: restartProbe, capture: restartCapture },
        benchmark,
    };
} finally {
    await lifecycle("stop");
}

for (const config of ["sidecar.toml", "sidecar.benchmark.toml"]) {
    sidecarConfig = config;
    const cleanup = await sidecar(["status"]);
    if (field<boolean>(object(cleanup, "runtime"), "running", "boolean")) {
        fail(`${config} broker remains running`);
    }
    for (const target of array(cleanup, "targets") as Record<string, unknown>[]) {
        if (field<boolean>(target, "running", "boolean")) fail(`${config} target remains running`);
    }
}
if (!finalReport) fail("no terrain acceptance report was produced");
await Deno.mkdir(reportPath.replace(/[\\/][^\\/]+$/, ""), { recursive: true });
await Deno.writeTextFile(reportPath, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify({
    outcome: finalReport.outcome,
    report: "out/captures/0013-gpu-streamed-terrain/acceptance.json",
}));
