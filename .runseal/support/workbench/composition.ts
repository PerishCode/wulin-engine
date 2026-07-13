type RunSidecar = (args: string[]) => Promise<void>;

function fail(message: string): never {
    console.error(message);
    Deno.exit(1);
}

function unsigned(value: string, name: string): number {
    const parsed = Number(value);
    if (!Number.isSafeInteger(parsed) || parsed < 0) {
        fail(`workbench: ${name} must be a non-negative integer`);
    }
    return parsed;
}

function signed(value: string, name: string): number {
    const parsed = Number(value);
    if (!Number.isSafeInteger(parsed)) fail(`workbench: ${name} must be a safe integer`);
    return parsed;
}

export async function dispatchComposition(
    verb: string | undefined,
    args: string[],
    run: RunSidecar,
): Promise<boolean> {
    switch (verb) {
        case "composition":
            if (args.length > 0) fail("workbench: composition does not accept arguments");
            await run(["inspect", "workbench", "composition.status", "--format", "json"]);
            return true;
        case "composition-schedule":
            if (args.length !== 2) {
                fail("workbench: composition-schedule requires center x and center z");
            }
            await run([
                "inspect",
                "workbench",
                "composition.schedule",
                JSON.stringify({
                    world_region_side: 128,
                    active_center_x: unsigned(args[0], "active center x"),
                    active_center_z: unsigned(args[1], "active center z"),
                    active_radius: 2,
                }),
                "--format",
                "json",
            ]);
            return true;
        case "composition-global-schedule":
            if (args.length < 4 || args.length > 5) {
                fail(
                    "workbench: composition-global-schedule requires origin x, origin z, center x, center z, and optional radius",
                );
            }
            await run([
                "inspect",
                "workbench",
                "composition.global.schedule",
                JSON.stringify({
                    origin_x: signed(args[0], "global origin x"),
                    origin_z: signed(args[1], "global origin z"),
                    center_x: signed(args[2], "global center x"),
                    center_z: signed(args[3], "global center z"),
                    active_radius: unsigned(args[4] ?? "2", "active radius"),
                }),
                "--format",
                "json",
            ]);
            return true;
        case "composition-enable":
        case "composition-disable":
        case "composition-traversal-enable":
        case "composition-traversal-disable":
        case "composition-prefetch-enable":
        case "composition-prefetch-disable":
            if (args.length > 0) fail(`workbench: ${verb} does not accept arguments`);
            await run(["inspect", "workbench", verb.replaceAll("-", "."), "--format", "json"]);
            return true;
        case "composition-order":
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
            return true;
        case "composition-fixture":
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
            return true;
        default:
            return false;
    }
}
