function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) fail("workbench: RUNSEAL_PROFILE_PATH is not set");
const root = profilePath.replace(/[\\/][^\\/]+$/, "");

async function sidecar(args: string[]): Promise<void> {
    const status = await new Deno.Command("sidecar", {
        args: [...args, "--config", "sidecar.toml"],
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
    }).spawn().status;
    if (!status.success) fail(`workbench: sidecar failed with exit code ${status.code}`);
}

async function event(verb: string, payload: unknown = {}): Promise<void> {
    await sidecar([
        "inspect",
        "workbench",
        verb,
        JSON.stringify(payload),
        "--format",
        "json",
    ]);
}

function integer(value: string | undefined, name: string): number {
    const parsed = Number(value);
    if (!Number.isSafeInteger(parsed)) fail(`workbench: ${name} must be a safe integer`);
    return parsed;
}

function finite(value: string | undefined, name: string): number {
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) fail(`workbench: ${name} must be finite`);
    return parsed;
}

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log(
        "Usage: runseal :workbench <start|restart|stop|status|inspect|pause|resume|input|input-record-start|input-record-stop|input-replay|terrain-open|objects-open|schedule|canonical-status|probe|traversal-enable|traversal-disable|prefetch-enable|prefetch-disable|camera|camera-set|camera-reset|capture|perception|objects-io-arm|objects-io-release|objects-copy-arm|objects-copy-release|terrain-io-arm|terrain-io-release|terrain-copy-arm|terrain-copy-release>",
    );
    Deno.exit(0);
}

const [verb, ...args] = Deno.args;
switch (verb) {
    case "start":
    case "restart":
    case "stop":
        if (args.length !== 0) fail(`workbench: ${verb} accepts no arguments`);
        await sidecar([verb]);
        break;
    case "status":
        if (args.length !== 0) fail("workbench: status accepts no arguments");
        await sidecar(["status", "--format", "json"]);
        break;
    case "inspect":
        if (args.length !== 0) fail("workbench: inspect accepts no arguments");
        await event("workbench.status");
        break;
    case "pause":
    case "resume":
        if (args.length !== 0) fail(`workbench: ${verb} accepts no arguments`);
        await event(`workbench.${verb}`);
        break;
    case "input":
        if (args.length !== 0) fail("workbench: input accepts no arguments");
        await event("input.status");
        break;
    case "input-record-start":
    case "input-record-stop":
        if (args.length !== 0) fail(`workbench: ${verb} accepts no arguments`);
        await event(`input.record.${verb.endsWith("start") ? "start" : "stop"}`);
        break;
    case "input-replay":
        if (args.length !== 0) fail("workbench: input-replay accepts no arguments");
        await event("input.replay");
        break;
    case "terrain-open":
    case "objects-open":
        if (args.length !== 1) fail(`workbench: ${verb} requires a pack path`);
        await event(verb === "terrain-open" ? "source.terrain.open" : "source.objects.open", {
            path: args[0],
        });
        break;
    case "schedule":
        if (args.length !== 5) {
            fail("workbench: schedule requires origin-x origin-z center-x center-z radius");
        }
        await event("canonical.schedule", {
            origin_x: integer(args[0], "origin x"),
            origin_z: integer(args[1], "origin z"),
            center_x: integer(args[2], "center x"),
            center_z: integer(args[3], "center z"),
            active_radius: integer(args[4], "active radius"),
        });
        break;
    case "canonical-status":
        if (args.length !== 0) fail("workbench: canonical-status accepts no arguments");
        await event("canonical.status");
        break;
    case "probe":
        if (args.length !== 0) fail("workbench: probe accepts no arguments");
        await event("canonical.probe");
        break;
    case "traversal-enable":
    case "traversal-disable":
    case "prefetch-enable":
    case "prefetch-disable":
        if (args.length !== 0) fail(`workbench: ${verb} accepts no arguments`);
        await event(`canonical.${verb.replace("-", ".")}`);
        break;
    case "camera":
    case "camera-reset":
        if (args.length !== 0) fail(`workbench: ${verb} accepts no arguments`);
        await event(verb === "camera" ? "camera.status" : "camera.reset");
        break;
    case "camera-set":
        if (args.length !== 6 && args.length !== 7) {
            fail("workbench: camera-set requires px py pz tx ty tz [vertical-fov]");
        }
        await event("camera.set_pose", {
            position: args.slice(0, 3).map((value, index) => finite(value, `position ${index}`)),
            target: args.slice(3, 6).map((value, index) => finite(value, `target ${index}`)),
            vertical_fov_degrees: finite(args[6] ?? "60", "vertical FOV"),
        });
        break;
    case "capture":
        if (args.length !== 1) fail("workbench: capture requires an ID");
        await event("workbench.capture", { id: args[0], collection: "operator" });
        break;
    case "perception":
        if (args.length !== 1) fail("workbench: perception requires an ID");
        await event("perception.capture", {
            id: args[0],
            collection: "operator",
            samples: [{ x: 0, y: 0 }, { x: 640, y: 360 }],
        });
        break;
    case "objects-io-arm":
    case "objects-io-release":
    case "objects-copy-arm":
    case "objects-copy-release":
    case "terrain-io-arm":
    case "terrain-io-release":
    case "terrain-copy-arm":
    case "terrain-copy-release": {
        if (args.length !== 0) fail(`workbench: ${verb} accepts no arguments`);
        const [domain, stage, action] = verb.split("-");
        await event(`canonical.${domain}.${stage}_gate.${action}`);
        break;
    }
    default:
        fail(`workbench: unknown command ${verb ?? "<missing>"}`);
}
