use crate::load::LoadConfig;
use crate::region::RegionCoord;
use crate::rendering::renderer::object_target::{ProjectedObjectTarget, project, validate};
use crate::rendering::terrain::TerrainProjection;
use crate::runtime::{
    CANONICAL_OBJECTS_PER_REGION, CanonicalObjectIdentity, ObjectTargetFeedback,
    ObjectTargetFeedbackKind,
};
use crate::streaming::address::GlobalRegionConfig;
use crate::streaming::async_resident::ObjectSourceNamespace;

fn source() -> ObjectSourceNamespace {
    ObjectSourceNamespace::from_bytes([7; 32])
}

fn config(center: RegionCoord) -> GlobalRegionConfig {
    GlobalRegionConfig::new(center.x, center.z, center.x, center.z, 2).unwrap()
}

fn projection() -> TerrainProjection {
    TerrainProjection::for_objects(LoadConfig::new(128, 64, 64, 2).unwrap()).unwrap()
}

fn identity(
    source_namespace: ObjectSourceNamespace,
    region: RegionCoord,
    id: u32,
) -> CanonicalObjectIdentity {
    CanonicalObjectIdentity {
        source_namespace,
        region,
        authored_local_id: id,
    }
}

fn feedback(identity: CanonicalObjectIdentity) -> ObjectTargetFeedback {
    ObjectTargetFeedback {
        identity,
        kind: ObjectTargetFeedbackKind::Selected,
    }
}

#[test]
fn projects_current_source_window() {
    let center = RegionCoord::new(i64::MAX - 20, i64::MIN + 20);
    let exact = identity(source(), center.checked_offset(1, -1).unwrap(), 511);
    assert_eq!(
        project(
            Some(feedback(exact)),
            source(),
            config(center),
            projection()
        )
        .unwrap(),
        Some(ProjectedObjectTarget {
            active_index: 8,
            semantic_region: 63 * 128 + 65,
            authored_local_id: 511,
            kind: ObjectTargetFeedbackKind::Selected,
        })
    );
    assert_eq!(
        project(
            Some(feedback(identity(
                ObjectSourceNamespace::from_bytes([8; 32]),
                exact.region,
                511,
            ))),
            source(),
            config(center),
            projection(),
        )
        .unwrap(),
        None
    );
    assert_eq!(
        project(
            Some(feedback(identity(
                source(),
                center.checked_offset(3, 0).unwrap(),
                511,
            ))),
            source(),
            config(center),
            projection(),
        )
        .unwrap(),
        None
    );
}

#[test]
fn remaps_after_traversal() {
    let first_center = RegionCoord::new(900, -400);
    let exact = identity(source(), first_center.checked_offset(1, 0).unwrap(), 29);
    let first = project(
        Some(feedback(exact)),
        source(),
        config(first_center),
        projection(),
    )
    .unwrap()
    .unwrap();
    let second = project(
        Some(ObjectTargetFeedback {
            identity: exact,
            kind: ObjectTargetFeedbackKind::Activated,
        }),
        source(),
        config(first_center.checked_offset(1, 0).unwrap()),
        projection(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(first.active_index, 13);
    assert_eq!(first.semantic_region, 64 * 128 + 65);
    assert_eq!(second.active_index, 12);
    assert_eq!(second.semantic_region, 64 * 128 + 64);
    assert_eq!(first.authored_local_id, second.authored_local_id);
    assert_eq!(second.kind, ObjectTargetFeedbackKind::Activated);
}

#[test]
fn rejects_invalid_id() {
    let error = validate(Some(feedback(identity(
        source(),
        RegionCoord::ZERO,
        CANONICAL_OBJECTS_PER_REGION,
    ))))
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("outside the canonical region capacity")
    );
}
