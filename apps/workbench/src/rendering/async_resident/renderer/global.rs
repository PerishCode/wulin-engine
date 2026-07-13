use anyhow::Result;

use crate::address::GlobalRegionConfig;
use crate::async_resident::AsyncReservationReport;

use super::AsyncResidentRenderer;

impl AsyncResidentRenderer {
    pub(in crate::rendering) fn reserve_global_composition(
        &mut self,
        config: GlobalRegionConfig,
    ) -> Result<AsyncReservationReport> {
        self.transfer
            .reserve_global_composition(config, &self.protected_slots())
    }

    pub(in crate::rendering) fn global_config(&self) -> Option<GlobalRegionConfig> {
        self.published
            .as_ref()
            .and_then(|snapshot| snapshot.global_config)
    }
}
