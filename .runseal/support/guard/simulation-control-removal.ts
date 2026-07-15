type Fail = (message: string) => never;

export async function requireSimulationHistoryRemoved(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> removed independent simulation mutation surface");
    for (
        const path of [
            ".runseal/support/simulation-schedule.ts",
            ".runseal/support/terrain/retained-advance.ts",
            ".runseal/support/terrain/retained-batch.ts",
        ]
    ) {
        try {
            await Deno.stat(`${root}/${path}`);
            fail(`guard: removed simulation support remains: ${path}`);
        } catch (error) {
            if (!(error instanceof Deno.errors.NotFound)) throw error;
        }
    }

    const livePattern = [
        "SimulationProbe",
        "SimulationAdvancePayload",
        "CanonicalTerrainBodyRetained(Advance|Batch)",
        "RetainedTerrainBodyAdvance",
        "pub fn (advance_simulation|simulation_schedule_probe|advance_retained_terrain_body|advance_retained_body_batch)\\(",
        "simulation_(advance|probe)_failed",
        "retained_terrain_(advance|batch)_failed",
        "simulation\\.(advance|probe)",
        "canonical\\.terrain\\.body\\.retained\\.(advance|batch)",
    ].join("|");
    await requireAbsent(
        root,
        fail,
        livePattern,
        ["apps", "crates/engine-runtime/src"],
        "independent simulation mutation symbol",
    );

    const gatePattern = [
        "simulationScheduleGates",
        "retainedAdvanceGates",
        "retainedBatchGates",
    ].join("|");
    await requireAbsent(
        root,
        fail,
        gatePattern,
        [".runseal/wrappers", ".runseal/support/terrain"],
        "independent simulation gate",
    );
}

async function requireAbsent(
    root: string,
    fail: Fail,
    pattern: string,
    paths: string[],
    label: string,
): Promise<void> {
    const output = await new Deno.Command("git", {
        args: ["grep", "--no-index", "-n", "-E", pattern, "--", ...paths],
        cwd: root,
        stdout: "piped",
        stderr: "inherit",
    }).output();
    if (output.code === 0) {
        fail(
            `guard: removed ${label} found\n${new TextDecoder().decode(output.stdout)}`,
        );
    }
    if (output.code !== 1) {
        fail(`guard: removed ${label} scan failed with exit code ${output.code}`);
    }
}
