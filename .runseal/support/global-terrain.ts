import {
    array,
    captureEvidence,
    distribution,
    fail,
    field,
    object,
    same,
    validateProbe,
} from "./terrain.ts";

export type Coord = [number, number];
export type GlobalConfig = {
    origin_x: number;
    origin_z: number;
    center_x: number;
    center_z: number;
    active_radius: number;
};

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("RUNSEAL_PROFILE_PATH is not set");
export const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();
let sidecarConfig = "sidecar.toml";

export function useSidecar(config: string): void {
    sidecarConfig = config;
}

export function globalConfig(origin: Coord, center = origin, radius = 2): GlobalConfig {
    return {
        origin_x: origin[0],
        origin_z: origin[1],
        center_x: center[0],
        center_z: center[1],
        active_radius: radius,
    };
}

export async function run(command: string, args: string[], label: string): Promise<void> {
    const status = await new Deno.Command(command, {
        args,
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`${label} failed with exit code ${status.code}`);
}

async function invoke(
    args: string[],
    allowFailure = false,
): Promise<Record<string, unknown>> {
    for (let attempt = 0; attempt < 3; attempt += 1) {
        const result = await new Deno.Command("sidecar", {
            args: [...args, "--config", sidecarConfig, "--format", "json"],
            cwd: root,
            stdout: "piped",
            stderr: "piped",
        }).output();
        const stdout = decoder.decode(result.stdout).trim();
        if (!stdout) {
            if (attempt < 2) {
                await sleep(50);
                continue;
            }
            fail(`sidecar ${args[0]} returned no JSON`);
        }
        let response: Record<string, unknown>;
        try {
            response = JSON.parse(stdout) as Record<string, unknown>;
        } catch (error) {
            if (attempt < 2) {
                await sleep(50);
                continue;
            }
            fail(`sidecar ${args[0]} returned invalid JSON: ${error}: ${stdout}`);
        }
        if (!allowFailure && (!result.success || response.ok === false)) {
            fail(`sidecar ${args[0]} failed: ${JSON.stringify(response.error)}`);
        }
        return response;
    }
    fail("sidecar retry loop exhausted");
}

export async function lifecycle(verb: "start" | "restart" | "stop"): Promise<void> {
    await run("sidecar", [verb, "--config", sidecarConfig], `sidecar ${verb}`);
}

export async function event(
    verb: string,
    payload: unknown = {},
): Promise<Record<string, unknown>> {
    return object(
        await invoke(["inspect", "workbench", verb, JSON.stringify(payload)]),
        "data",
    );
}

export async function rawEvent(
    verb: string,
    payload: string,
): Promise<Record<string, unknown>> {
    return object(await invoke(["inspect", "workbench", verb, payload]), "data");
}

function failure(response: Record<string, unknown>, verb: string, code: string) {
    if (response.ok !== false) fail(`${verb} unexpectedly succeeded`);
    const raw = response.error;
    if (typeof raw === "string") {
        const prefix = `${code}: `;
        if (!raw.startsWith(prefix)) fail(`${verb} returned unexpected error ${raw}`);
        return { code, message: raw.slice(prefix.length) };
    }
    const value = object(response, "error");
    if (field<string>(value, "code", "string") !== code) {
        fail(`${verb} returned unexpected code ${String(value.code)}`);
    }
    return value;
}

export async function failedEvent(verb: string, payload: unknown, code: string) {
    return failure(
        await invoke(["inspect", "workbench", verb, JSON.stringify(payload)], true),
        verb,
        code,
    );
}

export async function failedRawEvent(verb: string, payload: string, code: string) {
    return failure(
        await invoke(["inspect", "workbench", verb, payload], true),
        verb,
        code,
    );
}

export async function cook(path: string, centers: Coord[]): Promise<Record<string, unknown>> {
    const args = ["run", "--locked", "--release", "-p", "terrain-cooker", "--", path];
    for (const [x, z] of centers) args.push("--center", String(x), String(z));
    const output = await new Deno.Command("cargo", {
        args,
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail("terrain cooker failed");
    return JSON.parse(decoder.decode(output.stdout).trim());
}

export async function waitPublished(
    transactionId: number,
): Promise<Record<string, unknown>> {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const renderer = object(status, "renderer");
        if (
            object(status, "stream").pending === null &&
            object(renderer, "transfer").copyPending === null && renderer.published
        ) {
            const published = object(renderer, "published");
            const transaction = object(published, "transaction");
            if (transaction.transactionId === transactionId) return status;
        }
        await sleep(10);
    }
    fail(`terrain transaction ${transactionId} publication timed out`);
}

export async function publish(
    verb: "terrain.schedule" | "terrain.global.schedule",
    config: unknown,
): Promise<Record<string, unknown>> {
    const started = performance.now();
    const scheduled = await event(verb, config);
    const transactionId = field<number>(scheduled, "transactionId", "number");
    await event("workbench.resume");
    const status = await waitPublished(transactionId);
    await event("workbench.pause");
    const published = object(object(status, "renderer"), "published");
    return {
        scheduled,
        published,
        transaction: object(published, "transaction"),
        operatorPublicationMs: performance.now() - started,
    };
}

export async function prepare(
    path: string,
    config: GlobalConfig,
): Promise<Record<string, unknown>> {
    await event("terrain.open", { path });
    const publication = await publish("terrain.global.schedule", config);
    await event("terrain.enable");
    await event("camera.reset");
    await event("workbench.pause");
    return publication;
}

export async function probe(config?: GlobalConfig): Promise<Record<string, unknown>> {
    const value = await event("load.probe");
    validateProbe(value);
    if (config) validateGlobalProbe(value, config);
    return value;
}

export function validateGlobalProbe(
    value: Record<string, unknown>,
    config: GlobalConfig,
): void {
    const global = object(value, "globalAddressing");
    if (
        global.entryCount !== 25 || global.duplicateGlobalCount !== 0 ||
        global.mismatchCount !== 0
    ) fail("global terrain mapping evidence failed");
    field<string>(global, "mappingSha256", "string");
    const actual = object(global, "config");
    same(object(actual, "globalOrigin"), { x: config.origin_x, z: config.origin_z }, "origin");
    same(object(actual, "globalCenter"), { x: config.center_x, z: config.center_z }, "center");
    if (actual.activeRadius !== config.active_radius) fail("global terrain radius mismatch");
    for (const assignment of array(value, "activeMapping")) {
        object(assignment as Record<string, unknown>, "globalRegion");
    }
}

export function localProbe(value: Record<string, unknown>): Record<string, unknown> {
    return {
        revision: value.revision,
        config: value.config,
        localRegions: array(value, "activeMapping").map((entry) =>
            field<number>(entry as Record<string, unknown>, "regionId", "number")
        ),
        payloadSha256: value.payloadSha256,
        cpuEdges: value.cpuEdges,
        gpuEdges: value.gpuEdges,
        geometry: value.geometry,
        submission: value.submission,
        resources: value.resources,
        lod: value.lod,
    };
}

export function globalProbe(value: Record<string, unknown>): Record<string, unknown> {
    const global = object(value, "globalAddressing");
    return {
        config: global.config,
        mappingSha256: global.mappingSha256,
        globals: array(value, "activeMapping").map((entry) =>
            object(entry as Record<string, unknown>, "globalRegion")
        ),
    };
}

export async function capture(id: string, collection: string): Promise<Record<string, unknown>> {
    const raw = await event("perception.capture", {
        id,
        collection,
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    });
    return {
        ...captureEvidence(raw),
        png: field<string>(object(raw, "image"), "pngSha256", "string"),
    };
}

export function transaction(
    publication: Record<string, unknown>,
): Record<string, unknown> {
    return object(publication, "transaction");
}

export function expectCounts(
    report: Record<string, unknown>,
    expected: Record<string, number>,
): void {
    for (const [name, value] of Object.entries(expected)) {
        if (field<number>(report, name, "number") !== value) {
            fail(`terrain transaction ${name} mismatch`);
        }
    }
    const io = object(report, "io");
    if (io.payloadBytes !== report.payloadBytes) fail("terrain I/O payload count diverged");
}

export async function hold(
    kind: "io" | "copy",
    config: GlobalConfig,
    collection: string,
): Promise<Record<string, unknown>> {
    const before = await capture(`${kind}-before`, collection);
    const beforeProbe = await probe();
    await event(`terrain.${kind}_gate.arm`);
    const scheduled = await event("terrain.global.schedule", config);
    const transactionId = field<number>(scheduled, "transactionId", "number");
    await event("workbench.resume");
    const deadline = Date.now() + 10_000;
    let heldStatus: Record<string, unknown> | undefined;
    while (Date.now() < deadline) {
        const status = await event("terrain.status");
        const reached = kind === "io"
            ? object(status, "stream").pending !== null
            : object(object(status, "renderer"), "transfer").copyPending !== null;
        if (reached) {
            heldStatus = status;
            break;
        }
        await sleep(10);
    }
    if (!heldStatus) fail(`${kind} gate did not hold the transaction`);
    await event("workbench.pause");
    const heldCapture = await capture(`${kind}-held`, collection);
    same(heldCapture, before, `${kind} held attachment`);
    same(globalProbe(await probe()), globalProbe(beforeProbe), `${kind} held mapping`);
    await event(`terrain.${kind}_gate.release`);
    await event("workbench.resume");
    const status = await waitPublished(transactionId);
    await event("workbench.pause");
    return {
        before,
        heldCapture,
        heldStatus,
        published: object(object(status, "renderer"), "published"),
    };
}

export function transactionDistributions(samples: Record<string, unknown>[]) {
    const values = (name: string) => samples.map((sample) => field<number>(sample, name, "number"));
    const io = (name: string) =>
        samples.map((sample) => field<number>(object(sample, "io"), name, "number"));
    return {
        sampleCount: samples.length,
        scheduleMs: distribution(values("scheduleMs")),
        copyGpuMs: nonnegativeDistribution(values("copyGpuMs")),
        copyToPublicationMs: distribution(values("copyToPublicationMs")),
        pendingMs: distribution(values("pendingMs")),
        io: {
            payloadBytes: [...new Set(io("payloadBytes"))],
            readMs: nonnegativeDistribution(io("readMs")),
            verifyMs: nonnegativeDistribution(io("verifyMs")),
            totalMs: nonnegativeDistribution(io("totalMs")),
        },
    };
}

function nonnegativeDistribution(values: number[]): Record<string, number> {
    if (values.some((value) => !Number.isFinite(value) || value < 0)) {
        fail("invalid nonnegative timing sample");
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

export async function assertStopped(config: string): Promise<void> {
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

export function sleep(milliseconds: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
