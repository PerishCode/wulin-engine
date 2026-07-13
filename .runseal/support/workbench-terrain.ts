type SidecarRun = (args: string[]) => Promise<void>;

function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

function integer(value: string, name: string): number {
    const parsed = Number(value);
    if (!Number.isSafeInteger(parsed) || parsed < 0) {
        fail(`workbench: ${name} must be a non-negative integer`);
    }
    return parsed;
}

export async function dispatchTerrain(
    verb: string | undefined,
    args: string[],
    run: SidecarRun,
): Promise<boolean> {
    switch (verb) {
        case "terrain":
            if (args.length > 0) fail("workbench: terrain does not accept arguments");
            await run(["inspect", "workbench", "terrain.status", "--format", "json"]);
            return true;
        case "terrain-open":
            if (args.length !== 1) {
                fail("workbench: terrain-open requires a repository-relative pack");
            }
            await run([
                "inspect",
                "workbench",
                "terrain.open",
                JSON.stringify({ path: args[0] }),
                "--format",
                "json",
            ]);
            return true;
        case "terrain-schedule":
            if (args.length < 2 || args.length > 3) {
                fail(
                    "workbench: terrain-schedule requires center x, center z, and optional radius",
                );
            }
            await run([
                "inspect",
                "workbench",
                "terrain.schedule",
                JSON.stringify({
                    world_region_side: 128,
                    active_center_x: integer(args[0], "active center x"),
                    active_center_z: integer(args[1], "active center z"),
                    active_radius: integer(args[2] ?? "2", "active radius"),
                }),
                "--format",
                "json",
            ]);
            return true;
        case "terrain-enable":
        case "terrain-disable":
            if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
            await run([
                "inspect",
                "workbench",
                verb === "terrain-enable" ? "terrain.enable" : "terrain.disable",
                "--format",
                "json",
            ]);
            return true;
        case "terrain-lod":
            if (args.length > 0) fail("workbench: terrain-lod does not accept arguments");
            await run(["inspect", "workbench", "terrain.lod.status", "--format", "json"]);
            return true;
        case "terrain-lod-config": {
            if (args.length < 2 || args.length > 3) {
                fail(
                    "workbench: terrain-lod-config requires near radius, middle radius, and optional forced LOD",
                );
            }
            const forced = args[2] ?? "auto";
            await run([
                "inspect",
                "workbench",
                "terrain.lod.configure",
                JSON.stringify({
                    near_patch_radius: integer(args[0], "near patch radius"),
                    middle_patch_radius: integer(args[1], "middle patch radius"),
                    forced_lod: forced === "auto" ? null : integer(forced, "forced LOD"),
                }),
                "--format",
                "json",
            ]);
            return true;
        }
        case "terrain-lod-enable":
        case "terrain-lod-disable":
            if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
            await run([
                "inspect",
                "workbench",
                verb === "terrain-lod-enable" ? "terrain.lod.enable" : "terrain.lod.disable",
                "--format",
                "json",
            ]);
            return true;
        case "terrain-io-gate-arm":
        case "terrain-io-gate-release":
        case "terrain-copy-gate-arm":
        case "terrain-copy-gate-release": {
            if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
            const [kind, action] = verb.replace("terrain-", "").split("-gate-");
            await run([
                "inspect",
                "workbench",
                `terrain.${kind}_gate.${action}`,
                "--format",
                "json",
            ]);
            return true;
        }
        default:
            return false;
    }
}
