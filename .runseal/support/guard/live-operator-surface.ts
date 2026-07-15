type Fail = (message: string) => never;

const CURRENT_BOUNDARY_TARGET = "../../AGENTS.md#4-current-runtime-boundary";

const DIRECT_PROTOTYPE_SIDECAR = [
    /\bsidecar\s+(?:start|restart|stop|status)\b[^\r\n`]*sidecar\.prototype\.toml/i,
    /\bsidecar\b[^\r\n`]*sidecar\.prototype\.toml[^\r\n`]*\b(?:start|restart|stop|status)\b/i,
];

export async function requireLiveOperatorSurface(root: string, fail: Fail): Promise<void> {
    console.log("==> live operator surface");
    await requireWrapperSet(root, fail);

    const model = await Deno.readTextFile(`${root}/docs/architecture/repository-model.md`);
    if (/^## State\r?$/m.test(model) || /Experiments?\s+through\s+\d{4}/i.test(model)) {
        fail("guard: repository model contains a stage-specific current-state ledger");
    }
    if (
        !model.includes("## Current boundary authority") ||
        !model.includes(CURRENT_BOUNDARY_TARGET)
    ) {
        fail("guard: repository model does not name the current runtime boundary authority");
    }

    for (const path of ["README.md", "AGENTS.md"]) {
        const source = await Deno.readTextFile(`${root}/${path}`);
        if (!source.includes("runseal :prototype start")) {
            fail(`guard: ${path} does not retain the maintained prototype operator`);
        }
        if (DIRECT_PROTOTYPE_SIDECAR.some((pattern) => pattern.test(source))) {
            fail(`guard: ${path} exposes a direct prototype Sidecar lifecycle command`);
        }
    }
}

async function requireWrapperSet(root: string, fail: Fail): Promise<void> {
    const names: string[] = [];
    for await (const entry of Deno.readDir(`${root}/.runseal/wrappers`)) {
        if (entry.isFile) names.push(entry.name);
    }
    names.sort();
    const expected = [
        "canonical-actor.ts",
        "canonical-frame.ts",
        "canonical-prototype.ts",
        "canonical-resources.ts",
        "canonical-runtime.ts",
        "gpu-lab.ts",
        "guard.ts",
        "init.ts",
        "prototype.ts",
        "workbench.ts",
    ];
    if (JSON.stringify(names) !== JSON.stringify(expected)) {
        fail(`guard: Runseal wrapper set diverged: ${JSON.stringify(names)}`);
    }
}
