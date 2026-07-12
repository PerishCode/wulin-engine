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

if (Deno.args.includes("--help") || Deno.args.includes("-h")) {
    console.log(
        "Usage: runseal :workbench <start|status|inspect|capture|color|camera|camera-set|camera-reset|scene|pause|resume|restart|stop>",
    );
    console.log("");
    console.log("Control and inspect the native engine workbench through Sidecar.");
    Deno.exit(0);
}

const profilePath = Deno.env.get("RUNSEAL_PROFILE_PATH");
if (!profilePath) {
    fail("workbench: RUNSEAL_PROFILE_PATH is not set");
}
const root = profilePath.replace(/[\\/][^\\/]+$/, "");
const [verb, ...args] = Deno.args;

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
    default:
        fail(`workbench: expected a command, received ${verb ?? "nothing"}`);
}
