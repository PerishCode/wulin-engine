import { dispatchTerrain } from "../support/workbench-terrain.ts";
import { dispatchWorld } from "../support/workbench-world.ts";

function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

async function run(args: string[]): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [...args, "--config", "sidecar.toml"],
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) {
        fail(`workbench: sidecar failed with exit code ${status.code}`);
    }
}

function channel(value: string, name: string): number {
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed < 0 || parsed > 1) {
        fail(`workbench: ${name} must be a number in the range 0..=1`);
    }
    return parsed;
}

function finite(value: string, name: string): number {
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) fail(`workbench: ${name} must be a finite number`);
    return parsed;
}

function pixel(value: string, name: string): number {
    const parsed = Number(value);
    if (!Number.isSafeInteger(parsed) || parsed < 0) {
        fail(`workbench: ${name} must be a non-negative integer`);
    }
    return parsed;
}

function perceptionPayload(verb: string, args: string[]): Record<string, unknown> {
    const hasRegion = verb === "perception-region";
    if ((!hasRegion && args.length > 1) || (hasRegion && args.length !== 5)) {
        fail(
            hasRegion
                ? "workbench: perception-region requires id x y width height"
                : "workbench: perception accepts at most one capture id",
        );
    }
    const payload: Record<string, unknown> = {
        id: args[0] ?? "perception",
        collection: "operator",
        samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
    };
    if (hasRegion) {
        payload.region = {
            x: pixel(args[1], "region x"),
            y: pixel(args[2], "region y"),
            width: pixel(args[3], "region width"),
            height: pixel(args[4], "region height"),
        };
    }
    return payload;
}

async function configureSkeletal(args: string[]): Promise<void> {
    if (args.length > 6) {
        fail("workbench: skeletal-config accepts animated bones phases tick mode LOD");
    }
    const mode = args[4] ?? "shared";
    if (mode !== "shared" && mode !== "unique") {
        fail("workbench: skeletal mode must be shared or unique");
    }
    const forced = args[5] ?? "auto";
    await run([
        "inspect",
        "workbench",
        "skeletal.configure",
        JSON.stringify({
            animated_percent: pixel(args[0] ?? "100", "animated percent"),
            bone_count: pixel(args[1] ?? "64", "bone count"),
            phase_count: pixel(args[2] ?? "64", "phase count"),
            time_tick: pixel(args[3] ?? "0", "time tick"),
            unique_poses: mode === "unique",
            forced_lod: forced === "auto" ? null : pixel(forced, "forced LOD"),
        }),
        "--format",
        "json",
    ]);
}

async function configureSurface(args: string[]): Promise<void> {
    if (args.length > 2) fail("workbench: surface-config accepts materials and mip");
    await run([
        "inspect",
        "workbench",
        "surface.configure",
        JSON.stringify({
            material_count: pixel(args[0] ?? "64", "material count"),
            mip_level: pixel(args[1] ?? "0", "mip level"),
        }),
        "--format",
        "json",
    ]);
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log(
        "Usage: runseal :workbench <start|status|inspect|capture|perception|perception-region|color|camera|camera-set|camera-reset|scene|world|world-relocate|world-rebase|world-reset|world-probe|load|load-config|load-disable|load-probe|resident|resident-stream|async|async-schedule|async-gate-arm|async-gate-release|cooked|cooked-open|cooked-schedule|cooked-gate-arm|cooked-gate-release|meshlet|meshlet-config|meshlet-enable|meshlet-disable|skeletal|skeletal-config|skeletal-enable|skeletal-disable|surface|surface-config|surface-enable|surface-disable|occlusion-enable|occlusion-disable|occlusion-reset|terrain|terrain-open|terrain-schedule|terrain-global-schedule|terrain-enable|terrain-disable|terrain-lod|terrain-lod-config|terrain-lod-enable|terrain-lod-disable|terrain-io-gate-arm|terrain-io-gate-release|terrain-copy-gate-arm|terrain-copy-gate-release|composition|composition-schedule|composition-enable|composition-disable|composition-traversal-enable|composition-traversal-disable|composition-order|composition-fixture|pause|resume|restart|stop>",
    );
    console.log("\nControl and inspect the native engine workbench through Sidecar.");
    Deno.exit(0);
}

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("workbench: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const [verb, ...args] = Deno.args;

if (await dispatchTerrain(verb, args, run)) Deno.exit(0);
if (await dispatchWorld(verb, args, run)) Deno.exit(0);

switch (verb) {
    case "start":
    case "restart":
    case "stop":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([verb]);
        break;
    case "status":
        if (args.length > 0) fail("workbench: status does not accept arguments");
        await run(["status", "--format", "json"]);
        break;
    case "inspect":
        if (args.length > 0) fail("workbench: inspect does not accept arguments");
        await run(["inspect", "workbench", "workbench.status", "--format", "json"]);
        break;
    case "camera":
        if (args.length > 0) fail("workbench: camera does not accept arguments");
        await run(["inspect", "workbench", "camera.status", "--format", "json"]);
        break;
    case "camera-reset":
        if (args.length > 0) fail("workbench: camera-reset does not accept arguments");
        await run(["inspect", "workbench", "camera.reset", "--format", "json"]);
        break;
    case "scene":
        if (args.length > 0) fail("workbench: scene does not accept arguments");
        await run(["inspect", "workbench", "scene.list_objects", "--format", "json"]);
        break;
    case "load":
        if (args.length > 0) fail("workbench: load does not accept arguments");
        await run(["inspect", "workbench", "load.status", "--format", "json"]);
        break;
    case "load-disable":
        if (args.length > 0) fail("workbench: load-disable does not accept arguments");
        await run(["inspect", "workbench", "load.disable", "--format", "json"]);
        break;
    case "load-probe":
        if (args.length > 0) fail("workbench: load-probe does not accept arguments");
        await run(["inspect", "workbench", "load.probe", "--format", "json"]);
        break;
    case "load-config": {
        if (args.length < 1 || args.length > 4) {
            fail(
                "workbench: load-config requires world side and optional center x, center z, radius",
            );
        }
        await run([
            "inspect",
            "workbench",
            "load.configure",
            JSON.stringify({
                world_region_side: pixel(args[0], "world region side"),
                active_center_x: pixel(args[1] ?? "64", "active center x"),
                active_center_z: pixel(args[2] ?? "64", "active center z"),
                active_radius: pixel(args[3] ?? "2", "active radius"),
            }),
            "--format",
            "json",
        ]);
        break;
    }
    case "resident":
        if (args.length > 0) fail("workbench: resident does not accept arguments");
        await run(["inspect", "workbench", "resident.status", "--format", "json"]);
        break;
    case "resident-stream": {
        if (args.length !== 2) fail("workbench: resident-stream requires center x and center z");
        await run([
            "inspect",
            "workbench",
            "resident.stream",
            JSON.stringify({
                world_region_side: 128,
                active_center_x: pixel(args[0], "active center x"),
                active_center_z: pixel(args[1], "active center z"),
                active_radius: 2,
            }),
            "--format",
            "json",
        ]);
        break;
    }
    case "async":
        if (args.length > 0) fail("workbench: async does not accept arguments");
        await run(["inspect", "workbench", "async.status", "--format", "json"]);
        break;
    case "async-schedule": {
        if (args.length !== 2) fail("workbench: async-schedule requires center x and center z");
        await run([
            "inspect",
            "workbench",
            "async.schedule",
            JSON.stringify({
                world_region_side: 128,
                active_center_x: pixel(args[0], "active center x"),
                active_center_z: pixel(args[1], "active center z"),
                active_radius: 2,
            }),
            "--format",
            "json",
        ]);
        break;
    }
    case "async-gate-arm":
    case "async-gate-release":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            verb === "async-gate-arm" ? "async.gate.arm" : "async.gate.release",
            "--format",
            "json",
        ]);
        break;
    case "cooked":
        if (args.length > 0) fail("workbench: cooked does not accept arguments");
        await run(["inspect", "workbench", "cooked.status", "--format", "json"]);
        break;
    case "cooked-open":
        if (args.length !== 1) fail("workbench: cooked-open requires a repository-relative pack");
        await run([
            "inspect",
            "workbench",
            "cooked.open",
            JSON.stringify({ path: args[0] }),
            "--format",
            "json",
        ]);
        break;
    case "cooked-schedule": {
        if (args.length !== 2) fail("workbench: cooked-schedule requires center x and center z");
        await run([
            "inspect",
            "workbench",
            "cooked.schedule",
            JSON.stringify({
                world_region_side: 128,
                active_center_x: pixel(args[0], "active center x"),
                active_center_z: pixel(args[1], "active center z"),
                active_radius: 2,
            }),
            "--format",
            "json",
        ]);
        break;
    }
    case "cooked-gate-arm":
    case "cooked-gate-release":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            verb === "cooked-gate-arm" ? "cooked.gate.arm" : "cooked.gate.release",
            "--format",
            "json",
        ]);
        break;
    case "composition":
        if (args.length > 0) fail("workbench: composition does not accept arguments");
        await run(["inspect", "workbench", "composition.status", "--format", "json"]);
        break;
    case "composition-schedule": {
        if (args.length !== 2) {
            fail("workbench: composition-schedule requires center x and center z");
        }
        await run([
            "inspect",
            "workbench",
            "composition.schedule",
            JSON.stringify({
                world_region_side: 128,
                active_center_x: pixel(args[0], "active center x"),
                active_center_z: pixel(args[1], "active center z"),
                active_radius: 2,
            }),
            "--format",
            "json",
        ]);
        break;
    }
    case "composition-enable":
    case "composition-disable":
    case "composition-traversal-enable":
    case "composition-traversal-disable":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            verb.replaceAll("-", "."),
            "--format",
            "json",
        ]);
        break;
    case "composition-order": {
        if (args.length !== 1 || !["terrain-first", "object-first"].includes(args[0])) {
            fail("workbench: composition-order requires terrain-first or object-first");
        }
        await run([
            "inspect",
            "workbench",
            "composition.order",
            JSON.stringify({ order: args[0] }),
            "--format",
            "json",
        ]);
        break;
    }
    case "composition-fixture": {
        if (args.length !== 1 || !["cell-center", "arbitrary-q8"].includes(args[0])) {
            fail("workbench: composition-fixture requires cell-center or arbitrary-q8");
        }
        await run([
            "inspect",
            "workbench",
            "composition.fixture",
            JSON.stringify({ fixture: args[0] }),
            "--format",
            "json",
        ]);
        break;
    }
    case "meshlet":
        if (args.length > 0) fail("workbench: meshlet does not accept arguments");
        await run(["inspect", "workbench", "meshlet.status", "--format", "json"]);
        break;
    case "meshlet-enable":
    case "meshlet-disable":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            verb === "meshlet-enable" ? "meshlet.enable" : "meshlet.disable",
            "--format",
            "json",
        ]);
        break;
    case "meshlet-config": {
        if (args.length > 2) fail("workbench: meshlet-config accepts mask and forced LOD");
        const forced = args[1] ?? "auto";
        const forcedLod = forced === "auto" ? null : pixel(forced, "forced LOD");
        await run([
            "inspect",
            "workbench",
            "meshlet.configure",
            JSON.stringify({
                archetype_mask: pixel(args[0] ?? "255", "archetype mask"),
                forced_lod: forcedLod,
            }),
            "--format",
            "json",
        ]);
        break;
    }
    case "skeletal":
        if (args.length > 0) fail("workbench: skeletal does not accept arguments");
        await run(["inspect", "workbench", "skeletal.status", "--format", "json"]);
        break;
    case "skeletal-enable":
    case "skeletal-disable":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            verb === "skeletal-enable" ? "skeletal.enable" : "skeletal.disable",
            "--format",
            "json",
        ]);
        break;
    case "skeletal-config": {
        await configureSkeletal(args);
        break;
    }
    case "surface":
        if (args.length > 0) fail("workbench: surface does not accept arguments");
        await run(["inspect", "workbench", "surface.status", "--format", "json"]);
        break;
    case "surface-enable":
    case "surface-disable":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            verb === "surface-enable" ? "surface.enable" : "surface.disable",
            "--format",
            "json",
        ]);
        break;
    case "surface-config": {
        await configureSurface(args);
        break;
    }
    case "occlusion-enable":
    case "occlusion-disable":
    case "occlusion-reset":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            `surface.occlusion.${verb.replace("occlusion-", "")}`,
            "--format",
            "json",
        ]);
        break;
    case "camera-set": {
        if (args.length !== 6 && args.length !== 7) {
            fail("workbench: camera-set requires px py pz tx ty tz and optional vertical FOV");
        }
        const values = args.map((value, index) => finite(value, `camera value ${index + 1}`));
        await run([
            "inspect",
            "workbench",
            "camera.set_pose",
            JSON.stringify({
                position: values.slice(0, 3),
                target: values.slice(3, 6),
                vertical_fov_degrees: values[6] ?? 60,
            }),
            "--format",
            "json",
        ]);
        break;
    }
    case "pause":
    case "resume":
        if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
        await run(["inspect", "workbench", `workbench.${verb}`, "--format", "json"]);
        break;
    case "color": {
        if (args.length !== 3 && args.length !== 4) {
            fail("workbench: color requires r g b and optional alpha channels");
        }
        const rgba = [
            channel(args[0], "red"),
            channel(args[1], "green"),
            channel(args[2], "blue"),
            channel(args[3] ?? "1", "alpha"),
        ];
        await run([
            "inspect",
            "workbench",
            "workbench.set_clear_color",
            JSON.stringify({ rgba }),
            "--format",
            "json",
        ]);
        break;
    }
    case "capture": {
        if (args.length > 1) fail("workbench: capture accepts at most one capture id");
        const id = args[0] ?? "capture";
        await run([
            "inspect",
            "workbench",
            "workbench.capture",
            JSON.stringify({ id, collection: "operator" }),
            "--format",
            "json",
        ]);
        break;
    }
    case "perception":
    case "perception-region": {
        await run([
            "inspect",
            "workbench",
            "perception.capture",
            JSON.stringify(perceptionPayload(verb, args)),
            "--format",
            "json",
        ]);
        break;
    }
    default:
        fail(`workbench: expected a command, received ${verb ?? "nothing"}`);
}
