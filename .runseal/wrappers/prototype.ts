type Json = Record<string, unknown>;

const REVISION = "prototype-operator-v2";
const SIDECAR = "sidecar.prototype.toml";
const DIRECTORY = "out/cooked/prototype";
const TERRAIN = `${DIRECTORY}/terrain.wlt`;
const OBJECTS = `${DIRECTORY}/objects.wlr`;
const CONFIG = "out/cooked/bootstrap/runtime.json";
const HALF_EXTENT = 8;
const PLAYABLE_HALF_EXTENT = 6;
const ACTIVE_RADIUS = 2;
const EXPECTED_CENTER_COUNT = (HALF_EXTENT * 2 + 1) ** 2;
const EXPECTED_REGION_COUNT = (HALF_EXTENT * 2 + ACTIVE_RADIUS * 2 + 1) ** 2;

function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("prototype: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const decoder = new TextDecoder();

function object(value: unknown, key: string): Json {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
        fail(`prototype: ${key} must be an object`);
    }
    return value as Json;
}

function number(value: Json, key: string): number {
    const field = value[key];
    if (typeof field !== "number" || !Number.isFinite(field)) {
        fail(`prototype: ${key} must be a finite number`);
    }
    return field;
}

function string(value: Json, key: string): string {
    const field = value[key];
    if (typeof field !== "string") fail(`prototype: ${key} must be a string`);
    return field;
}

async function captured(command: string, args: string[], label: string): Promise<Json> {
    const output = await new Deno.Command(command, {
        args,
        cwd: root,
        stdin: "null",
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (!output.success) fail(`prototype: ${label} failed with exit code ${output.code}`);
    try {
        return JSON.parse(decoder.decode(output.stdout).trim()) as Json;
    } catch (error) {
        fail(`prototype: ${label} returned invalid JSON: ${error}`);
    }
}

async function sidecar(args: string[]): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [...args, "--config", SIDECAR],
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`prototype: Sidecar failed with exit code ${status.code}`);
}

async function sidecarStatus(): Promise<Json> {
    return await captured(
        "sidecar",
        ["status", "--config", SIDECAR, "--format", "json"],
        "Sidecar status",
    );
}

function hasOwnedProcess(status: Json): boolean {
    const runtime = object(status.runtime, "runtime");
    if (typeof runtime.running !== "boolean") {
        fail("prototype: Sidecar runtime running state must be boolean");
    }
    if (runtime.running) return true;
    const targets = status.targets;
    if (!Array.isArray(targets)) fail("prototype: Sidecar targets must be an array");
    return targets.some((target) => {
        const value = object(target, "target");
        const pids = value.pids;
        if (typeof value.running !== "boolean") {
            fail("prototype: Sidecar target running state must be boolean");
        }
        if (!Array.isArray(pids)) fail("prototype: Sidecar target PIDs must be an array");
        return value.running || pids.length !== 0;
    });
}

function centers(): [number, number][] {
    const values: [number, number][] = [];
    for (let z = -HALF_EXTENT; z <= HALF_EXTENT; z += 1) {
        for (let x = -HALF_EXTENT; x <= HALF_EXTENT; x += 1) values.push([x, z]);
    }
    return values;
}

function centerArgs(values: [number, number][]): string[] {
    return values.flatMap(([x, z]) => ["--global-center", String(x), String(z)]);
}

async function prepare(): Promise<Json> {
    const started = performance.now();
    await Deno.mkdir(`${root}/${DIRECTORY}`, { recursive: true });
    await Deno.mkdir(`${root}/out/cooked/bootstrap`, { recursive: true });
    const scheduledCenters = centers();
    const args = centerArgs(scheduledCenters);
    const terrain = await captured(
        "cargo",
        ["run", "--locked", "--release", "-p", "terrain-cooker", "--", TERRAIN, ...args],
        "terrain cook",
    );
    const objects = await captured(
        "cargo",
        [
            "run",
            "--locked",
            "--release",
            "-p",
            "region-cooker",
            "--",
            OBJECTS,
            "--physical-order",
            "a",
            "--presentation",
            "base",
            ...args,
        ],
        "object cook",
    );
    const terrainMetadata = object(terrain.metadata, "terrain metadata");
    const objectMetadata = object(objects.metadata, "object metadata");
    if (
        scheduledCenters.length !== EXPECTED_CENTER_COUNT ||
        number(terrainMetadata, "regionCount") !== EXPECTED_REGION_COUNT ||
        number(objectMetadata, "regionCount") !== EXPECTED_REGION_COUNT ||
        number(objectMetadata, "payloadSchema") !== 3
    ) fail("prototype: cooked sandbox shape diverged");

    const document = {
        schemaVersion: 2,
        terrain: TERRAIN,
        objects: OBJECTS,
        globalOrigin: { x: 0, z: 0 },
        globalCenter: { x: 0, z: 0 },
        activeRadius: ACTIVE_RADIUS,
        playableRegionBounds: {
            minimum: { x: -PLAYABLE_HALF_EXTENT, z: -PLAYABLE_HALF_EXTENT },
            maximum: { x: PLAYABLE_HALF_EXTENT, z: PLAYABLE_HALF_EXTENT },
        },
    };
    const encoded = new TextEncoder().encode(`${JSON.stringify(document, null, 2)}\n`);
    await Deno.writeFile(`${root}/${CONFIG}`, encoded);
    const configSha256 = Array.from(
        new Uint8Array(await crypto.subtle.digest("SHA-256", encoded)),
        (byte) => byte.toString(16).padStart(2, "0"),
    ).join("");
    return {
        revision: REVISION,
        sandbox: {
            halfExtentRegions: HALF_EXTENT,
            scheduledCenterCount: scheduledCenters.length,
            sourceRegionCount: EXPECTED_REGION_COUNT,
            activeRadius: ACTIVE_RADIUS,
            globalOrigin: { x: 0, z: 0 },
            globalCenter: { x: 0, z: 0 },
            playableRegionBounds: {
                minimum: { x: -PLAYABLE_HALF_EXTENT, z: -PLAYABLE_HALF_EXTENT },
                maximum: { x: PLAYABLE_HALF_EXTENT, z: PLAYABLE_HALF_EXTENT },
            },
        },
        terrain: {
            path: TERRAIN,
            fileBytes: number(terrainMetadata, "fileBytes"),
            fileSha256: string(terrain, "fileSha256"),
        },
        objects: {
            path: OBJECTS,
            fileBytes: number(objectMetadata, "fileBytes"),
            fileSha256: string(objects, "fileSha256"),
        },
        config: { path: CONFIG, fileBytes: encoded.byteLength, fileSha256: configSha256 },
        elapsedMilliseconds: performance.now() - started,
    };
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log("Usage: runseal :prototype <start|restart|stop|status>");
    Deno.exit(0);
}

const [verb, ...args] = Deno.args;
if (args.length !== 0) fail(`prototype: ${verb ?? "<missing>"} accepts no arguments`);
switch (verb) {
    case "start": {
        if (hasOwnedProcess(await sidecarStatus())) {
            fail("prototype: stop the running prototype before preparing a fresh sandbox");
        }
        console.log(JSON.stringify(await prepare(), null, 2));
        await sidecar(["start"]);
        break;
    }
    case "restart":
    case "stop":
        await sidecar([verb]);
        break;
    case "status":
        console.log(JSON.stringify(await sidecarStatus(), null, 2));
        break;
    default:
        fail(`prototype: unknown command ${verb ?? "<missing>"}`);
}
