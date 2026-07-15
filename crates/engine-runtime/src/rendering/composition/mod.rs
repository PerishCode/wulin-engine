use std::time::Instant;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use windows::Win32::Graphics::Direct3D12::ID3D12GraphicsCommandList;

use crate::address::GlobalRegionConfig;
use crate::async_resident::ObjectSourceNamespace;
use crate::load::LoadConfig;
use crate::region::RegionCoord;
use crate::terrain::TerrainSourceNamespace;

use super::renderer::Renderer;
use super::terrain::control::TerrainPollOutcome;

mod authority;
mod contact;
mod global;
mod probe;
mod schedule;
mod state;
mod traversal;

use traversal::TraversalTarget;

pub use probe::CompositionProbe;

const COMPOSITION_REVISION: &str = "atomic-terrain-object-composition-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum HalfState {
    InFlight,
    Staged,
    Failed,
    Discarded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PairPurpose {
    Manual,
    Traversal,
    Prefetch,
}

impl PairPurpose {
    fn camera_driven(self) -> bool {
        self != Self::Manual
    }

    fn prefetch(self) -> bool {
        self == Self::Prefetch
    }
}

struct PendingPair {
    token: u64,
    config: LoadConfig,
    global_config: GlobalRegionConfig,
    terrain_source_namespace: TerrainSourceNamespace,
    object_source_namespace: ObjectSourceNamespace,
    object_stable_seed_namespace: ObjectSourceNamespace,
    terrain_transaction_id: u64,
    instance_transaction_id: u64,
    terrain: HalfState,
    instance: HalfState,
    failure: Option<String>,
    purpose: PairPurpose,
    started_at: Instant,
}

struct PendingPairInput {
    config: LoadConfig,
    global_config: GlobalRegionConfig,
    terrain_source_namespace: TerrainSourceNamespace,
    object_source_namespace: ObjectSourceNamespace,
    object_stable_seed_namespace: ObjectSourceNamespace,
    terrain_transaction_id: u64,
    instance_transaction_id: u64,
    purpose: PairPurpose,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishedPair {
    token: u64,
    config: LoadConfig,
    global_config: GlobalRegionConfig,
    terrain_source_namespace: TerrainSourceNamespace,
    object_source_namespace: ObjectSourceNamespace,
    object_stable_seed_namespace: ObjectSourceNamespace,
    terrain_transaction_id: u64,
    instance_transaction_id: u64,
    publication_count: u64,
    physical_slot_divergence_count: usize,
    logical_region_ids: Vec<u32>,
    instance_slots: Vec<u32>,
    terrain_slots: Vec<u32>,
    instance_mapping_sha256: String,
    terrain_mapping_sha256: String,
    global_regions: Vec<RegionCoord>,
    global_mapping_sha256: String,
    publication_ms: f64,
    camera_driven: bool,
    objects: crate::async_resident::AsyncTransactionReport,
    terrain: crate::terrain::TerrainTransactionReport,
}

pub(super) struct CompositionCoordinator {
    enabled: bool,
    next_token: u64,
    publication_count: u64,
    pending: Option<PendingPair>,
    published: Option<PublishedPair>,
    last_failure: Option<Value>,
    traversal: traversal::CameraTraversal,
}

impl Renderer {
    pub fn composition_status(&self) -> Value {
        self.composition.status_json()
    }

    pub fn composition_enabled(&self) -> bool {
        self.composition.enabled
    }

    pub(in crate::rendering) fn composition_grounding_mode(&self) -> u32 {
        authority::GROUNDING_MODE
    }

    pub fn enable_composition(&mut self) -> Result<()> {
        let published = self
            .composition
            .published
            .as_ref()
            .context("composition requires a published pair")?;
        ensure!(
            self.async_resident_renderer.config() == Some(published.config)
                && self.terrain_renderer.config() == Some(published.config)
                && self.async_resident_renderer.global_config() == Some(published.global_config)
                && self.terrain_renderer.global_config() == Some(published.global_config)
                && self.async_resident_renderer.object_source_namespace()
                    == Some(published.object_source_namespace)
                && self.async_resident_renderer.object_stable_seed_namespace()
                    == Some(published.object_stable_seed_namespace)
                && self.terrain_renderer.source_namespace()
                    == Some(published.terrain_source_namespace),
            "composition snapshots do not match the published pair"
        );
        self.terrain_renderer.enable()?;
        self.skeletal_scene_renderer.enable_canonical_surface();
        self.composition.enabled = true;
        Ok(())
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
            state::rollback_failed_pair(self);
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
        if pending.purpose.prefetch() {
            let instance_report = self
                .async_resident_renderer
                .discard_staged()
                .context("prefetch instance stage disappeared")?;
            let terrain_report = self
                .terrain_renderer
                .discard_staged()
                .context("prefetch terrain stage disappeared")?;
            ensure!(
                instance_report.transaction_id == pending.instance_transaction_id
                    && terrain_report.transaction_id == pending.terrain_transaction_id,
                "composition prepared the wrong transactions"
            );
            self.complete_cooked_object(&instance_report)?;
            self.terrain_streamer.mark_completed(&terrain_report)?;
            self.composition.traversal.mark_prefetch_completed(
                pending.token,
                TraversalTarget {
                    config: pending.config,
                    global_config: pending.global_config,
                },
                json!({
                    "terrain": terrain_report,
                    "objects": instance_report,
                    "preparationMs": pending.started_at.elapsed().as_secs_f64() * 1_000.0,
                }),
            );
            self.composition.last_failure = None;
            return Ok(());
        }
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
        let addressed = global::addressed(pending.global_config)?;
        let logical_region_ids = global::local_ids(&addressed);
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
        let (global_regions, global_mapping_sha256) = global::evidence(&addressed);
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
        self.complete_cooked_object(&instance_report)?;
        self.terrain_streamer.mark_completed(&terrain_report)?;
        self.composition.publication_count += 1;
        self.composition.published = Some(PublishedPair {
            token: pending.token,
            config: pending.config,
            global_config: pending.global_config,
            terrain_source_namespace: pending.terrain_source_namespace,
            object_source_namespace: pending.object_source_namespace,
            object_stable_seed_namespace: pending.object_stable_seed_namespace,
            terrain_transaction_id: pending.terrain_transaction_id,
            instance_transaction_id: pending.instance_transaction_id,
            publication_count: self.composition.publication_count,
            physical_slot_divergence_count: divergence,
            logical_region_ids,
            instance_slots,
            terrain_slots,
            instance_mapping_sha256,
            terrain_mapping_sha256,
            global_regions,
            global_mapping_sha256,
            publication_ms: pending.started_at.elapsed().as_secs_f64() * 1_000.0,
            camera_driven: pending.purpose.camera_driven(),
            objects: instance_report,
            terrain: terrain_report,
        });
        self.enable_composition()?;
        if pending.purpose == PairPurpose::Traversal {
            self.composition.traversal.mark_published(
                pending.token,
                pending.config,
                pending.global_config,
            )?;
        }
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
                    && report.config == pending.config
                    && report.global_config == pending.global_config
                    && report.object_source_namespace == pending.object_source_namespace
                    && report.object_stable_seed_namespace == pending.object_stable_seed_namespace,
                "staged instance half does not match the composition pair"
            );
        }
        if let Some(report) = self.terrain_renderer.staged_report() {
            ensure!(
                report.transaction_id == pending.terrain_transaction_id
                    && report.config == pending.config
                    && report.global_config == pending.global_config
                    && report.source_namespace == pending.terrain_source_namespace,
                "staged terrain half does not match the composition pair"
            );
        }
        if pending.terrain == HalfState::Staged && pending.instance == HalfState::Staged {
            let addressed = global::addressed(pending.global_config)?;
            let expected_regions = global::local_ids(&addressed);
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
            ensure!(
                terrain.iter().zip(addressed).all(|(assignment, region)| {
                    assignment.global_region == region.global_region
                        && assignment.region_id == region.local_region_id
                }),
                "terrain mapping does not match signed composition order"
            );
        }
        Ok(())
    }
}
