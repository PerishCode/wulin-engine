function fail(message: string): never {
    throw new Error(message);
}

function field<T>(owner: Record<string, unknown>, name: string, type: string): T {
    const value = owner[name];
    if (typeof value !== type) fail(`meshlet-scene: expected ${name} to be ${type}`);
    return value as T;
}

function object(owner: Record<string, unknown>, name: string): Record<string, unknown> {
    const value = owner[name];
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        fail(`meshlet-scene: expected ${name} to be an object`);
    }
    return value as Record<string, unknown>;
}

function array(owner: Record<string, unknown>, name: string): unknown[] {
    const value = owner[name];
    if (!Array.isArray(value)) fail(`meshlet-scene: expected ${name} to be an array`);
    return value;
}

function same(actual: unknown, expected: unknown, label: string): void {
    if (JSON.stringify(canonical(actual)) !== JSON.stringify(canonical(expected))) {
        fail(
            `meshlet-scene: ${label} mismatch: actual=${JSON.stringify(actual)} expected=${
                JSON.stringify(expected)
            }`,
        );
    }
}

function canonical(value: unknown): unknown {
    if (Array.isArray(value)) return value.map(canonical);
    if (typeof value !== "object" || value === null) return value;
    const owner = value as Record<string, unknown>;
    return Object.fromEntries(
        Object.keys(owner).sort().map((key) => [key, canonical(owner[key])]),
    );
}

function config(
    worldRegionSide = 128,
    activeRadius = 2,
    x = 64,
    z = 64,
): Record<string, number> {
    return {
        world_region_side: worldRegionSide,
        active_center_x: x,
        active_center_z: z,
        active_radius: activeRadius,
    };
}

async function sidecar(
    args: string[],
): Promise<{ success: boolean; value: Record<string, unknown> }> {
    const output = await new Deno.Command("sidecar", {
        args: [...args, "--config", "sidecar.toml", "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    if (!stdout) fail(`meshlet-scene: sidecar ${args[0]} returned no JSON`);
    try {
        return { success: output.success, value: JSON.parse(stdout) };
    } catch {
        fail(`meshlet-scene: sidecar returned invalid JSON: ${stdout}`);
    }
}

async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [verb, "--config", "sidecar.toml"],
        cwd: root,
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`meshlet-scene: sidecar ${verb} failed`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    const response = await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
    ]);
    if (!response.success || response.value.ok !== true) {
        fail(`meshlet-scene: ${verb} failed: ${String(response.value.error)}`);
    }
    return object(response.value, "data");
}

async function cook(path: string): Promise<Record<string, unknown>> {
    const output = await new Deno.Command("cargo", {
        args: ["run", "--locked", "--release", "-p", "region-cooker", "--", path],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("meshlet-scene: region cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim());
}

async function waitPublished(expected: Record<string, number>): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline) {
        const status = await event("async.status");
        if (status.pending === null && typeof status.published === "object" && status.published) {
            const published = status.published as Record<string, unknown>;
            if (
                published.worldRegionSide === expected.world_region_side &&
                published.activeCenterX === expected.active_center_x &&
                published.activeCenterZ === expected.active_center_z &&
                published.activeRadius === expected.active_radius
            ) return status;
        }
        await new Promise((resolve) => setTimeout(resolve, 10));
    }
    fail("meshlet-scene: cooked publication timed out");
}

async function publish(workload: Record<string, number>): Promise<Record<string, unknown>> {
    const report = await event("cooked.schedule", workload);
    await event("workbench.resume");
    const status = await waitPublished(workload);
    await event("workbench.pause");
    report.publication = object(status, "lastCompleted");
    return report;
}

async function followCamera(x = 64, z = 64): Promise<void> {
    const worldX = (x - 64) * 16;
    const worldZ = (z - 64) * 16;
    await event("camera.set_pose", {
        position: [worldX, 30, worldZ + 30],
        target: [worldX, 0, worldZ],
        vertical_fov_degrees: 60,
    });
}

async function configureMeshlets(mask = 255, forcedLod: number | null = null): Promise<void> {
    await event("meshlet.configure", { archetype_mask: mask, forced_lod: forcedLod });
}

function validateProbe(probe: Record<string, unknown>): void {
    if (
        field<string>(probe, "revision", "string") !== "gpu-meshlet-scene-v1" ||
        field<number>(probe, "resetDispatchCount", "number") !== 1 ||
        field<number>(probe, "cullDispatchCount", "number") !== 1 ||
        field<number>(probe, "indirectMeshDispatchCount", "number") !== 1 ||
        field<number>(probe, "probeIterations", "number") !== 16 ||
        field<string>(probe, "catalogSha256", "string") !== CATALOG_SHA256
    ) fail("meshlet-scene: probe execution contract mismatch");
    const gpu = object(probe, "gpu");
    same(gpu, object(probe, "cpuOracle"), "GPU/CPU oracle");
    const visible = field<number>(gpu, "visible", "number");
    const rejected = field<number>(gpu, "rejected", "number");
    const candidates = field<number>(probe, "candidateInstanceCount", "number");
    if (
        visible + rejected !== candidates ||
        (array(gpu, "lodCounts") as number[]).reduce((sum, value) => sum + value, 0) !== visible ||
        field<number>(gpu, "meshlets", "number") <= visible ||
        field<number>(gpu, "emittedVertices", "number") <= visible ||
        field<number>(gpu, "emittedTriangles", "number") <= visible
    ) fail("meshlet-scene: probe aggregate bounds mismatch");
}

async function probe(): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateProbe(value);
    return value;
}

function distribution(values: number[]): Record<string, number> {
    if (values.some((value) => !Number.isFinite(value) || value <= 0)) {
        fail("meshlet-scene: invalid GPU timing sample");
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

async function measuredProbes(): Promise<Record<string, unknown>> {
    const samples: Record<string, unknown>[] = [];
    for (let index = 0; index < 8; index += 1) samples.push(await probe());
    const baseline = object(samples[0], "gpu");
    for (const sample of samples.slice(1)) same(object(sample, "gpu"), baseline, "probe counts");
    return {
        sampleCount: samples.length,
        probeIterations: field<number>(samples[0], "probeIterations", "number"),
        counts: baseline,
        gpuCullLodMs: distribution(
            samples.map((sample) => field(sample, "gpuCullLodMs", "number")),
        ),
        gpuMeshMs: distribution(samples.map((sample) => field(sample, "gpuMeshMs", "number"))),
        gpuTotalMs: distribution(samples.map((sample) => field(sample, "gpuTotalMs", "number"))),
    };
}

async function captureEvidence(id: string): Promise<Record<string, unknown>> {
    const color = await event("workbench.capture", {
        id: `${id}-color`,
        collection: "0009-gpu-meshlet-scene",
    });
    const ids = await event("perception.capture", {
        id: `${id}-ids`,
        collection: "0009-gpu-meshlet-scene",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }, { x: 600, y: 600 }],
    });
    for (const manifest of [color, ids]) {
        if (
            manifest.lastError !== null || object(manifest, "renderer").deviceRemovedReason !== null
        ) fail(`meshlet-scene: ${id} reported a renderer failure`);
        const workload = object(manifest, "workload");
        if (
            field<string>(workload, "mode", "string") !== "async-resident-load" ||
            field<boolean>(object(workload, "meshlet"), "enabled", "boolean") !== true
        ) fail(`meshlet-scene: ${id} workload mode mismatch`);
    }
    const perception = object(ids, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`meshlet-scene: ${id} has unknown IDs`);
    const visibleObjects = array(object(evidence, "fullFrame"), "objects") as Record<
        string,
        unknown
    >[];
    if (
        visibleObjects.length === 0 ||
        visibleObjects.some((entry) =>
            entry.kind !== "region-proxy" || !String(entry.name).startsWith("load.region.")
        )
    ) fail(`meshlet-scene: ${id} semantic joins are invalid`);
    return {
        colorSha256: field<string>(object(color, "image"), "pixelSha256", "string"),
        rawIdSha256: field<string>(perception, "rawSha256", "string"),
        diagnosticPngSha256: field<string>(perception, "diagnosticPngSha256", "string"),
        visibleRegionCount: visibleObjects.length,
        backgroundPixelCount: field<number>(
            object(evidence, "fullFrame"),
            "backgroundPixelCount",
            "number",
        ),
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :meshlet-scene");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`meshlet-scene: unexpected argument ${Deno.args[0]}`);

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("meshlet-scene: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
const output = "out/cooked/0009-gpu-meshlet-scene/regions.wlr";
const reportPath = `${root}/out/captures/0009-gpu-meshlet-scene/acceptance.json`;
const CATALOG_SHA256 = "9553748209f9de17e9b524b1c21080404f32df57be62959714b58db1121f0a4e";
let finalReport: Record<string, unknown> | undefined;

await lifecycle("stop");
try {
    await Deno.remove(reportPath);
} catch (error) {
    if (!(error instanceof Deno.errors.NotFound)) throw error;
}
const cooked = await cook(output);

try {
    await lifecycle("start");
    await event("cooked.open", { path: output });
    const initialTransaction = await publish(config());
    await followCamera();
    await configureMeshlets();
    await event("meshlet.enable");

    const status = await event("workbench.status");
    const renderer = object(status, "renderer");
    if (
        field<number>(renderer, "meshShaderTier", "number") < 1 ||
        Number(field<string>(renderer, "shaderModel", "string")) < 6.6 ||
        renderer.deviceRemovedReason !== null
    ) fail("meshlet-scene: reference capability gate failed");
    const meshletStatus = await event("meshlet.status");
    const catalog = object(meshletStatus, "catalog");
    if (
        field<string>(catalog, "sha256", "string") !== CATALOG_SHA256 ||
        field<number>(catalog, "archetypeCount", "number") !== 8 ||
        field<number>(catalog, "lodCount", "number") !== 3 ||
        field<number>(catalog, "meshletCount", "number") !== 88 ||
        field<number>(catalog, "gpuBytes", "number") !== 57_152
    ) fail("meshlet-scene: catalog contract mismatch");

    const initialProbe = await probe();
    same(object(initialProbe, "gpu"), {
        visible: 18_928,
        rejected: 6_672,
        lodCounts: [6_243, 12_640, 45],
        meshlets: 69_270,
        emittedVertices: 2_399_960,
        emittedTriangles: 3_316_944,
        observedArchetypeMask: 255,
    }, "canonical counters");
    const timing = await measuredProbes();
    const initialEvidence = await captureEvidence("initial");
    const firstProcess = field<number>(status, "processId", "number");

    const activeSweep: Record<string, unknown>[] = [];
    for (const radius of [0, 1, 2]) {
        const workload = config(128, radius);
        const transaction = await publish(workload);
        const sample = await probe();
        const expectedCandidates = (radius * 2 + 1) ** 2 * 1_024;
        if (field<number>(sample, "candidateInstanceCount", "number") !== expectedCandidates) {
            fail(`meshlet-scene: radius ${radius} candidate count mismatch`);
        }
        activeSweep.push({ radius, transaction, probe: sample });
    }

    const logicalWorldSweep: Record<string, unknown>[] = [];
    for (const side of [32, 64, 128]) {
        const workload = config(side, 2);
        const transaction = await publish(workload);
        const sample = await probe();
        if (field<number>(sample, "logicalInstanceCount", "number") !== side * side * 1_024) {
            fail(`meshlet-scene: logical world ${side} count mismatch`);
        }
        same(object(sample, "gpu"), object(initialProbe, "gpu"), "logical-world GPU work");
        logicalWorldSweep.push({ side, transaction, probe: sample });
    }

    const maskSweep: Record<string, unknown>[] = [];
    for (const mask of [1, 15, 255]) {
        await configureMeshlets(mask, null);
        const sample = await probe();
        if (field<number>(object(sample, "gpu"), "observedArchetypeMask", "number") !== mask) {
            fail(`meshlet-scene: archetype mask ${mask} was not preserved`);
        }
        maskSweep.push({ mask, probe: sample });
    }

    const forcedLodSweep: Record<string, unknown>[] = [];
    for (const lod of [0, 1, 2]) {
        await configureMeshlets(255, lod);
        const sample = await probe();
        const counts = array(object(sample, "gpu"), "lodCounts") as number[];
        if (counts.some((count, index) => index !== lod && count !== 0)) {
            fail(`meshlet-scene: forced LOD ${lod} leaked into another LOD`);
        }
        forcedLodSweep.push({ lod, probe: sample });
    }
    await configureMeshlets();

    const adjacentTransaction = await publish(config(128, 2, 65, 64));
    await followCamera(65, 64);
    const adjacentProbe = await probe();
    const revisitTransaction = await publish(config());
    await followCamera();
    const revisitProbe = await probe();
    same(object(revisitProbe, "gpu"), object(initialProbe, "gpu"), "revisit counters");
    const revisitEvidence = await captureEvidence("revisit");
    same(revisitEvidence, initialEvidence, "revisit visual evidence");

    await lifecycle("restart");
    await event("cooked.open", { path: output });
    const restartTransaction = await publish(config());
    await followCamera();
    await configureMeshlets();
    await event("meshlet.enable");
    const restartProbe = await probe();
    same(object(restartProbe, "gpu"), object(initialProbe, "gpu"), "restart counters");
    const restartEvidence = await captureEvidence("restart");
    same(restartEvidence, initialEvidence, "restart visual evidence");
    const restartedStatus = await event("workbench.status");
    const restartedProcess = field<number>(restartedStatus, "processId", "number");
    if (restartedProcess === firstProcess) fail("meshlet-scene: process survived restart");

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: "gpu-meshlet-scene-v1",
        capability: {
            adapter: renderer.adapter,
            featureLevel: renderer.featureLevel,
            meshShaderTier: renderer.meshShaderTier,
            shaderModel: renderer.shaderModel,
            debugLayer: renderer.debugLayer,
        },
        catalog,
        cooked,
        processes: { first: firstProcess, restarted: restartedProcess },
        initial: {
            transaction: initialTransaction,
            probe: initialProbe,
            timing,
            evidence: initialEvidence,
        },
        sweeps: {
            active: activeSweep,
            logicalWorld: logicalWorldSweep,
            mask: maskSweep,
            forcedLod: forcedLodSweep,
        },
        movement: {
            adjacent: { transaction: adjacentTransaction, probe: adjacentProbe },
            revisit: {
                transaction: revisitTransaction,
                probe: revisitProbe,
                evidence: revisitEvidence,
            },
        },
        restart: {
            transaction: restartTransaction,
            probe: restartProbe,
            evidence: restartEvidence,
        },
    };
} finally {
    await lifecycle("stop");
}

const cleanup = await sidecar(["status"]);
if (field<boolean>(object(cleanup.value, "runtime"), "running", "boolean")) {
    fail("meshlet-scene: broker remains running");
}
for (const target of array(cleanup.value, "targets") as Record<string, unknown>[]) {
    if (field<boolean>(target, "running", "boolean")) fail("meshlet-scene: target remains running");
}
if (!finalReport) fail("meshlet-scene: no acceptance report was produced");
await Deno.mkdir(reportPath.replace(/[\\/][^\\/]+$/, ""), { recursive: true });
await Deno.writeTextFile(reportPath, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));
