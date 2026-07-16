use anyhow::{Result, ensure};
use serde::Serialize;

use crate::runtime::{
    CANONICAL_OBJECTS_PER_REGION, ObjectTargetFeedback, ObjectTargetFeedbackKind,
};
use crate::streaming::address::GlobalRegionConfig;
use crate::streaming::async_resident::ObjectSourceNamespace;

use super::super::terrain::TerrainProjection;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectedObjectTarget {
    pub active_index: u32,
    pub semantic_region: u32,
    pub authored_local_id: u32,
    pub kind: ObjectTargetFeedbackKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectedObjectSuppression {
    pub active_index: u32,
    pub authored_local_id: u32,
}

pub(crate) fn validate(feedback: Option<ObjectTargetFeedback>) -> Result<()> {
    if let Some(feedback) = feedback {
        ensure!(
            feedback.identity.authored_local_id < CANONICAL_OBJECTS_PER_REGION,
            "object target authored local ID is outside the canonical region capacity"
        );
    }
    Ok(())
}

pub(crate) fn project(
    feedback: Option<ObjectTargetFeedback>,
    source_namespace: ObjectSourceNamespace,
    global_config: GlobalRegionConfig,
    projection: TerrainProjection,
) -> Result<Option<ProjectedObjectTarget>> {
    validate(feedback)?;
    let Some(feedback) = feedback else {
        return Ok(None);
    };
    let identity = feedback.identity;
    if identity.source_namespace != source_namespace {
        return Ok(None);
    }
    let Some(active_index) = global_config.active_index(identity.region) else {
        return Ok(None);
    };
    Ok(Some(ProjectedObjectTarget {
        active_index: active_index as u32,
        semantic_region: projection.region_id(active_index)?,
        authored_local_id: identity.authored_local_id,
        kind: feedback.kind,
    }))
}

pub(crate) fn project_suppression(
    identity: Option<crate::runtime::CanonicalObjectIdentity>,
    source_namespace: ObjectSourceNamespace,
    global_config: GlobalRegionConfig,
) -> Result<Option<ProjectedObjectSuppression>> {
    let Some(identity) = identity else {
        return Ok(None);
    };
    ensure!(
        identity.authored_local_id < CANONICAL_OBJECTS_PER_REGION,
        "object suppression authored local ID is outside the canonical region capacity"
    );
    if identity.source_namespace != source_namespace {
        return Ok(None);
    }
    let Some(active_index) = global_config.active_index(identity.region) else {
        return Ok(None);
    };
    Ok(Some(ProjectedObjectSuppression {
        active_index: active_index as u32,
        authored_local_id: identity.authored_local_id,
    }))
}

pub(crate) fn pack_object_suppression(
    suppression: Option<ProjectedObjectSuppression>,
) -> Result<u32> {
    let Some(suppression) = suppression else {
        return Ok(0);
    };
    ensure!(
        suppression.active_index < 1 << 5,
        "object suppression active index exceeds its packed range"
    );
    ensure!(
        suppression.authored_local_id < 1 << 10,
        "object suppression local ID exceeds its packed range"
    );
    Ok(1 << 31 | suppression.active_index << 10 | suppression.authored_local_id)
}
