mod resources;
mod skeletal;

pub(in crate::rendering) use resources::CatalogBuffers;
pub(in crate::rendering) use skeletal::GROUND_BYTES;
pub(in crate::rendering) use skeletal::{
    CompositionProbeInput, CompositionSurfaceInput, SurfaceProbe,
};
pub use skeletal::{SkeletalFrame, SkeletalSceneRenderer};
