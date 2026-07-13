use std::time::Instant;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use windows::Win32::Graphics::Direct3D12::ID3D12GraphicsCommandList;

use crate::load::LoadConfig;
use crate::resident::active_region_ids;

use super::renderer::Renderer;
use super::terrain::control::TerrainPollOutcome;

mod contact;
mod fixture;
mod probe;

pub use fixture::CompositionFixture;
pub use probe::CompositionProbe;

const COMPOSITION_REVISION: &str = "atomic-terrain-object-composition-v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompositionOrder {
    #[default]
    TerrainFirst,
    ObjectFirst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum HalfState {
    InFlight,
    Staged,
    Failed,
    Discarded,
}

struct PendingPair {
    token: u64,
    config: LoadConfig,
    fixture: CompositionFixture,
    terrain_transaction_id: u64,
    instance_transaction_id: u64,
    terrain: HalfState,
    instance: HalfState,
    failure: Option<String>,
    started_at: Instant,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishedPair {
    token: u64,
    config: LoadConfig,
    fixture: CompositionFixture,
    terrain_transaction_id: u64,
    instance_transaction_id: u64,
    publication_count: u64,
    physical_slot_divergence_count: usize,
    logical_region_ids: Vec<u32>,
    instance_slots: Vec<u32>,
    terrain_slots: Vec<u32>,
    instance_mapping_sha256: String,
    terrain_mapping_sha256: String,
    publication_ms: f64,
}

pub(super) struct CompositionCoordinator {
    enabled: bool,
    order: CompositionOrder,
    fixture: CompositionFixture,
    next_token: u64,
    publication_count: u64,
    pending: Option<PendingPair>,
    published: Option<PublishedPair>,
    last_failure: Option<Value>,
}

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
        }
    }
}

impl CompositionCoordinator {
    pub(super) fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    fn begin(
        &mut self,
        config: LoadConfig,
        fixture: CompositionFixture,
        terrain_transaction_id: u64,
        instance_transaction_id: u64,
    ) -> u64 {
        let token = self.next_token;
        self.next_token += 1;
        self.pending = Some(PendingPair {
            token,
            config,
            fixture,
            terrain_transaction_id,
            instance_transaction_id,
            terrain: HalfState::InFlight,
            instance: HalfState::InFlight,
            failure: None,
            started_at: Instant::now(),
        });
        token
    }

    fn fail_half(&mut self, terrain: bool, transaction_id: u64, message: String) {
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

    fn status_json(&self) -> Value {
        let pending = self.pending.as_ref().map(|value| {
            json!({
                "token": value.token,
                "config": value.config,
                "fixture": value.fixture,
                "terrainTransactionId": value.terrain_transaction_id,
                "instanceTransactionId": value.instance_transaction_id,
                "terrainStage": value.terrain,
                "instanceStage": value.instance,
                "failure": value.failure,
                "pendingMs": value.started_at.elapsed().as_secs_f64() * 1_000.0,
            })
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
        })
    }
}

impl Renderer {
    pub unsafe fn schedule_composition(&mut self, config: LoadConfig) -> Result<Value> {
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        ensure!(
            !self.cooked_streamer.has_pending(),
            "cooked stream is active"
        );

        let terrain_reservation = self.terrain_renderer.reserve(config)?;
        let terrain_transaction_id = terrain_reservation.transaction_id;
        let instance_reservation = match self.async_resident_renderer.reserve_composition(config) {
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

        let fixture = self.composition.fixture;
        let token = self.composition.begin(
            config,
            fixture,
            terrain_transaction_id,
            instance_transaction_id,
        );
        if let Err(error) =
            unsafe { fixture::submit_generated_instances(self, instance_reservation, fixture) }
        {
            self.composition.fail_half(
                false,
                instance_transaction_id,
                format!("instance half failed to submit: {error:#}"),
            );
            return Err(error);
        }
        Ok(json!({
            "revision": COMPOSITION_REVISION,
            "token": token,
            "config": config,
            "fixture": fixture,
            "terrainTransactionId": terrain_transaction_id,
            "instanceTransactionId": instance_transaction_id,
        }))
    }

    pub fn composition_status(&self) -> Value {
        self.composition.status_json()
    }

    pub fn composition_enabled(&self) -> bool {
        self.composition.enabled
    }

    pub(in crate::rendering) fn composition_order(&self) -> CompositionOrder {
        self.composition.order
    }

    pub(in crate::rendering) fn composition_grounding_mode(&self) -> u32 {
        self.composition
            .published
            .as_ref()
            .map_or(CompositionFixture::CellCenter.grounding_mode(), |pair| {
                pair.fixture.grounding_mode()
            })
    }

    pub fn set_composition_order(&mut self, terrain_first: bool) {
        self.composition.order = if terrain_first {
            CompositionOrder::TerrainFirst
        } else {
            CompositionOrder::ObjectFirst
        };
    }

    pub fn set_composition_fixture(&mut self, fixture: CompositionFixture) -> Result<()> {
        ensure!(self.composition.pending.is_none(), "composition_pair_busy");
        ensure!(
            self.composition
                .published
                .as_ref()
                .is_none_or(|published| published.fixture == fixture),
            "composition fixture change requires restart"
        );
        self.composition.fixture = fixture;
        Ok(())
    }

    pub fn enable_composition(&mut self) -> Result<()> {
        let published = self
            .composition
            .published
            .as_ref()
            .context("composition requires a published pair")?;
        ensure!(
            self.async_resident_renderer.config() == Some(published.config)
                && self.terrain_renderer.config() == Some(published.config),
            "composition snapshots do not match the published pair"
        );
        self.meshlet_scene_renderer.disable();
        self.terrain_renderer.enable()?;
        self.skeletal_scene_renderer.enable();
        self.composition.enabled = true;
        Ok(())
    }

    pub fn disable_composition(&mut self) {
        if !self.composition.enabled {
            return;
        }
        self.composition.enabled = false;
        self.terrain_renderer.disable();
        self.skeletal_scene_renderer.disable();
    }

    pub(in crate::rendering) unsafe fn poll_composition_publication(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        terrain_outcome: Option<TerrainPollOutcome>,
    ) -> Result<()> {
        let Some(_) = self.composition.pending else {
            return Ok(());
        };
        if let Some(TerrainPollOutcome::Failed {
            transaction_id,
            message,
        }) = terrain_outcome
        {
            self.composition.fail_half(true, transaction_id, message);
        }

        let instance_staged = unsafe { self.async_resident_renderer.stage_frame(command_list) };
        let terrain_staged = unsafe { self.terrain_renderer.stage_frame(command_list) }?;
        if instance_staged {
            self.composition
                .pending
                .as_mut()
                .expect("composition pending disappeared")
                .instance = HalfState::Staged;
        }
        if terrain_staged {
            self.composition
                .pending
                .as_mut()
                .expect("composition pending disappeared")
                .terrain = HalfState::Staged;
        }

        self.validate_staged_pair()?;
        if self
            .composition
            .pending
            .as_ref()
            .is_some_and(|pending| pending.failure.is_some())
        {
            self.rollback_failed_pair();
            return Ok(());
        }

        let ready = self.composition.pending.as_ref().is_some_and(|pending| {
            pending.terrain == HalfState::Staged && pending.instance == HalfState::Staged
        });
        if !ready {
            return Ok(());
        }

        let pending = self
            .composition
            .pending
            .take()
            .expect("ready composition pair disappeared");
        let instance_slots = self
            .async_resident_renderer
            .staged_active_slots()
            .context("ready composition has no staged instance mapping")?
            .to_vec();
        let terrain_slots = self
            .terrain_renderer
            .staged_assignments()
            .context("ready composition has no staged terrain mapping")?
            .iter()
            .map(|value| value.slot)
            .collect::<Vec<_>>();
        let divergence = instance_slots
            .iter()
            .zip(&terrain_slots)
            .filter(|(instance, terrain)| instance != terrain)
            .count();
        let logical_region_ids = active_region_ids(pending.config)?;
        let mapping_hash = |slots: &[u32]| {
            let mut digest = Sha256::new();
            for (region_id, slot) in logical_region_ids.iter().zip(slots) {
                digest.update(region_id.to_le_bytes());
                digest.update(slot.to_le_bytes());
            }
            format!("{:x}", digest.finalize())
        };
        let instance_mapping_sha256 = mapping_hash(&instance_slots);
        let terrain_mapping_sha256 = mapping_hash(&terrain_slots);
        let instance_report = self
            .async_resident_renderer
            .commit_staged()
            .context("instance staged publication disappeared")?;
        let terrain_report = self
            .terrain_renderer
            .commit_staged()
            .context("terrain staged publication disappeared")?;
        ensure!(
            instance_report.transaction_id == pending.instance_transaction_id
                && terrain_report.transaction_id == pending.terrain_transaction_id,
            "composition committed the wrong transactions"
        );
        self.terrain_streamer.mark_published(&terrain_report)?;
        self.composition.publication_count += 1;
        self.composition.published = Some(PublishedPair {
            token: pending.token,
            config: pending.config,
            fixture: pending.fixture,
            terrain_transaction_id: pending.terrain_transaction_id,
            instance_transaction_id: pending.instance_transaction_id,
            publication_count: self.composition.publication_count,
            physical_slot_divergence_count: divergence,
            logical_region_ids,
            instance_slots,
            terrain_slots,
            instance_mapping_sha256,
            terrain_mapping_sha256,
            publication_ms: pending.started_at.elapsed().as_secs_f64() * 1_000.0,
        });
        self.composition.last_failure = None;
        Ok(())
    }

    fn validate_staged_pair(&mut self) -> Result<()> {
        let Some(pending) = self.composition.pending.as_mut() else {
            return Ok(());
        };
        if let Some(report) = self.async_resident_renderer.staged_report() {
            ensure!(
                report.transaction_id == pending.instance_transaction_id
                    && report.config == pending.config,
                "staged instance half does not match the composition pair"
            );
        }
        if let Some(report) = self.terrain_renderer.staged_report() {
            ensure!(
                report.transaction_id == pending.terrain_transaction_id
                    && report.config == pending.config,
                "staged terrain half does not match the composition pair"
            );
        }
        if pending.terrain == HalfState::Staged && pending.instance == HalfState::Staged {
            let expected_regions = active_region_ids(pending.config)?;
            let terrain = self
                .terrain_renderer
                .staged_assignments()
                .context("staged terrain mapping is absent")?;
            let instances = self
                .async_resident_renderer
                .staged_active_slots()
                .context("staged instance mapping is absent")?;
            ensure!(
                terrain.len() == expected_regions.len()
                    && instances.len() == expected_regions.len(),
                "composition mapping lengths differ"
            );
            ensure!(
                terrain
                    .iter()
                    .zip(expected_regions)
                    .all(|(assignment, region_id)| assignment.region_id == region_id),
                "terrain mapping is not in canonical logical order"
            );
        }
        Ok(())
    }

    fn rollback_failed_pair(&mut self) {
        let Some(pending) = self.composition.pending.as_mut() else {
            return;
        };
        if pending.instance == HalfState::Staged {
            let _ = self.async_resident_renderer.discard_staged();
            pending.instance = HalfState::Discarded;
        }
        if pending.terrain == HalfState::Staged {
            let _ = self.terrain_renderer.discard_staged();
            pending.terrain = HalfState::Discarded;
        }
        if pending.instance == HalfState::InFlight || pending.terrain == HalfState::InFlight {
            return;
        }
        let pending = self
            .composition
            .pending
            .take()
            .expect("failed composition pair disappeared");
        self.composition.last_failure = Some(json!({
            "token": pending.token,
            "config": pending.config,
            "fixture": pending.fixture,
            "terrainTransactionId": pending.terrain_transaction_id,
            "instanceTransactionId": pending.instance_transaction_id,
            "terrainStage": pending.terrain,
            "instanceStage": pending.instance,
            "message": pending.failure,
            "rollbackMs": pending.started_at.elapsed().as_secs_f64() * 1_000.0,
        }));
    }
}
