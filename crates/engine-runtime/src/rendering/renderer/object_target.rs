use anyhow::{Result, ensure};
use serde::Serialize;

use crate::runtime::{CANONICAL_OBJECTS_PER_REGION, CanonicalObjectIdentity};
use crate::streaming::address::GlobalRegionConfig;
use crate::streaming::async_resident::ObjectSourceNamespace;

use super::super::terrain::TerrainProjection;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectTargetFeedback {
    pub active_index: u32,
    pub semantic_region: u32,
    pub authored_local_id: u32,
}

pub(crate) fn validate(identity: Option<CanonicalObjectIdentity>) -> Result<()> {
    if let Some(identity) = identity {
        ensure!(
            identity.authored_local_id < CANONICAL_OBJECTS_PER_REGION,
            "object target authored local ID is outside the canonical region capacity"
        );
    }
    Ok(())
}

pub(crate) fn project(
    identity: Option<CanonicalObjectIdentity>,
    source_namespace: ObjectSourceNamespace,
    global_config: GlobalRegionConfig,
    projection: TerrainProjection,
) -> Result<Option<ObjectTargetFeedback>> {
    validate(identity)?;
    let Some(identity) = identity else {
        return Ok(None);
    };
    if identity.source_namespace != source_namespace {
        return Ok(None);
    }
    let Some(active_index) = global_config.active_index(identity.region) else {
        return Ok(None);
    };
    Ok(Some(ObjectTargetFeedback {
        active_index: active_index as u32,
        semantic_region: projection.region_id(active_index)?,
        authored_local_id: identity.authored_local_id,
    }))
}
