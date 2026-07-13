use std::time::Instant;

use anyhow::{Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const REGION_SIDE_METERS: f32 = 16.0;
pub const REGION_HALF_SIDE_METERS: f32 = REGION_SIDE_METERS * 0.5;
pub const MAX_RENDER_REGION_DELTA: u64 = 8;
const PROBE_GRID_SIDE: i32 = 32;
const PROBE_REGION_RADIUS: i64 = 2;
const PROBE_Q9_DENOMINATOR: i32 = 512;
const PROBE_STEP_Q9: i32 = 256;
const PROBE_MIN_LOCAL_Q9: i32 = -8 * PROBE_Q9_DENOMINATOR;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionCoord {
    pub x: i64,
    pub z: i64,
}

impl RegionCoord {
    pub const ZERO: Self = Self { x: 0, z: 0 };

    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }

    pub fn checked_offset(self, x: i64, z: i64) -> Result<Self> {
        Ok(Self {
            x: self
                .x
                .checked_add(x)
                .ok_or_else(|| anyhow::anyhow!("global region X overflows signed 64-bit range"))?,
            z: self
                .z
                .checked_add(z)
                .ok_or_else(|| anyhow::anyhow!("global region Z overflows signed 64-bit range"))?,
        })
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitPosition {
    pub region: RegionCoord,
    pub local_meters: [f32; 3],
}

impl SplitPosition {
    pub fn from_scene_local(anchor: RegionCoord, local: [f32; 3]) -> Result<Self> {
        ensure!(
            local.iter().all(|value| value.is_finite()),
            "scene-local position must be finite"
        );
        let (region_x, local_x) = normalize_axis(anchor.x, local[0])?;
        let (region_z, local_z) = normalize_axis(anchor.z, local[2])?;
        Ok(Self {
            region: RegionCoord::new(region_x, region_z),
            local_meters: [local_x, local[1], local_z],
        })
    }

    pub fn render_relative(self, origin: RegionCoord) -> Result<[f32; 3]> {
        let delta_x = bounded_delta(self.region.x, origin.x, "X")?;
        let delta_z = bounded_delta(self.region.z, origin.z, "Z")?;
        Ok([
            delta_x as f32 * REGION_SIDE_METERS + self.local_meters[0],
            self.local_meters[1],
            delta_z as f32 * REGION_SIDE_METERS + self.local_meters[2],
        ])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorldSpace {
    scene_anchor: RegionCoord,
    render_origin: RegionCoord,
    relocation_count: u64,
    rebase_count: u64,
    reset_count: u64,
}

impl Default for WorldSpace {
    fn default() -> Self {
        Self {
            scene_anchor: RegionCoord::ZERO,
            render_origin: RegionCoord::ZERO,
            relocation_count: 0,
            rebase_count: 0,
            reset_count: 0,
        }
    }
}

impl WorldSpace {
    pub fn split_position(self, scene_local: [f32; 3]) -> Result<SplitPosition> {
        SplitPosition::from_scene_local(self.scene_anchor, scene_local)
    }

    pub fn render_position(self, scene_local: [f32; 3]) -> Result<[f32; 3]> {
        self.split_position(scene_local)?
            .render_relative(self.render_origin)
    }

    pub fn relocate(
        &mut self,
        anchor: RegionCoord,
        scene_local_positions: &[[f32; 3]],
    ) -> Result<()> {
        validate_scene(anchor, anchor, scene_local_positions)?;
        self.scene_anchor = anchor;
        self.render_origin = anchor;
        self.relocation_count += 1;
        Ok(())
    }

    pub fn rebase(
        &mut self,
        origin: RegionCoord,
        scene_local_positions: &[[f32; 3]],
    ) -> Result<()> {
        validate_scene(self.scene_anchor, origin, scene_local_positions)?;
        self.render_origin = origin;
        self.rebase_count += 1;
        Ok(())
    }

    pub fn reset(&mut self, scene_local_positions: &[[f32; 3]]) -> Result<()> {
        validate_scene(RegionCoord::ZERO, RegionCoord::ZERO, scene_local_positions)?;
        self.scene_anchor = RegionCoord::ZERO;
        self.render_origin = RegionCoord::ZERO;
        self.reset_count += 1;
        Ok(())
    }

    pub fn status_json(self) -> Value {
        json!({
            "revision": "camera-relative-global-space-v1",
            "sceneAnchor": self.scene_anchor,
            "renderOrigin": self.render_origin,
            "regionSideMeters": REGION_SIDE_METERS,
            "localIntervalMeters": [-REGION_HALF_SIDE_METERS, REGION_HALF_SIDE_METERS],
            "localIntervalUpperExclusive": true,
            "signedCoordinateBits": 64,
            "maximumRenderRegionDelta": MAX_RENDER_REGION_DELTA,
            "relocationCount": self.relocation_count,
            "rebaseCount": self.rebase_count,
            "resetCount": self.reset_count,
        })
    }

    pub fn probe(self) -> Result<Value> {
        let started = Instant::now();
        let mut global_hash = Sha256::new();
        let mut render_hash = Sha256::new();
        let mut sample_count = 0_u32;
        let mut normalization_mismatch_count = 0_u32;
        let mut reconstruction_mismatch_count = 0_u32;
        let mut non_finite_count = 0_u32;
        let mut maximum_rendered_region_delta = 0_u64;

        for region_z in -PROBE_REGION_RADIUS..=PROBE_REGION_RADIUS {
            for region_x in -PROBE_REGION_RADIUS..=PROBE_REGION_RADIUS {
                for local_z_index in 0..PROBE_GRID_SIDE {
                    for local_x_index in 0..PROBE_GRID_SIDE {
                        let local_x_q9 = PROBE_MIN_LOCAL_Q9 + local_x_index * PROBE_STEP_Q9;
                        let local_z_q9 = PROBE_MIN_LOCAL_Q9 + local_z_index * PROBE_STEP_Q9;
                        let scene_local = [
                            region_x as f32 * REGION_SIDE_METERS
                                + local_x_q9 as f32 / PROBE_Q9_DENOMINATOR as f32,
                            0.0,
                            region_z as f32 * REGION_SIDE_METERS
                                + local_z_q9 as f32 / PROBE_Q9_DENOMINATOR as f32,
                        ];
                        let split = self.split_position(scene_local)?;
                        let expected_region =
                            self.scene_anchor.checked_offset(region_x, region_z)?;
                        let expected_local = [
                            local_x_q9 as f32 / PROBE_Q9_DENOMINATOR as f32,
                            0.0,
                            local_z_q9 as f32 / PROBE_Q9_DENOMINATOR as f32,
                        ];
                        if split.region != expected_region
                            || split.local_meters.map(f32::to_bits)
                                != expected_local.map(f32::to_bits)
                        {
                            normalization_mismatch_count += 1;
                        }

                        let reconstructed = [
                            (split.region.x - self.scene_anchor.x) as f32 * REGION_SIDE_METERS
                                + split.local_meters[0],
                            split.local_meters[1],
                            (split.region.z - self.scene_anchor.z) as f32 * REGION_SIDE_METERS
                                + split.local_meters[2],
                        ];
                        if reconstructed.map(f32::to_bits) != scene_local.map(f32::to_bits) {
                            reconstruction_mismatch_count += 1;
                        }

                        let render = split.render_relative(self.render_origin)?;
                        if render.iter().any(|value| !value.is_finite()) {
                            non_finite_count += 1;
                        }
                        let delta_x = split
                            .region
                            .x
                            .checked_sub(self.render_origin.x)
                            .expect("validated probe X delta overflowed");
                        let delta_z = split
                            .region
                            .z
                            .checked_sub(self.render_origin.z)
                            .expect("validated probe Z delta overflowed");
                        maximum_rendered_region_delta = maximum_rendered_region_delta
                            .max(delta_x.unsigned_abs())
                            .max(delta_z.unsigned_abs());

                        hash_split_position(&mut global_hash, split);
                        hash_f32_array(&mut render_hash, render);
                        sample_count += 1;
                    }
                }
            }
        }

        let boundary_mismatch_count = boundary_mismatch_count(self.scene_anchor)?;
        let conversion_elapsed_ns = started.elapsed().as_nanos();
        Ok(json!({
            "revision": "camera-relative-global-space-probe-v1",
            "sceneAnchor": self.scene_anchor,
            "renderOrigin": self.render_origin,
            "sampleCount": sample_count,
            "regionGridSide": PROBE_REGION_RADIUS * 2 + 1,
            "localGridSide": PROBE_GRID_SIDE,
            "q9Denominator": PROBE_Q9_DENOMINATOR,
            "normalizationMismatchCount": normalization_mismatch_count,
            "reconstructionMismatchCount": reconstruction_mismatch_count,
            "boundarySampleCount": 9,
            "boundaryMismatchCount": boundary_mismatch_count,
            "nonFiniteCount": non_finite_count,
            "maximumRenderedRegionDelta": maximum_rendered_region_delta,
            "globalPositionHash": digest_hex(global_hash),
            "renderPositionHash": digest_hex(render_hash),
            "conversionElapsedNs": conversion_elapsed_ns,
            "perSampleAllocationBytes": 0,
        }))
    }
}

fn validate_scene(
    anchor: RegionCoord,
    origin: RegionCoord,
    scene_local_positions: &[[f32; 3]],
) -> Result<()> {
    for &position in scene_local_positions {
        SplitPosition::from_scene_local(anchor, position)?.render_relative(origin)?;
    }
    Ok(())
}

fn normalize_axis(anchor: i64, scene_local: f32) -> Result<(i64, f32)> {
    let shift = ((f64::from(scene_local) + f64::from(REGION_HALF_SIDE_METERS))
        / f64::from(REGION_SIDE_METERS))
    .floor();
    ensure!(
        shift >= i64::MIN as f64 && shift <= i64::MAX as f64,
        "scene-local position exceeds signed 64-bit region range"
    );
    let shift = shift as i64;
    let region = anchor
        .checked_add(shift)
        .ok_or_else(|| anyhow::anyhow!("global region overflows signed 64-bit range"))?;
    let local = (f64::from(scene_local) - shift as f64 * f64::from(REGION_SIDE_METERS)) as f32;
    ensure!(
        (-REGION_HALF_SIDE_METERS..REGION_HALF_SIDE_METERS).contains(&local),
        "normalized local coordinate is outside the half-open region interval"
    );
    Ok((region, local))
}

fn bounded_delta(position: i64, origin: i64, axis: &str) -> Result<i64> {
    let delta = position
        .checked_sub(origin)
        .ok_or_else(|| anyhow::anyhow!("render-relative {axis} region delta overflowed"))?;
    ensure!(
        delta.unsigned_abs() <= MAX_RENDER_REGION_DELTA,
        "render-relative {axis} region delta {} exceeds bound {}",
        delta,
        MAX_RENDER_REGION_DELTA
    );
    Ok(delta)
}

fn boundary_mismatch_count(anchor: RegionCoord) -> Result<u32> {
    let cases = [
        (-24.0, -1_i64, -8.0),
        (-16.0, -1, 0.0),
        (-8.0, 0, -8.0),
        (-0.5, 0, -0.5),
        (0.0, 0, 0.0),
        (7.5, 0, 7.5),
        (8.0, 1, -8.0),
        (16.0, 1, 0.0),
        (24.0, 2, -8.0),
    ];
    let mut mismatches = 0;
    for (value, region_offset, expected_local) in cases {
        let split = SplitPosition::from_scene_local(anchor, [value, 0.0, 0.0])?;
        if split.region.x != anchor.x + region_offset
            || split.local_meters[0].to_bits() != f32::to_bits(expected_local)
        {
            mismatches += 1;
        }
    }
    Ok(mismatches)
}

fn hash_split_position(hasher: &mut Sha256, position: SplitPosition) {
    hasher.update(position.region.x.to_le_bytes());
    hasher.update(position.region.z.to_le_bytes());
    hash_f32_array(hasher, position.local_meters);
}

pub fn hash_f32_array<const N: usize>(hasher: &mut Sha256, values: [f32; N]) {
    for value in values {
        hasher.update(value.to_bits().to_le_bytes());
    }
}

pub fn digest_hex(hasher: Sha256) -> String {
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_open_boundary() {
        let anchor = RegionCoord::new(7, -9);
        let negative = SplitPosition::from_scene_local(anchor, [-8.0, 0.0, -8.0]).unwrap();
        let positive = SplitPosition::from_scene_local(anchor, [8.0, 0.0, 8.0]).unwrap();
        assert_eq!(negative.region, anchor);
        assert_eq!(negative.local_meters, [-8.0, 0.0, -8.0]);
        assert_eq!(positive.region, RegionCoord::new(8, -8));
        assert_eq!(positive.local_meters, [-8.0, 0.0, -8.0]);
    }

    #[test]
    fn far_anchor_is_exact() {
        let anchor = RegionCoord::new(1_i64 << 40, -(1_i64 << 40));
        let split = SplitPosition::from_scene_local(anchor, [9.0, 6.0, 12.0]).unwrap();
        assert_eq!(split.region, RegionCoord::new(anchor.x + 1, anchor.z + 1));
        assert_eq!(split.render_relative(anchor).unwrap(), [9.0, 6.0, 12.0]);
    }

    #[test]
    fn rejected_rebase_is_transactional() {
        let positions = [[0.0, 0.0, 0.0], [9.0, 6.0, 12.0]];
        let mut world = WorldSpace::default();
        let rejected = RegionCoord::new(MAX_RENDER_REGION_DELTA as i64 + 2, 0);
        assert!(world.rebase(rejected, &positions).is_err());
        assert_eq!(world.render_origin, RegionCoord::ZERO);
        assert_eq!(world.status_json()["rebaseCount"], 0);
    }

    #[test]
    fn probe_covers_exact_fixture() {
        let probe = WorldSpace::default().probe().unwrap();
        assert_eq!(probe["sampleCount"], 25_600);
        assert_eq!(probe["normalizationMismatchCount"], 0);
        assert_eq!(probe["reconstructionMismatchCount"], 0);
        assert_eq!(probe["boundaryMismatchCount"], 0);
        assert_eq!(probe["nonFiniteCount"], 0);
    }
}
