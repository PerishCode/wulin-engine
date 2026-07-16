mod actor;
mod boundary;
mod camera;
mod interaction;
mod jump;
mod locomotion;
mod observation;
mod presentation;
mod time;

use anyhow::{Context, Result};
use engine_runtime::{
    ActorSimulationAdvance, ActorSimulationCommand, FrameRequest, GlobalRegionConfig,
    ObjectTargetFeedback, ObjectTargetFeedbackKind, Runtime, RuntimeActor, TerrainPosition,
};
use reference_host::{HostClock, HostClockStatus, HostElapsedSample, HostInput, bootstrap, window};
use serde_json::{Value, json};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const ESCAPE: u8 = 0x1B;
const SPACE: u8 = 0x20;
const OBSERVE_OBJECT: u8 = 0x46;
const ACTIVATE_OBJECT: u8 = 0x0D;
const CLEAR_COLOR: [f32; 4] = [0.035, 0.105, 0.14, 1.0];
const WINDOW_CONFIG: window::Config = window::Config {
    class_name: "WulinEnginePrototypeWindow",
    title: "Wulin Engine Prototype",
    width: WIDTH,
    height: HEIGHT,
};

fn main() {
    if let Err(error) = unsafe { run() } {
        eprintln!("prototype failed: {error:#}");
        std::process::exit(1);
    }
}

unsafe fn run() -> Result<()> {
    let arguments = bootstrap::Arguments::parse()?;
    let plan = arguments
        .bootstrap
        .context("prototype requires --bootstrap=<path>")?;
    let hwnd = window::create(WINDOW_CONFIG)?;
    let mut runtime = unsafe { Runtime::new(hwnd, WIDTH, HEIGHT)? };
    let mut input = HostInput::new();
    let ready = unsafe { bootstrap::drive(&mut runtime, &mut input, &plan, CLEAR_COLOR)? };
    let runtime_actor = spawn_initial_actor(&mut runtime, plan.global_config())?;
    let playable_region_bounds = plan.playable_region_bounds();
    runtime
        .enable_composition_traversal()
        .context("prototype composition traversal activation failed")?;
    let mut clock = HostClock::new();
    let mut startup = Some(ready.status);
    let bootstrap_frame_count = ready.frame_count;
    let mut live_frame_count = 0_u64;
    let mut camera_anchor_count = 0_u64;
    let mut render_block_count = 0_u64;
    let mut object_target_frame_count = 0_u64;
    let mut object_action_frame_count = 0_u64;
    let mut presentation_policy = presentation::Policy::new();
    let mut camera_policy = camera::Policy::new();
    let mut jump_policy = jump::Policy::new();
    let mut observation_policy = observation::Policy::new();
    let mut interaction_policy = interaction::Policy::new();
    unsafe { window::show(hwnd) };

    'running: loop {
        if !window::pump_messages() {
            break 'running;
        }
        input.ingest(window::drain_input());
        if input.was_pressed(ESCAPE) {
            window::request_close(hwnd)?;
            continue;
        }
        let sample = clock.sample(&window::drain_activation())?;
        jump_policy.observe_sample(sample);
        observation_policy.observe_sample(sample);
        interaction_policy.observe_sample(sample);
        jump_policy.ingest(input.was_pressed(SPACE));
        observation_policy.ingest(input.was_pressed(OBSERVE_OBJECT));
        interaction_policy.ingest(input.was_pressed(ACTIVATE_OBJECT));
        let camera_candidate = camera_policy.candidate(&input);
        let requested_locomotion = locomotion::command(&input, camera_candidate.rig().orbit_index);
        let actor = runtime
            .read_actor(runtime_actor.handle)
            .context("prototype current actor read failed")?;
        let locomotion = boundary::admit(
            actor.motion.body().position(),
            playable_region_bounds,
            requested_locomotion,
        )
        .context("prototype playable boundary admission failed")?;
        let command = ActorSimulationCommand {
            delta_x_q9: locomotion.delta_x_q9,
            delta_z_q9: locomotion.delta_z_q9,
            step_up_limit_q16: locomotion.step_up_limit_q16,
            initial_step_velocity_delta_q16: jump_policy.initial_velocity_delta_q16(),
            step_acceleration_q16: actor::GRAVITY_STEP_ACCELERATION_Q16,
            presentation: presentation_policy.command(locomotion),
        };
        let outcome = time::admitted_elapsed(sample)
            .map(|elapsed_nanoseconds| {
                runtime.advance_simulation_actor(runtime_actor.handle, elapsed_nanoseconds, command)
            })
            .transpose()?;
        if let Some(outcome) = outcome {
            jump_policy.observe_outcome(outcome)?;
        }
        let advance = outcome
            .map(|outcome| time::consume_actor_outcome(outcome, &mut render_block_count))
            .transpose()?
            .flatten();
        if let Some(advance) = &advance {
            presentation_policy.observe_advance(
                advance.simulation.step_count,
                advance.actor.output.presentation,
            );
        }
        let object_observation = if let Some(advance) = &advance
            && observation_policy.wants_completion(advance.simulation.step_count)
        {
            let origin = advance.actor.output.motion.body().position();
            let snapshot = runtime
                .canonical_object_snapshot()
                .context("prototype object observation snapshot failed")?;
            let query = runtime
                .query_nearest_canonical_object(origin, observation::OBJECT_OBSERVATION_RADIUS_Q9)
                .context("prototype committed object observation failed")?;
            observation_policy
                .complete_after_advance(advance.simulation.step_count, origin, snapshot, query)
                .context("prototype object target admission failed")?
        } else {
            None
        };
        let interaction_attempt = if let Some(advance) = &advance
            && interaction_policy.wants_completion(advance.simulation.step_count)
        {
            let target = observation_policy.status().target;
            let interaction_target = target.map(|target| interaction::Target {
                identity: target.identity,
                available: target.availability == observation::Availability::Resolved,
            });
            let resolution = match target {
                Some(target) if target.availability == observation::Availability::Resolved => Some(
                    runtime
                        .resolve_canonical_object(target.identity)
                        .context("prototype object action resolution failed")?,
                ),
                _ => None,
            };
            interaction_policy
                .prepare_after_advance(
                    advance.simulation.step_count,
                    advance.actor.output.motion.body().position(),
                    interaction_target,
                    resolution,
                )
                .context("prototype object action eligibility failed")?
        } else {
            None
        };
        let completed = advance
            .filter(|advance| advance.simulation.step_count != 0)
            .map(|advance| {
                (
                    sample,
                    clock.status(),
                    advance,
                    command,
                    object_observation,
                    interaction_attempt,
                )
            });
        let anchored_rig = camera_candidate.rig();
        runtime.set_actor_relative_camera(
            runtime_actor.handle,
            anchored_rig.position_offset,
            anchored_rig.target_offset,
            anchored_rig.vertical_fov_degrees,
        )?;
        camera_policy.commit(camera_candidate);
        let anchored_camera = runtime.camera_json();
        camera_anchor_count = camera_anchor_count
            .checked_add(1)
            .context("prototype camera anchor count overflowed")?;
        let object_target = observation_policy.target_identity();
        let object_target_feedback =
            interaction_policy.frame_feedback(object_target, interaction_attempt);
        let render_outcome = unsafe {
            runtime.frame(FrameRequest {
                clear_color: CLEAR_COLOR,
                capture: false,
                capture_object_ids: false,
                probe: false,
                object_target_feedback,
            })?
        };
        let interaction_completion = interaction_policy
            .complete_frame(
                interaction_attempt,
                object_target_feedback,
                render_outcome.object_target_feedback,
            )
            .context("prototype object action frame commit failed")?;
        if object_target.is_some() {
            object_target_frame_count = object_target_frame_count
                .checked_add(1)
                .context("prototype object target frame count overflowed")?;
        }
        if render_outcome
            .object_target_feedback
            .is_some_and(|feedback| feedback.kind == ObjectTargetFeedbackKind::Activated)
        {
            object_action_frame_count = object_action_frame_count
                .checked_add(1)
                .context("prototype object action frame count overflowed")?;
        }
        live_frame_count = live_frame_count
            .checked_add(1)
            .context("prototype live frame count overflowed")?;
        if observation_policy.has_target() {
            let snapshot = runtime
                .canonical_object_snapshot()
                .context("prototype object target snapshot check failed")?;
            if let Some(identity) = observation_policy.validation_request(snapshot) {
                let resolution = runtime
                    .resolve_canonical_object(identity)
                    .context("prototype retained object target resolution failed")?;
                observation_policy
                    .complete_validation(snapshot, resolution)
                    .context("prototype retained object target transition failed")?;
            }
        }
        interaction_policy.observe_target(observation_policy.status().target.map(|target| {
            interaction::Target {
                identity: target.identity,
                available: target.availability == observation::Availability::Resolved,
            }
        }));
        if let Some((sample, clock, advance, command, object_observation, interaction_attempt)) =
            completed
            && let Some(startup) = startup.take()
        {
            publish_readiness(ReadinessEvidence {
                startup,
                traversal: runtime.composition_status()["traversal"].clone(),
                sample,
                clock,
                advance,
                command,
                bootstrap_frame_count,
                live_frame_count,
                anchored_rig,
                anchored_camera,
                camera_anchor_count,
                render_block_count,
                object_target_frame_count,
                object_action_frame_count,
                submitted_object_feedback: object_target_feedback,
                rendered_object_feedback: render_outcome.object_target_feedback,
                jump_status: jump_policy.status(),
                object_observation,
                observation_status: observation_policy.status(),
                interaction_attempt,
                interaction_completion,
                interaction_status: interaction_policy.status(),
            })?;
        }
    }

    unsafe { runtime.wait_idle()? };
    window::teardown()?;
    Ok(())
}

struct ReadinessEvidence {
    startup: Value,
    traversal: Value,
    sample: HostElapsedSample,
    clock: HostClockStatus,
    advance: ActorSimulationAdvance,
    command: ActorSimulationCommand,
    bootstrap_frame_count: u64,
    live_frame_count: u64,
    anchored_rig: camera::Rig,
    anchored_camera: Value,
    camera_anchor_count: u64,
    render_block_count: u64,
    object_target_frame_count: u64,
    object_action_frame_count: u64,
    submitted_object_feedback: Option<ObjectTargetFeedback>,
    rendered_object_feedback: Option<ObjectTargetFeedback>,
    jump_status: jump::Status,
    object_observation: Option<observation::Completed>,
    observation_status: observation::Status,
    interaction_attempt: Option<interaction::Attempt>,
    interaction_completion: Option<interaction::FrameCompletion>,
    interaction_status: interaction::Status,
}

fn publish_readiness(evidence: ReadinessEvidence) -> Result<()> {
    let total_frame_count = evidence
        .bootstrap_frame_count
        .checked_add(evidence.live_frame_count)
        .context("prototype total frame count overflowed")?;
    let object_observation = evidence.object_observation.map(|completed| {
        json!({
            "origin": completed.origin,
            "snapshot": completed.snapshot,
            "query": completed.query,
        })
    });
    let object_target = evidence.observation_status.target.map(|target| {
        json!({
            "identity": target.identity,
            "snapshot": target.snapshot,
            "availability": match target.availability {
                observation::Availability::Resolved => "resolved",
                observation::Availability::OutsidePublishedWindow => "outside-published-window",
            },
        })
    });
    let object_action_attempt = evidence.interaction_attempt.map(|attempt| match attempt {
        interaction::Attempt::Eligible(eligible) => json!({
            "outcome": "eligible",
            "feedback": eligible.feedback,
            "proximity": eligible.proximity,
        }),
        interaction::Attempt::Ineligible(reason) => json!({
            "outcome": "ineligible",
            "reason": match reason {
                interaction::Ineligible::MissingTarget => "missing-target",
                interaction::Ineligible::UnavailableTarget => "unavailable-target",
                interaction::Ineligible::SourceReplaced => "source-replaced",
                interaction::Ineligible::OutsidePublishedWindow => "outside-published-window",
                interaction::Ineligible::OutsideRadius => "outside-radius",
            },
        }),
    });
    let acknowledgement = evidence
        .interaction_status
        .acknowledgement
        .map(|acknowledgement| {
            json!({
                "identity": acknowledgement.identity,
                "remainingFrames": acknowledgement.remaining_frames,
            })
        });
    println!(
        "{}",
        json!({
            "role": "prototype",
            "instance_id": std::process::id().to_string(),
            "startup": evidence.startup,
            "traversal": evidence.traversal,
            "actor": {
                "capacity": 1,
                "liveCount": 1,
                "state": evidence.advance.actor.output,
            },
            "simulation_driver": {
                "revision": "live-prototype-locomotion-driver-v8",
                "sample": evidence.sample,
                "clock": evidence.clock,
                "command": evidence.command,
                "advance": evidence.advance,
                "renderBlockCount": evidence.render_block_count,
                "bootstrapFrameCount": evidence.bootstrap_frame_count,
                "liveFrameCount": evidence.live_frame_count,
                "totalFrameCount": total_frame_count,
            },
            "camera_driver": {
                "revision": "live-prototype-actor-camera-v2",
                "actor": evidence.advance.actor.output.handle,
                "rig": {
                    "orbitIndex": evidence.anchored_rig.orbit_index,
                    "positionOffset": evidence.anchored_rig.position_offset,
                    "targetOffset": evidence.anchored_rig.target_offset,
                    "verticalFovDegrees": evidence.anchored_rig.vertical_fov_degrees,
                },
                "camera": evidence.anchored_camera,
                "anchorCount": evidence.camera_anchor_count,
                "liveFrameCount": evidence.live_frame_count,
            },
            "jump_driver": {
                "revision": "live-prototype-jump-policy-v1",
                "stepVelocityDeltaQ16": jump::JUMP_VELOCITY_DELTA_Q16,
                "status": {
                    "pending": evidence.jump_status.pending,
                    "grounded": evidence.jump_status.grounded,
                },
            },
            "object_observation_driver": {
                "revision": "live-prototype-object-target-v4",
                "maxDistanceQ9": observation::OBJECT_OBSERVATION_RADIUS_Q9,
                "status": {
                    "pending": evidence.observation_status.pending,
                    "target": object_target,
                },
                "completed": object_observation.is_some(),
                "observation": object_observation,
                "frameFeedback": {
                    "submitted": evidence.submitted_object_feedback,
                    "projected": evidence.rendered_object_feedback,
                    "submittedFrameCount": evidence.object_target_frame_count,
                    "copiedObjectState": false,
                },
            },
            "object_interaction_driver": {
                "revision": "live-prototype-object-action-v1",
                "input": "Enter",
                "maxDistanceQ9": interaction::OBJECT_ACTION_RADIUS_Q9,
                "acknowledgementFrameCount": interaction::ACKNOWLEDGEMENT_FRAME_COUNT,
                "attempt": object_action_attempt,
                "completion": evidence.interaction_completion.map(|completion| json!({
                    "applied": completion.applied,
                    "feedback": completion.feedback,
                })),
                "status": {
                    "pending": evidence.interaction_status.pending,
                    "acknowledgement": acknowledgement,
                    "committedCount": evidence.interaction_status.committed_count,
                    "ineligibleCount": evidence.interaction_status.ineligible_count,
                },
                "activatedFrameCount": evidence.object_action_frame_count,
                "copiedObjectState": false,
            },
        })
    );
    Ok(())
}

fn spawn_initial_actor(
    runtime: &mut Runtime,
    global_config: GlobalRegionConfig,
) -> Result<RuntimeActor> {
    let position = TerrainPosition::new(global_config.global_center, 0, 0)
        .context("prototype initial actor position failed")?;
    let terrain = runtime
        .query_terrain_height(position)
        .context("prototype initial terrain query failed")?;
    let motion = actor::initial_motion(position, terrain)?;
    runtime
        .spawn_actor(motion, presentation::initial())
        .context("prototype initial actor spawn failed")
}
