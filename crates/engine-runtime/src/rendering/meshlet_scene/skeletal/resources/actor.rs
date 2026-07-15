use std::mem::size_of;
use std::ptr;

use anyhow::{Context, Result, ensure};
use windows::Win32::Graphics::Direct3D12::{
    D3D12_HEAP_TYPE_UPLOAD, D3D12_RANGE, D3D12_RESOURCE_FLAG_NONE,
    D3D12_RESOURCE_STATE_GENERIC_READ, ID3D12Device, ID3D12Resource,
};

use crate::rendering::ActorRenderProjection;
use crate::rendering::resident::create_buffer;
use crate::terrain_query::{TERRAIN_BODY_HEIGHT_DENOMINATOR, TERRAIN_POSITION_DENOMINATOR};

use super::ACTOR_CANDIDATE_INDEX;

pub const ACTOR_VISIBLE_RECORD_BYTES: u32 = size_of::<ActorVisibleCandidate>() as u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorVisibleCandidate {
    pub position: [f32; 3],
    pub height: f32,
    pub semantic_region: u32,
    pub archetype: u32,
    pub lod: u32,
    pub stable_identity_low: u32,
    pub stable_identity_high: u32,
    pub pose_slot: u32,
    pub candidate_index: u32,
    pub material: u32,
    pub yaw_q16: u32,
    pub animation: u32,
}

impl ActorVisibleCandidate {
    pub const EMPTY: Self = Self {
        position: [0.0; 3],
        height: 0.0,
        semantic_region: 0,
        archetype: 0,
        lod: 0,
        stable_identity_low: 0,
        stable_identity_high: 0,
        pose_slot: u32::MAX,
        candidate_index: u32::MAX,
        material: 0,
        yaw_q16: 0,
        animation: u32::MAX,
    };

    pub fn from_projection(projection: ActorRenderProjection) -> Result<Self> {
        ensure!(
            projection.position_denominator == TERRAIN_POSITION_DENOMINATOR
                && projection.height_denominator == TERRAIN_BODY_HEIGHT_DENOMINATOR,
            "actor GPU candidate projection denominators diverged"
        );
        let generation = projection.actor.handle.generation();
        ensure!(
            generation != 0,
            "actor GPU candidate generation must be nonzero"
        );
        let presentation = projection.actor.presentation;
        let position_scale = projection.position_denominator as f32;
        let height_scale = projection.height_denominator as f32;
        Ok(Self {
            position: [
                projection.window_position_q9[0] as f32 / position_scale,
                (projection.center_height_q16 as f32 - projection.half_height_q16 as f32)
                    / height_scale,
                projection.window_position_q9[1] as f32 / position_scale,
            ],
            height: projection.half_height_q16 as f32 * 2.0 / height_scale,
            semantic_region: projection.semantic_region,
            archetype: presentation.archetype,
            lod: 0,
            stable_identity_low: generation as u32,
            stable_identity_high: (generation >> 32) as u32,
            pose_slot: u32::MAX,
            candidate_index: ACTOR_CANDIDATE_INDEX,
            material: presentation.material,
            yaw_q16: presentation.yaw_q16,
            animation: presentation.animation,
        })
    }

    pub fn words(self) -> [u32; 14] {
        [
            self.position[0].to_bits(),
            self.position[1].to_bits(),
            self.position[2].to_bits(),
            self.height.to_bits(),
            self.semantic_region,
            self.archetype,
            self.lod,
            self.stable_identity_low,
            self.stable_identity_high,
            self.pose_slot,
            self.candidate_index,
            self.material,
            self.yaw_q16,
            self.animation,
        ]
    }
}

pub struct ActorFrameUpload {
    pub resource: ID3D12Resource,
    frame_slots: u32,
    write_count: u64,
}

impl ActorFrameUpload {
    pub unsafe fn new(device: &ID3D12Device, frame_slots: u32) -> Result<Self> {
        ensure!(frame_slots != 0, "actor GPU upload requires a frame slot");
        let resource = unsafe {
            create_buffer(
                device,
                u64::from(frame_slots) * u64::from(ACTOR_VISIBLE_RECORD_BYTES),
                D3D12_HEAP_TYPE_UPLOAD,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        Ok(Self {
            resource,
            frame_slots,
            write_count: 0,
        })
    }

    pub unsafe fn write(
        &mut self,
        frame_slot: u32,
        projection: Option<ActorRenderProjection>,
    ) -> Result<(u64, ActorVisibleCandidate)> {
        ensure!(
            frame_slot < self.frame_slots,
            "actor GPU upload frame slot is outside the allocation"
        );
        let next_write_count = self
            .write_count
            .checked_add(1)
            .context("actor GPU upload write count exhausted")?;
        let candidate = projection
            .map(ActorVisibleCandidate::from_projection)
            .transpose()?
            .unwrap_or(ActorVisibleCandidate::EMPTY);
        let offset = usize::try_from(frame_slot)
            .context("actor GPU upload frame slot exceeds process size")?
            * size_of::<ActorVisibleCandidate>();
        let mut mapped = ptr::null_mut();
        unsafe {
            self.resource.Map(
                0,
                Some(&D3D12_RANGE { Begin: 0, End: 0 }),
                Some(&mut mapped),
            )
        }
        .context("actor GPU upload map failed")?;
        unsafe {
            ptr::copy_nonoverlapping(
                std::ptr::from_ref(&candidate).cast::<u8>(),
                mapped.cast::<u8>().add(offset),
                size_of::<ActorVisibleCandidate>(),
            );
            self.resource.Unmap(
                0,
                Some(&D3D12_RANGE {
                    Begin: offset,
                    End: offset + size_of::<ActorVisibleCandidate>(),
                }),
            );
        }
        self.write_count = next_write_count;
        Ok((
            unsafe { self.resource.GetGPUVirtualAddress() } + offset as u64,
            candidate,
        ))
    }

    pub const fn frame_slots(&self) -> u32 {
        self.frame_slots
    }

    pub const fn allocation_bytes(&self) -> u64 {
        self.frame_slots as u64 * ACTOR_VISIBLE_RECORD_BYTES as u64
    }

    pub const fn write_count(&self) -> u64 {
        self.write_count
    }
}

const _: () = assert!(ACTOR_VISIBLE_RECORD_BYTES == 56);

#[cfg(test)]
#[path = "../../../../../tests/private/actor_gpu.rs"]
mod tests;
