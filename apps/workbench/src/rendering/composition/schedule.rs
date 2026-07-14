use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};

use crate::address::GlobalRegionConfig;
use crate::load::LoadConfig;

use super::{COMPOSITION_REVISION, PairPurpose, PendingPairInput, Renderer};

impl Renderer {
    pub(super) unsafe fn schedule_composition_pair(
        &mut self,
        config: LoadConfig,
        global_config: GlobalRegionConfig,
        purpose: PairPurpose,
    ) -> Result<Value> {
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        let terrain_source_namespace = self.terrain_streamer.source_namespace()?;
        let object_source = self
            .cooked_object_streamer
            .source()
            .context("canonical schedule requires a schema-3 object source")?;
        ensure!(
            object_source.stable_seed_namespace == super::authority::object_source_namespace(),
            "object stable-seed namespace does not match canonical runtime"
        );
        let object_source_namespace = object_source.source_namespace;
        let object_stable_seed_namespace = object_source.stable_seed_namespace;

        let terrain_reservation = self
            .terrain_renderer
            .reserve_canonical_global(global_config, terrain_source_namespace)?;
        let terrain_transaction_id = terrain_reservation.transaction_id;
        let instance_reservation = self
            .async_resident_renderer
            .reserve_canonical_global_composition(
                global_config,
                object_source_namespace,
                object_stable_seed_namespace,
            );
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
            terrain_transaction_id,
            instance_transaction_id,
            purpose,
        });
        let object_submission = self
            .cooked_object_streamer
            .schedule(instance_reservation)
            .map(|_| ());
        let object_failure = object_submission.err().map(|error| format!("{error:#}"));
        if let Some(message) = &object_failure {
            let _ = self
                .async_resident_renderer
                .cancel_reservation(instance_transaction_id);
            self.cooked_object_streamer.mark_failed(
                instance_transaction_id,
                message.clone(),
                Default::default(),
            );
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
            "terrainTransactionId": terrain_transaction_id,
            "instanceTransactionId": instance_transaction_id,
            "cameraDriven": purpose.camera_driven(),
        });
        if purpose.prefetch() {
            response["prefetch"] = json!(true);
        }
        if let Some(message) = object_failure {
            response["objectScheduleFailure"] = json!(message);
        }
        response["globalConfig"] = json!(global_config);
        response["terrainSourceNamespace"] = json!(terrain_source_namespace);
        response["objectSourceNamespace"] = json!(object_source_namespace);
        Ok(response)
    }
}
