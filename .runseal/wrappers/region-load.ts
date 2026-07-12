function fail(message: string): never {
    throw new Error(message);
}

async function sidecar(args: string[]): Promise<unknown> {
    const output = await new Deno.Command("sidecar", {
        args: [...args, "--config", "sidecar.toml", "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    const stderr = decoder.decode(output.stderr).trim();
    if (!output.success) {
        fail(`region-load: sidecar ${args[0]} failed${stderr ? `: ${stderr}` : ""}`);
    }
    if (!stdout) return null;
    try {
        return JSON.parse(stdout);
    } catch {
        fail(`region-load: sidecar returned non-JSON output: ${stdout}`);
    }
}

async function lifecycle(command: "start" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [command, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`region-load: sidecar ${command} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]) as Record<string, unknown>;
    if (response.ok !== true || typeof response.data !== "object" || response.data === null) {
        fail(`region-load: ${verb} returned an invalid response`);
    }
    return response.data as Record<string, unknown>;
}

function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) fail(`region-load: expected ${name} to be ${type}`);
    return value as T;
}

function object(owner: Record<string, unknown>, name: string): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`region-load: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) fail(`region-load: expected ${name} to be an array`);
    return value;
}

type Distribution = {
    minimum: number;
    median: number;
    p95: number;
    p99: number;
    maximum: number;
};

function distribution(values: number[]): Distribution {
    if (values.length === 0 || values.some((value) => !Number.isFinite(value) || value <= 0)) {
        fail("region-load: timing distribution contains invalid values");
    }
    const sorted = [...values].sort((left, right) => left - right);
    const percentile = (value: number) => sorted[Math.ceil(value * sorted.length) - 1];
    return {
        minimum: sorted[0],
        median: percentile(0.5),
        p95: percentile(0.95),
        p99: percentile(0.99),
        maximum: sorted.at(-1)!,
    };
}

async function configure(side: number): Promise<void> {
    const status = await event("load.configure", {
        world_region_side: side,
        active_center_x: 64,
        active_center_z: 64,
        active_radius: 2,
    });
    if (field<string>(status, "mode", "string") !== "region-load") {
        fail(`${side}: load mode was not enabled`);
    }
}

function validateProbe(
    probe: Record<string, unknown>,
    side: number,
    expectedVisible?: number,
): number {
    const logical = side * side * 1024;
    if (field<number>(probe, "logicalInstanceCount", "number") !== logical) {
        fail(`${side}: logical instance count mismatch`);
    }
    if (field<number>(probe, "activeRegionCount", "number") !== 25) {
        fail(`${side}: active region count mismatch`);
    }
    if (field<number>(probe, "candidateInstanceCount", "number") !== 25_600) {
        fail(`${side}: candidate count mismatch`);
    }
    if (field<number>(probe, "indirectDrawCount", "number") !== 1) {
        fail(`${side}: indirect draw count mismatch`);
    }
    if (field<number>(probe, "probeIterations", "number") !== 64) {
        fail(`${side}: probe iteration count mismatch`);
    }
    const dispatch = array(probe, "dispatch");
    if (JSON.stringify(dispatch) !== JSON.stringify([25, 4, 1])) {
        fail(`${side}: dispatch mismatch`);
    }
    const visible = field<number>(probe, "visibleInstanceCount", "number");
    if (visible <= 0 || visible >= 25_600) fail(`${side}: culling did not compact candidates`);
    if (expectedVisible !== undefined && visible !== expectedVisible) {
        fail(`${side}: visible count drifted`);
    }
    const compaction = field<number>(probe, "gpuCompactionMs", "number");
    const draw = field<number>(probe, "gpuDrawMs", "number");
    const total = field<number>(probe, "gpuTotalMs", "number");
    if (compaction <= 0 || draw <= 0 || total < compaction + draw) {
        fail(`${side}: GPU timestamps are invalid`);
    }
    return visible;
}

async function captureEvidence(side: number): Promise<Record<string, unknown>> {
    const color = await event("workbench.capture", {
        id: `color-${side}`,
        collection: "0005-gpu-region-compaction",
    });
    const ids = await event("perception.capture", {
        id: `ids-${side}`,
        collection: "0005-gpu-region-compaction",
        samples: [{ x: 0, y: 0 }, { x: 600, y: 600 }],
    });
    for (const manifest of [color, ids]) {
        if (manifest.lastError !== null) fail(`${side}: renderer error was reported`);
        if (object(manifest, "renderer").deviceRemovedReason !== null) {
            fail(`${side}: device removal was reported`);
        }
        const workload = object(manifest, "workload");
        if (field<string>(workload, "mode", "string") !== "region-load") {
            fail(`${side}: capture workload mismatch`);
        }
    }
    const perception = object(ids, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`${side}: unknown IDs were reported`);
    const samples = array(evidence, "samples") as Record<string, unknown>[];
    if (samples[0].id !== 0 || typeof samples[1].id !== "number" || samples[1].id === 0) {
        fail(`${side}: semantic samples mismatch`);
    }
    if (typeof samples[1].name !== "string" || !samples[1].name.startsWith("load.region.")) {
        fail(`${side}: visible sample did not resolve to a region proxy`);
    }
    const colorPixels = field<number>(object(color, "image"), "differentPixelCount", "number");
    const background = field<number>(
        object(evidence, "fullFrame"),
        "backgroundPixelCount",
        "number",
    );
    if (colorPixels !== 921_600 - background) {
        fail(`${side}: color and object-ID coverage differ`);
    }
    return {
        colorSha256: field<string>(object(color, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(perception, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(perception, "diagnosticPngSha256", "string"),
        visibleRegionCount: array(object(evidence, "fullFrame"), "objects").length,
        sampleId: samples[1].id,
        sampleName: samples[1].name,
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :region-load");
    console.log("");
    console.log("Run the canonical Experiment 0005 GPU region compaction workload.");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`region-load: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("region-load: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const reportPath = `${root}/out/captures/0005-gpu-region-compaction/acceptance.json`;
const worldSides = [32, 64, 128];
const warmupCount = 16;
const measuredCount = 64;
const results: Record<string, unknown>[] = [];
let expectedVisible: number | undefined;
let referenceEvidence: Record<string, unknown> | undefined;
let report: Record<string, unknown> | undefined;

await lifecycle("stop");
try {
    await Deno.remove(reportPath);
} catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) throw error;
}
try {
    await lifecycle("start");
    await event("workbench.pause");
    await event("camera.set_pose", {
        position: [0, 30, 30],
        target: [0, 0, 0],
        vertical_fov_degrees: 60,
    });
    for (const side of worldSides) {
        await configure(side);
        for (let index = 0; index < warmupCount; index++) {
            const warmup = await event("load.probe");
            expectedVisible = validateProbe(warmup, side, expectedVisible);
        }
        const compaction: number[] = [];
        const draw: number[] = [];
        const total: number[] = [];
        const roundTrip: number[] = [];
        for (let index = 0; index < measuredCount; index++) {
            const start = performance.now();
            const probe = await event("load.probe");
            roundTrip.push(performance.now() - start);
            expectedVisible = validateProbe(probe, side, expectedVisible);
            compaction.push(field<number>(probe, "gpuCompactionMs", "number"));
            draw.push(field<number>(probe, "gpuDrawMs", "number"));
            total.push(field<number>(probe, "gpuTotalMs", "number"));
        }
        const evidence = await captureEvidence(side);
        if (referenceEvidence) {
            for (const key of ["colorSha256", "rawIdSha256", "diagnosticPngSha256"]) {
                if (evidence[key] !== referenceEvidence[key]) {
                    fail(`${side}: ${key} changed with logical world size`);
                }
            }
        } else {
            referenceEvidence = evidence;
        }
        results.push({
            worldRegionSide: side,
            logicalInstanceCount: side * side * 1024,
            visibleInstanceCount: expectedVisible,
            gpuCompactionMs: distribution(compaction),
            gpuDrawMs: distribution(draw),
            gpuTotalMs: distribution(total),
            cpuRoundTripMs: distribution(roundTrip),
            evidence,
        });
    }

    const medians = results.map((result) =>
        field<number>(object(result, "gpuTotalMs"), "median", "number")
    );
    const fastest = Math.min(...medians);
    const slowest = Math.max(...medians);
    if (slowest > fastest * 1.35 + 0.02) {
        fail(
            `region-load: world-size timing drift is too large (${JSON.stringify(medians)} ms)`,
        );
    }

    await event("load.disable");
    await event("camera.reset");
    const calibrationColor = await event("workbench.capture", {
        id: "calibration-color",
        collection: "0005-gpu-region-compaction",
    });
    const calibrationIds = await event("perception.capture", {
        id: "calibration-ids",
        collection: "0005-gpu-region-compaction",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    const calibrationColorHash = field<string>(
        object(calibrationColor, "image"),
        "pixelSha256",
        "string",
    );
    const calibrationIdHash = field<string>(
        object(calibrationIds, "perception"),
        "rawSha256",
        "string",
    );
    if (
        calibrationColorHash !== "8f0fc6e9a49b95330921ff4f6b30c18bb4d16f2bd86f1b452a00f2ff00b68f6d"
    ) {
        fail("region-load: calibration color baseline changed");
    }
    if (calibrationIdHash !== "b132c850f0295119f9beba14d9287881a8bf1e8d51487800726de942f10e1da4") {
        fail("region-load: calibration object-ID baseline changed");
    }

    report = {
        schemaVersion: 1,
        outcome: "pass",
        loadRevision: "region-grid-v1",
        warmupCount,
        measuredCount,
        activeRegionCount: 25,
        candidateInstanceCount: 25_600,
        dispatch: [25, 4, 1],
        visibleInstanceCount: expectedVisible,
        worldSizeMedianRatio: slowest / fastest,
        results,
        calibration: {
            colorSha256: calibrationColorHash,
            rawIdSha256: calibrationIdHash,
        },
    };
} finally {
    await lifecycle("stop");
}

const status = await sidecar(["status"]) as Record<string, unknown>;
if (field<boolean>(object(status, "runtime"), "running", "boolean")) {
    fail("region-load: broker remains running");
}
for (const target of array(status, "targets")) {
    if (typeof target !== "object" || target === null || Array.isArray(target)) {
        fail("region-load: invalid status target");
    }
    if (field<boolean>(target as Record<string, unknown>, "running", "boolean")) {
        fail("region-load: a target remains running");
    }
}
if (!report) fail("region-load: acceptance report was not produced");
await Deno.writeTextFile(reportPath, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
