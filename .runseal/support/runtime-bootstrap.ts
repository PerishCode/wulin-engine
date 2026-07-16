import {
    assertStopped,
    type Coord,
    event,
    fail,
    frame,
    type Json,
    lifecycle,
    number,
    object,
    root,
    same,
    status,
    useSidecar,
} from "./canonical-runtime.ts";

const SIDECAR = "sidecar.bootstrap.toml";
const CONFIG = "out/cooked/bootstrap/runtime.json";
const decoder = new TextDecoder();

function document(
    terrain: string,
    objects: string,
    origin: Coord,
    center: Coord,
): Json {
    return {
        schemaVersion: 2,
        terrain,
        objects,
        globalOrigin: { x: origin[0], z: origin[1] },
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
    const output = await new Deno.Command("cargo", {
        args: [
            "run",
            "--locked",
            "-p",
            "workbench",
            "--",
            `--bootstrap=${CONFIG}`,
            `--sidecar-stamp=bootstrap-failure-${label.replaceAll(" ", "-")}`,
        ],
        cwd: root,
        stdout: "piped",
        stderr: "piped",
    }).output();
    const stdout = decoder.decode(output.stdout).trim();
    const stderr = decoder.decode(output.stderr).trim();
    if (output.success) fail(`${label} bootstrap unexpectedly succeeded`);
    if (stdout.includes('"role":"workbench"')) {
        fail(`${label} bootstrap emitted readiness before failing`);
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

function startupInvariant(value: Json): Json {
    return {
        revision: value.revision,
        mode: value.mode,
        configPath: value.configPath,
        configBytes: value.configBytes,
        configSha256: value.configSha256,
        terrainPath: value.terrainPath,
        objectPath: value.objectPath,
        globalConfig: value.globalConfig,
        playableRegionBounds: value.playableRegionBounds,
    };
}

export async function bootstrapGates(
    terrain: string,
    objects: string,
    corruptObjects: string,
    base: Coord,
    collection: string,
): Promise<Json> {
    console.log("==> declarative canonical bootstrap and readiness gates");
    useSidecar(SIDECAR);
    await lifecycle("stop");

    const invalid = document(terrain, objects, base, base);
    invalid.fallback = true;
    await writeDocument(invalid);
    const invalidDocument = await failedStart("invalid document");

    await writeDocument(document(terrain, "out/cooked/bootstrap/missing.wlr", base, base));
    const missingSource = await failedStart("missing source");

    const corruptCenter: Coord = [base[0] + 70, base[1]];
    await writeDocument(document(terrain, corruptObjects, corruptCenter, corruptCenter));
    const corruptPayload = await failedStart("corrupt payload");

    await writeDocument(document(terrain, objects, base, base));
    await lifecycle("start");
    const firstStatus = await status();
    if (object(firstStatus, "workload").mode !== "canonical-runtime") {
        fail("configured startup became ready before canonical runtime publication");
    }
    const firstStartup = object(firstStatus, "startup");
    if (
        firstStartup.mode !== "canonical-bootstrap" ||
        number(firstStartup, "readyFrameIndex") < 1 ||
        number(firstStartup, "elapsedMs") <= 0
    ) fail("configured startup status is incomplete");
    await event("canonical.time.pause");
    await event("canonical.time.set", { tick: 0 });
    const firstFrame = await frame("bootstrap-first", collection);
    const firstProcessId = number(firstStatus, "processId");

    await lifecycle("restart");
    const restartedStatus = await status();
    if (
        number(restartedStatus, "processId") === firstProcessId ||
        object(restartedStatus, "workload").mode !== "canonical-runtime"
    ) fail("configured restart did not create a fresh canonical-ready process");
    const restartedStartup = object(restartedStatus, "startup");
    same(
        startupInvariant(restartedStartup),
        startupInvariant(firstStartup),
        "bootstrap restart configuration",
    );
    await event("canonical.time.pause");
    await event("canonical.time.set", { tick: 0 });
    const restartedFrame = await frame("bootstrap-restarted", collection);
    same(restartedFrame.stable, firstFrame.stable, "bootstrap restart frame");
    const restartedProcessId = number(restartedStatus, "processId");
    await lifecycle("stop");
    const stopped = await assertStopped(restartedProcessId);
    useSidecar("sidecar.toml");

    return {
        configPath: CONFIG,
        invalidDocument,
        missingSource,
        corruptPayload,
        first: { status: firstStatus, frame: firstFrame },
        restarted: { status: restartedStatus, frame: restartedFrame },
        stopped,
    };
}
