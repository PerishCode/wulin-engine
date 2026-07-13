import {
    array,
    collectEnvironment,
    distribution,
    fail,
    field,
    object,
    same,
} from "../support/terrain.ts";

const REVISION = "camera-relative-global-space-v1";
const COLLECTION = "0019-camera-relative-global-space";
const REPORT_PATH = `out/captures/${COLLECTION}/acceptance.json`;
const FAR = 2 ** 40;

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :global-space");
    Deno.exit(0);
}
if (Deno.args.length > 0) fail(`unexpected argument ${Deno.args[0]}`);
const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
let sidecarConfig = "sidecar.toml";

async function run(command: string, args: string[], label: string): Promise<void> {
    const status = await new Deno.Command(command, {
        args,
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`${label} failed with exit code ${status.code}`);
}

async function invoke(args: string[], allowFailure = false): Promise<Record<string, unknown>> {
    const result = await new Deno.Command("sidecar", {
        args: [...args, "--config", sidecarConfig, "--format", "json"],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(result.stdout).trim();
    const stderr = decoder.decode(result.stderr).trim();
    if (!stdout) fail(`sidecar ${args[0]} returned no JSON${stderr ? `: ${stderr}` : ""}`);
    let response: Record<string, unknown>;
    try {
        response = JSON.parse(stdout) as Record<string, unknown>;
    } catch (error) {
        fail(`sidecar ${args[0]} returned invalid JSON: ${error}: ${stdout}`);
    }
    if (!allowFailure && (!result.success || response.ok === false)) {
        fail(`sidecar ${args[0]} failed: ${JSON.stringify(response.error)}`);
    }
    return response;
}

async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    await run("sidecar", [verb, "--config", sidecarConfig], `sidecar ${verb}`);
}

async function event(verb: string, payload: unknown = {}): Promise<Record<string, unknown>> {
    return object(
        await invoke(["inspect", "workbench", verb, JSON.stringify(payload)]),
        "data",
    );
}

async function failedEvent(
    verb: string,
    payload: unknown,
    expectedCode: string,
): Promise<Record<string, unknown>> {
    const response = await invoke(
        ["inspect", "workbench", verb, JSON.stringify(payload)],
        true,
    );
    if (response.ok !== false) fail(`${verb} unexpectedly succeeded`);
    const raw = response.error;
    if (typeof raw === "string") {
        const prefix = `${expectedCode}: `;
        if (!raw.startsWith(prefix)) fail(`${verb} failed with unexpected error ${raw}`);
        return { code: expectedCode, message: raw.slice(prefix.length) };
    }
    const structured = object(response, "error");
    if (field<string>(structured, "code", "string") !== expectedCode) {
        fail(`${verb} failed with unexpected code ${String(structured.code)}`);
    }
    return structured;
}

function coord(owner: Record<string, unknown>, name: string): [number, number] {
    const value = object(owner, name);
    return [field<number>(value, "x", "number"), field<number>(value, "z", "number")];
}

function expectCoord(
    owner: Record<string, unknown>,
    name: string,
    expected: [number, number],
): void {
    same(coord(owner, name), expected, `${name} coordinate`);
}

function validateWorld(status: Record<string, unknown>): void {
    if (field<string>(status, "revision", "string") !== REVISION) {
        fail("world revision mismatch");
    }
    if (
        field<number>(status, "regionSideMeters", "number") !== 16 ||
        field<number>(status, "signedCoordinateBits", "number") !== 64 ||
        field<number>(status, "maximumRenderRegionDelta", "number") !== 8
    ) fail("world coordinate contract mismatch");
    if (array(status, "objects").length !== 8) fail("world object count mismatch");
}

function validateProbe(probe: Record<string, unknown>): void {
    if (field<string>(probe, "revision", "string") !== `${REVISION.replace("-v1", "")}-probe-v1`) {
        fail("world probe revision mismatch");
    }
    for (
        const [name, expected] of [
            ["sampleCount", 25_600],
            ["normalizationMismatchCount", 0],
            ["reconstructionMismatchCount", 0],
            ["boundarySampleCount", 9],
            ["boundaryMismatchCount", 0],
            ["nonFiniteCount", 0],
            ["perSampleAllocationBytes", 0],
        ] as const
    ) {
        if (field<number>(probe, name, "number") !== expected) {
            fail(`world probe ${name} mismatch`);
        }
    }
    if (field<number>(probe, "maximumRenderedRegionDelta", "number") > 8) {
        fail("world probe exceeded the render-region bound");
    }
    for (
        const name of [
            "globalPositionHash",
            "renderPositionHash",
            "localMatrixHash",
            "renderMatrixHash",
            "canonicalClipSpaceHash",
            "renderClipSpaceHash",
        ]
    ) field<string>(probe, name, "string");
    const clipError = field<number>(probe, "maximumClipSpaceAbsoluteError", "number");
    if (!Number.isFinite(clipError) || clipError > 0.0001) {
        fail(`world probe clip-space error ${clipError} exceeds 0.0001`);
    }
}

async function probe(): Promise<Record<string, unknown>> {
    const value = await event("world.probe");
    validateProbe(value);
    return value;
}

function stableProbe(probe: Record<string, unknown>): Record<string, unknown> {
    return {
        globalPositionHash: probe.globalPositionHash,
        renderPositionHash: probe.renderPositionHash,
        localMatrixHash: probe.localMatrixHash,
        renderMatrixHash: probe.renderMatrixHash,
        canonicalClipSpaceHash: probe.canonicalClipSpaceHash,
        renderClipSpaceHash: probe.renderClipSpaceHash,
        maximumClipSpaceAbsoluteError: probe.maximumClipSpaceAbsoluteError,
        maximumRenderedRegionDelta: probe.maximumRenderedRegionDelta,
    };
}

async function capture(id: string): Promise<Record<string, unknown>> {
    const value = await event("perception.capture", {
        id,
        collection: COLLECTION,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    if (value.lastError !== null || object(value, "renderer").deviceRemovedReason !== null) {
        fail(`${id} reported a renderer failure`);
    }
    const perception = object(value, "perception");
    const evidence = object(perception, "evidence");
    if (array(evidence, "unknownIds").length !== 0) fail(`${id} contains unknown IDs`);
    return {
        color: field<string>(object(value, "image"), "pixelSha256", "string"),
        png: field<string>(object(value, "image"), "pngSha256", "string"),
        objectId: field<string>(perception, "rawSha256", "string"),
        diagnostic: field<string>(perception, "diagnosticPngSha256", "string"),
        visibleObjects: array(object(evidence, "fullFrame"), "objects").length,
        gpuReadbackMs: field<number>(
            object(value, "timing"),
            "gpuSubmissionAndReadbackMs",
            "number",
        ),
    };
}

function stableCapture(capture: Record<string, unknown>): Record<string, unknown> {
    return {
        color: capture.color,
        png: capture.png,
        objectId: capture.objectId,
        diagnostic: capture.diagnostic,
        visibleObjects: capture.visibleObjects,
    };
}

async function relocate(anchor: [number, number]): Promise<Record<string, unknown>> {
    const status = await event("world.relocate", {
        region_x: anchor[0],
        region_z: anchor[1],
    });
    validateWorld(status);
    expectCoord(status, "sceneAnchor", anchor);
    expectCoord(status, "renderOrigin", anchor);
    return status;
}

async function rebase(origin: [number, number]): Promise<Record<string, unknown>> {
    const status = await event("world.rebase", {
        region_x: origin[0],
        region_z: origin[1],
    });
    validateWorld(status);
    expectCoord(status, "renderOrigin", origin);
    return status;
}

function expectLocalInvariant(
    baseline: Record<string, unknown>,
    candidate: Record<string, unknown>,
    label: string,
): void {
    for (
        const name of [
            "renderPositionHash",
            "localMatrixHash",
            "renderMatrixHash",
            "canonicalClipSpaceHash",
            "renderClipSpaceHash",
        ]
    ) {
        if (candidate[name] !== baseline[name]) fail(`${label} changed ${name}`);
    }
}

async function runCompatibility(): Promise<Record<string, unknown>> {
    await run("runseal", [":spatial-scene"], "Experiment 0003 compatibility workflow");
    await run("runseal", [":region-traversal"], "Experiment 0018 compatibility workflow");
    return {
        spatial: "out/captures/0003-spatial-calibration-scene/acceptance.json",
        traversal: "out/captures/0018-camera-driven-region-traversal/acceptance.json",
    };
}

async function assertStopped(config: string): Promise<void> {
    const previous = sidecarConfig;
    sidecarConfig = config;
    const status = await invoke(["status"]);
    if (field<boolean>(object(status, "runtime"), "running", "boolean")) {
        fail(`${config} broker remains running`);
    }
    for (const target of array(status, "targets")) {
        if (
            typeof target !== "object" || target === null || Array.isArray(target) ||
            field<boolean>(target as Record<string, unknown>, "running", "boolean")
        ) fail(`${config} target remains running`);
    }
    sidecarConfig = previous;
}

await lifecycle("stop");
sidecarConfig = "sidecar.benchmark.toml";
await lifecycle("stop");
sidecarConfig = "sidecar.toml";
const compatibility = await runCompatibility();
const environment = await collectEnvironment(root);
let finalReport: Record<string, unknown> | undefined;

try {
    await lifecycle("start");
    await event("workbench.pause");
    await event("camera.reset");
    const firstWorkbench = await event("workbench.status");
    const firstProcess = field<number>(firstWorkbench, "processId", "number");
    const correctnessRenderer = object(firstWorkbench, "renderer");
    if (
        correctnessRenderer.debugLayer !== true || correctnessRenderer.deviceRemovedReason !== null
    ) {
        fail("debug world capability gate failed");
    }

    const zeroStatus = await event("world.reset");
    validateWorld(zeroStatus);
    const baselineProbe = await probe();
    const baselineCapture = await capture("baseline-zero");
    const anchors: [number, number][] = [
        [2 ** 20, -(2 ** 20)],
        [FAR, -FAR],
        [-FAR, FAR],
    ];
    const anchorEvidence = [];
    const globalHashes = new Set([baselineProbe.globalPositionHash]);
    for (let index = 0; index < anchors.length; index += 1) {
        const anchor = anchors[index];
        const status = await relocate(anchor);
        const anchorProbe = await probe();
        const anchorCapture = await capture(`anchor-${index}`);
        expectLocalInvariant(baselineProbe, anchorProbe, `anchor ${anchor}`);
        same(
            stableCapture(anchorCapture),
            stableCapture(baselineCapture),
            `anchor ${anchor} attachments`,
        );
        globalHashes.add(anchorProbe.globalPositionHash);
        anchorEvidence.push({
            anchor,
            status,
            probe: stableProbe(anchorProbe),
            capture: anchorCapture,
        });
    }
    if (globalHashes.size !== anchors.length + 1) fail("global anchor hashes are not distinct");

    const farAnchor: [number, number] = [FAR, -FAR];
    await relocate(farAnchor);
    const farProbe = await probe();
    const farCapture = await capture("far-anchor");
    const rebaseEvidence = [];
    for (const [index, offset] of [[1, 1], [-1, -1], [4, -4], [-4, 4]].entries()) {
        const origin: [number, number] = [farAnchor[0] + offset[0], farAnchor[1] + offset[1]];
        const status = await rebase(origin);
        const rebasedProbe = await probe();
        const rebasedCapture = await capture(`rebase-${index}`);
        if (rebasedProbe.globalPositionHash !== farProbe.globalPositionHash) {
            fail(`rebase ${offset} changed global positions`);
        }
        if (rebasedProbe.localMatrixHash !== farProbe.localMatrixHash) {
            fail(`rebase ${offset} changed local matrices`);
        }
        if (rebasedProbe.canonicalClipSpaceHash !== farProbe.canonicalClipSpaceHash) {
            fail(`rebase ${offset} changed canonical clip-space evidence`);
        }
        if (rebasedProbe.renderPositionHash === farProbe.renderPositionHash) {
            fail(`rebase ${offset} did not change render-relative positions`);
        }
        same(
            stableCapture(rebasedCapture),
            stableCapture(farCapture),
            `rebase ${offset} attachments`,
        );
        rebaseEvidence.push({ offset, status, probe: stableProbe(rebasedProbe) });
    }
    await rebase(farAnchor);
    same(stableProbe(await probe()), stableProbe(farProbe), "far-anchor rebase revisit");
    same(
        stableCapture(await capture("far-revisit")),
        stableCapture(farCapture),
        "far-anchor rebase revisit attachments",
    );

    const beforeReject = await event("world.status");
    const rejected = await failedEvent(
        "world.rebase",
        { region_x: farAnchor[0] + 10, region_z: farAnchor[1] },
        "invalid_world_space",
    );
    same(await event("world.status"), beforeReject, "rejected rebase state");
    same(
        stableCapture(await capture("rejected-rebase")),
        stableCapture(farCapture),
        "rejected rebase attachments",
    );

    await event("load.configure", {
        world_region_side: 128,
        active_center_x: 64,
        active_center_z: 64,
        active_radius: 2,
    });
    const modeRejected = await failedEvent(
        "world.relocate",
        { region_x: 0, region_z: 0 },
        "world_mode_required",
    );
    await event("load.disable");

    await event("world.reset");
    same(stableProbe(await probe()), stableProbe(baselineProbe), "zero reset probe");
    same(
        stableCapture(await capture("zero-revisit")),
        stableCapture(baselineCapture),
        "zero reset attachments",
    );

    await lifecycle("restart");
    await event("workbench.pause");
    const restartedWorkbench = await event("workbench.status");
    const restartedProcess = field<number>(restartedWorkbench, "processId", "number");
    if (restartedProcess === firstProcess) fail("workbench process survived restart");
    await relocate(farAnchor);
    same(stableProbe(await probe()), stableProbe(farProbe), "restart far-anchor probe");
    same(
        stableCapture(await capture("restart-far")),
        stableCapture(farCapture),
        "restart far-anchor attachments",
    );

    await lifecycle("stop");
    sidecarConfig = "sidecar.benchmark.toml";
    await lifecycle("start");
    await event("workbench.pause");
    const benchmarkWorkbench = await event("workbench.status");
    const benchmarkRenderer = object(benchmarkWorkbench, "renderer");
    if (benchmarkRenderer.debugLayer !== false || benchmarkRenderer.deviceRemovedReason !== null) {
        fail("release world capability gate failed");
    }
    const probeRoundTripMs: number[] = [];
    const conversionMs: number[] = [];
    const captureRoundTripMs: number[] = [];
    const gpuReadbackMs: number[] = [];
    let benchmarkCapture: Record<string, unknown> | undefined;
    let lastProbe: Record<string, unknown> | undefined;
    const offsets: [number, number][] = [[0, 0], [1, 1], [-1, -1], [4, -4], [-4, 4]];
    for (let index = 0; index < 64; index += 1) {
        const anchor: [number, number] = index % 2 === 0 ? farAnchor : [0, 0];
        await relocate(anchor);
        const offset = offsets[index % offsets.length];
        if (offset[0] !== 0 || offset[1] !== 0) {
            await rebase([anchor[0] + offset[0], anchor[1] + offset[1]]);
        }
        const probeStarted = performance.now();
        const measuredProbe = await probe();
        probeRoundTripMs.push(performance.now() - probeStarted);
        conversionMs.push(field<number>(measuredProbe, "conversionElapsedNs", "number") / 1e6);
        if (measuredProbe.canonicalClipSpaceHash !== baselineProbe.canonicalClipSpaceHash) {
            fail(`benchmark probe ${index} changed canonical clip-space evidence`);
        }
        lastProbe = measuredProbe;
        if (index % 4 === 0) {
            const captureStarted = performance.now();
            const measuredCapture = await capture(`benchmark-${String(index).padStart(2, "0")}`);
            captureRoundTripMs.push(performance.now() - captureStarted);
            gpuReadbackMs.push(field<number>(measuredCapture, "gpuReadbackMs", "number"));
            if (benchmarkCapture) {
                same(
                    stableCapture(measuredCapture),
                    stableCapture(benchmarkCapture),
                    "benchmark attachments",
                );
            }
            benchmarkCapture = measuredCapture;
        }
    }
    if (!lastProbe || !benchmarkCapture) fail("benchmark evidence was not produced");

    finalReport = {
        schemaVersion: 1,
        outcome: "pass",
        revision: REVISION,
        environment,
        compatibility,
        capability: { correctness: correctnessRenderer, benchmark: benchmarkRenderer },
        processes: { first: firstProcess, restarted: restartedProcess },
        baseline: {
            status: zeroStatus,
            probe: stableProbe(baselineProbe),
            capture: baselineCapture,
        },
        anchors: anchorEvidence,
        rebases: rebaseEvidence,
        rejection: { range: rejected, mode: modeRejected },
        restart: { processId: restartedProcess, probe: stableProbe(farProbe), capture: farCapture },
        benchmark: {
            probeSampleCount: 64,
            captureSampleCount: captureRoundTripMs.length,
            probeRoundTripMs: distribution(probeRoundTripMs),
            conversionMs: distribution(conversionMs),
            captureRoundTripMs: distribution(captureRoundTripMs),
            gpuReadbackMs: distribution(gpuReadbackMs),
            finalProbe: stableProbe(lastProbe),
            capture: benchmarkCapture,
        },
    };
} finally {
    await lifecycle("stop");
    sidecarConfig = "sidecar.toml";
    await lifecycle("stop");
}

await assertStopped("sidecar.toml");
await assertStopped("sidecar.benchmark.toml");
if (!finalReport) fail("global-space experiment did not produce a report");
await Deno.mkdir(`${root}/out/captures/${COLLECTION}`, { recursive: true });
await Deno.writeTextFile(`${root}/${REPORT_PATH}`, `${JSON.stringify(finalReport, null, 2)}\n`);
console.log(JSON.stringify(finalReport, null, 2));
