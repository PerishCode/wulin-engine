use super::*;

impl Default for CompositionCoordinator {
    fn default() -> Self {
        Self {
            enabled: false,
            order: CompositionOrder::default(),
            fixture: CompositionFixture::default(),
            next_token: 1,
            publication_count: 0,
            pending: None,
            published: None,
            last_failure: None,
            traversal: traversal::CameraTraversal::default(),
        }
    }
}

impl CompositionCoordinator {
    pub(in crate::rendering) fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn begin(
        &mut self,
        config: LoadConfig,
        global_config: Option<GlobalRegionConfig>,
        fixture: CompositionFixture,
        terrain_transaction_id: u64,
        instance_transaction_id: u64,
        camera_driven: bool,
    ) -> u64 {
        let token = self.next_token;
        self.next_token += 1;
        self.pending = Some(PendingPair {
            token,
            config,
            global_config,
            fixture,
            terrain_transaction_id,
            instance_transaction_id,
            terrain: HalfState::InFlight,
            instance: HalfState::InFlight,
            failure: None,
            camera_driven,
            started_at: Instant::now(),
        });
        token
    }

    pub(super) fn fail_half(&mut self, terrain: bool, transaction_id: u64, message: String) {
        let Some(pending) = self.pending.as_mut() else {
            return;
        };
        let expected = if terrain {
            pending.terrain_transaction_id
        } else {
            pending.instance_transaction_id
        };
        if expected != transaction_id {
            return;
        }
        if terrain {
            pending.terrain = HalfState::Failed;
        } else {
            pending.instance = HalfState::Failed;
        }
        pending.failure = Some(message);
    }

    pub(super) fn status_json(&self) -> Value {
        let pending = self.pending.as_ref().map(|value| {
            let mut pending = json!({
                "token": value.token,
                "config": value.config,
                "fixture": value.fixture,
                "terrainTransactionId": value.terrain_transaction_id,
                "instanceTransactionId": value.instance_transaction_id,
                "terrainStage": value.terrain,
                "instanceStage": value.instance,
                "failure": value.failure,
                "cameraDriven": value.camera_driven,
                "pendingMs": value.started_at.elapsed().as_secs_f64() * 1_000.0,
            });
            if let Some(global) = value.global_config {
                pending["globalConfig"] = json!(global);
            }
            pending
        });
        json!({
            "revision": COMPOSITION_REVISION,
            "enabled": self.enabled,
            "order": self.order,
            "fixture": self.fixture,
            "nextToken": self.next_token,
            "pending": pending,
            "published": self.published,
            "lastFailure": self.last_failure,
            "traversal": self.traversal.status_json(),
        })
    }
}
