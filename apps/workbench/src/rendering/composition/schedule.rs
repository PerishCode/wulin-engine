use anyhow::{Result, ensure};
use serde_json::{Value, json};

use crate::address::GlobalRegionConfig;
use crate::load::LoadConfig;

use super::{COMPOSITION_REVISION, PairPurpose, PendingPairInput, Renderer, fixture};

impl Renderer {
    pub unsafe fn schedule_composition(&mut self, config: LoadConfig) -> Result<Value> {
        ensure!(
            !self.composition.traversal.is_enabled(),
            "camera traversal owns composition scheduling"
        );
        unsafe { self.schedule_composition_pair(config, None, PairPurpose::Manual) }
    }

    pub(super) unsafe fn schedule_composition_pair(
        &mut self,
        config: LoadConfig,
        global_config: Option<GlobalRegionConfig>,
        purpose: PairPurpose,
    ) -> Result<Value> {
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        ensure!(
            !self.cooked_streamer.has_pending(),
            "cooked stream is active"
        );
        let fixture = self.composition.fixture;
        let terrain_source_namespace = self.terrain_streamer.source_namespace()?;
        ensure!(
            terrain_source_namespace.is_none() || global_config.is_some(),
            "signed terrain composition requires a global schedule"
        );
        let cooked_object_source = self.cooked_object_streamer.source();
        ensure!(
            cooked_object_source.is_none()
                || (terrain_source_namespace.is_some() && global_config.is_some()),
            "cooked object source requires signed canonical terrain composition"
        );
        if let Some(source) = cooked_object_source {
            ensure!(
                source.stable_seed_namespace == fixture.object_source_namespace(),
                "cooked object stable-seed namespace does not match the active fixture"
            );
        }
        let object_source_namespace = terrain_source_namespace.map(|_| {
            cooked_object_source.map_or_else(
                || fixture.object_source_namespace(),
                |source| source.source_namespace,
            )
        });
        let object_stable_seed_namespace = terrain_source_namespace.map(|_| {
            cooked_object_source.map_or_else(
                || fixture.object_source_namespace(),
                |source| source.stable_seed_namespace,
            )
        });

        let terrain_reservation = match (global_config, terrain_source_namespace) {
            (Some(global), Some(source)) => self
                .terrain_renderer
                .reserve_canonical_global(global, source)?,
            (Some(global), None) => self.terrain_renderer.reserve_global(global)?,
            (None, None) => self.terrain_renderer.reserve(config)?,
            (None, Some(_)) => unreachable!("signed composition source has no global config"),
        };
        let terrain_transaction_id = terrain_reservation.transaction_id;
        let instance_reservation = match (global_config, object_source_namespace) {
            (Some(global), Some(source)) => self
                .async_resident_renderer
                .reserve_canonical_global_composition(
                    global,
                    source,
                    object_stable_seed_namespace.expect("canonical seed namespace is absent"),
                ),
            (Some(global), None) => self
                .async_resident_renderer
                .reserve_global_composition(global),
            (None, None) => self.async_resident_renderer.reserve_composition(config),
            (None, Some(_)) => unreachable!("canonical object source has no global config"),
        };
        let instance_reservation = match instance_reservation {
            Ok(report) => report,
            Err(error) => {
                let _ = self.terrain_renderer.cancel(terrain_transaction_id);
                return Err(error);
            }
        };
        let instance_transaction_id = instance_reservation.transaction_id;
        if let Err(error) = self.terrain_streamer.schedule(terrain_reservation) {
            let _ = self.terrain_renderer.cancel(terrain_transaction_id);
            let _ = self
                .async_resident_renderer
                .cancel_reservation(instance_transaction_id);
            return Err(error);
        }

        let token = self.composition.begin(PendingPairInput {
            config,
            global_config,
            terrain_source_namespace,
            object_source_namespace,
            object_stable_seed_namespace,
            fixture,
            terrain_transaction_id,
            instance_transaction_id,
            purpose,
        });
        let object_submission = if cooked_object_source.is_some() {
            self.cooked_object_streamer
                .schedule(instance_reservation)
                .map(|_| ())
        } else {
            unsafe { fixture::submit_generated_instances(self, instance_reservation, fixture) }
        };
        let object_failure = object_submission.err().map(|error| format!("{error:#}"));
        if let Some(message) = &object_failure {
            let _ = self
                .async_resident_renderer
                .cancel_reservation(instance_transaction_id);
            if cooked_object_source.is_some() {
                self.cooked_object_streamer.mark_failed(
                    instance_transaction_id,
                    message.clone(),
                    Default::default(),
                );
            }
            self.composition.fail_half(
                false,
                instance_transaction_id,
                format!("instance half failed to submit: {message}"),
            );
        }
        let mut response = json!({
            "revision": COMPOSITION_REVISION,
            "token": token,
            "config": config,
            "fixture": fixture,
            "terrainTransactionId": terrain_transaction_id,
            "instanceTransactionId": instance_transaction_id,
            "cameraDriven": purpose.camera_driven(),
        });
        if purpose.prefetch() {
            response["prefetch"] = json!(true);
        }
        if cooked_object_source.is_some() {
            response["objectPayloadSource"] = json!("cooked-pack");
        }
        if let Some(message) = object_failure {
            response["objectScheduleFailure"] = json!(message);
        }
        if let Some(global) = global_config {
            response["globalConfig"] = json!(global);
        }
        if let Some(source) = terrain_source_namespace {
            response["terrainSourceNamespace"] = json!(source);
        }
        if let Some(source) = object_source_namespace {
            response["objectSourceNamespace"] = json!(source);
        }
        if let Some(namespace) = object_stable_seed_namespace
            .filter(|namespace| Some(*namespace) != object_source_namespace)
        {
            response["objectStableSeedNamespace"] = json!(namespace);
        }
        Ok(response)
    }
}
