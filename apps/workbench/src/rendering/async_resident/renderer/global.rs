use anyhow::Result;

use crate::address::GlobalRegionConfig;
use crate::async_resident::{AsyncReservationReport, ObjectSourceNamespace};

use super::AsyncResidentRenderer;

impl AsyncResidentRenderer {
    pub(in crate::rendering) fn reserve_canonical_global_composition(
        &mut self,
        config: GlobalRegionConfig,
        source_namespace: ObjectSourceNamespace,
        stable_seed_namespace: ObjectSourceNamespace,
    ) -> Result<AsyncReservationReport> {
        self.transfer.reserve_canonical_global_composition(
            config,
            source_namespace,
            stable_seed_namespace,
            &self.protected_slots(),
        )
    }

    pub(in crate::rendering) fn global_config(&self) -> Option<GlobalRegionConfig> {
        self.published
            .as_ref()
            .map(|snapshot| snapshot.global_config)
    }

    pub(in crate::rendering) fn object_source_namespace(&self) -> Option<ObjectSourceNamespace> {
        self.published
            .as_ref()
            .map(|snapshot| snapshot.object_source_namespace)
    }

    pub(in crate::rendering) fn object_stable_seed_namespace(
        &self,
    ) -> Option<ObjectSourceNamespace> {
        self.published
            .as_ref()
            .map(|snapshot| snapshot.object_stable_seed_namespace)
    }
}
