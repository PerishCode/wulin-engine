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
        "SimulationProbe|SimulationStatus",
        "SimulationAdvancePayload",
        "CanonicalTerrainBodyRetained(Advance|Batch)",
        "RetainedTerrainBodyAdvance",
        "pub fn (advance_simulation|simulation_status|simulation_schedule_probe|advance_retained_terrain_body|advance_retained_body_batch)\\(",
        "(^|[^a-z_])simulation_(advance|probe)_failed",
        "retained_terrain_(advance|batch)_failed",
        "simulation\\.(advance|probe|status)",
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
        "retiredControlGate",
        "retiredControls",
        "retiredStatusGate",
        "retiredStatus",
        "simulation\\.(advance|probe|status)",
        "canonical\\.terrain\\.body\\.(spawn|read|despawn|retained\\.(advance|batch))",
        "simulation\\.terrain\\.body\\.advance",
    ].join("|");
    await requireAbsent(
        root,
        fail,
        gatePattern,
        [".runseal/wrappers", ".runseal/support/actor", ".runseal/support/prototype/host.ts"],
        "independent simulation gate",
    );

    const actorAdmission = await Deno.readTextFile(
        `${root}/.runseal/support/actor/admission.ts`,
    );
    for (
        const retired of [
            "retiredShape",
            "retiredPayload",
            "aliasPayload",
            "initial_velocity_delta_q16",
        ]
    ) {
        if (actorAdmission.includes(retired)) {
            fail(`guard: retired actor velocity compatibility probe returned: ${retired}`);
        }
    }
    if (
        !actorAdmission.includes(
            "initial_step_velocity_delta_q16: initialVelocityDeltaQ16",
        ) ||
        !actorAdmission.includes("initial_step_velocity_delta_q16: 4_096") ||
        !actorAdmission.includes("shared-window actor initial velocity delta ordering diverged") ||
        !actorAdmission.includes("pending-window actor rollback")
    ) fail("guard: current actor velocity admission authority diverged");
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
