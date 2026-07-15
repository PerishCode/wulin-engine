type Fail = (message: string) => never;

export async function requireSimulationHistoryRemoved(
    root: string,
    fail: Fail,
): Promise<void> {
    console.log("==> removed independent simulation mutation surface");
    for (
        const path of [
            ".runseal/support/simulation-schedule.ts",
            ".runseal/support/simulation-body.ts",
            ".runseal/support/terrain/retained-body.ts",
            ".runseal/support/terrain/retained-advance.ts",
            ".runseal/support/terrain/retained-batch.ts",
            ".runseal/support/actor.ts",
            ".runseal/support/actor-projection.ts",
            ".runseal/support/actor/projection.ts",
            ".runseal/support/simulation-actor.ts",
            "apps/prototype/src/body.rs",
            "apps/workbench/src/inspect/app/retained_body.rs",
            "crates/engine-runtime/src/runtime/retained_batch.rs",
            "crates/engine-runtime/src/runtime/retained_body.rs",
            "crates/engine-runtime/src/runtime/simulation_body.rs",
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
        "(^|[^a-z_])simulation_(advance|probe)_failed",
        "retained_terrain_(advance|batch)_failed",
        "simulation\\.(advance|probe)",
        "canonical\\.terrain\\.body\\.retained\\.(advance|batch)",
        "TerrainBodyHandle|TerrainBodySlot|RetainedTerrainBody(Batch)?|RetainedSimulationAdvance",
        "(spawn|read|despawn)_terrain_body|advance_simulation_body",
        "CanonicalTerrainBody(Spawn|Read|Despawn)|SimulationTerrainBodyAdvance",
        "canonical\\.terrain\\.body\\.(spawn|read|despawn)|simulation\\.terrain\\.body\\.advance",
        "pub fn project_actor\\(",
        "actor\\.project",
        "ActorProject",
        "(^|[^A-Za-z0-9_])actor_project([^A-Za-z0-9_]|$)",
        "pub use rendering::\\{[^}]*ActorRenderProjection",
    ].join("|");
    await requireAbsent(
        root,
        fail,
        livePattern,
        ["apps", "crates/engine-runtime/src", ".runseal/wrappers/init.ts", "flavor.toml"],
        "independent simulation mutation symbol",
    );

    const gatePattern = [
        "simulationScheduleGates",
        "retainedAdvanceGates",
        "retainedBatchGates",
        "retainedBodyGates",
        "simulationBodyGates",
    ].join("|");
    await requireAbsent(
        root,
        fail,
        gatePattern,
        [
            ".runseal/wrappers",
            ".runseal/support/actor/lifecycle.ts",
            ".runseal/support/actor/simulation.ts",
            ".runseal/support/prototype-host.ts",
        ],
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
