mod load;
mod region;
mod rendering;
mod resident;
mod runtime;
mod scene;
mod streaming;
mod terrain_query;
mod timeline;

pub use load::{SemanticObject, semantic_object};
pub use region::RegionCoord;
pub use rendering::gpu_capture::CapturedPixels;
pub use rendering::{CapturedFrame, RenderOutcome};
pub use runtime::{
    FrameRequest, RetainedTerrainBody, RetainedTerrainBodyAdvance, RetainedTerrainBodyBatch,
    Runtime, TerrainBodyHandle,
};
pub use streaming::address::GlobalRegionConfig;
pub(crate) use streaming::{address, async_resident, objects, terrain};
pub use terrain_query::{
    TERRAIN_BODY_HEIGHT_DENOMINATOR, TERRAIN_POSITION_DENOMINATOR,
    TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE, TERRAIN_POSITION_LOCAL_MIN_Q9,
    TERRAIN_POSITION_REGION_SIDE_Q9, TERRAIN_QUERY_HEIGHT_DENOMINATOR, TerrainBody,
    TerrainBodyAdvance, TerrainBodyContact, TerrainBodyMotion, TerrainBodyStep,
    TerrainBodyTranslation, TerrainContactClassification, TerrainHeight, TerrainPosition,
    TerrainTriangle,
};
pub use timeline::{
    SIMULATION_MAX_ELAPSED_NANOSECONDS, SIMULATION_MAX_STEPS_PER_ADVANCE,
    SIMULATION_STEPS_PER_SECOND, SIMULATION_TIME_DENOMINATOR, SimulationAdvance,
};
