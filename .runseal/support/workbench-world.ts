type Run = (args: string[]) => Promise<void>;

export async function dispatchWorld(
    verb: string | undefined,
    args: string[],
    run: Run,
): Promise<boolean> {
    if (!verb?.startsWith("world")) return false;
    if (["world", "world-reset", "world-probe"].includes(verb)) {
        if (args.length > 0) fail(`${verb} does not accept arguments`);
        await run([
            "inspect",
            "workbench",
            verb === "world" ? "world.status" : verb.replace("-", "."),
            "--format",
            "json",
        ]);
        return true;
    }
    if (verb === "world-relocate" || verb === "world-rebase") {
        if (args.length !== 2) fail(`${verb} requires region x and region z`);
        await run([
            "inspect",
            "workbench",
            verb.replace("-", "."),
            JSON.stringify({
                region_x: signedRegion(args[0], "region x"),
                region_z: signedRegion(args[1], "region z"),
            }),
            "--format",
            "json",
        ]);
        return true;
    }
    return false;
}

function signedRegion(value: string, name: string): number {
    const parsed = Number(value);
    if (!Number.isSafeInteger(parsed)) fail(`${name} must be a signed safe integer`);
    return parsed;
}

function fail(message: string): never {
    throw new Error(`workbench: ${message}`);
}
