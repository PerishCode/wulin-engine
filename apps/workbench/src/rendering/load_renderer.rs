use std::mem::{ManuallyDrop, size_of};
use std::ptr;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};

use crate::load::{LoadConfig, MAX_VISIBLE_INSTANCES};
use crate::scene::SceneState;

use super::load_pipeline::{LOAD_CONSTANT_COUNT, LoadPipeline};

const QUERY_COUNT: u32 = 4;
pub const PROBE_ITERATIONS: u32 = 64;

pub struct LoadRenderer {
    pipeline: LoadPipeline,
    visible_instances: ID3D12Resource,
    draw_arguments: ID3D12Resource,
    query_heap: ID3D12QueryHeap,
    timestamp_readback: ID3D12Resource,
    argument_readback: ID3D12Resource,
    timestamp_frequency: u64,
    width: u32,
    height: u32,
    config: Option<LoadConfig>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadProbe {
    pub config: LoadConfig,
    pub logical_instance_count: u64,
    pub active_region_count: u32,
    pub candidate_instance_count: u32,
    pub dispatch: [u32; 3],
    pub indirect_draw_count: u32,
    pub probe_iterations: u32,
    pub visible_instance_count: u32,
    pub gpu_compaction_ms: f64,
    pub gpu_draw_ms: f64,
    pub gpu_total_ms: f64,
}

impl LoadRenderer {
    pub unsafe fn new(
        device: &ID3D12Device,
        timestamp_frequency: u64,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let pipeline = unsafe { LoadPipeline::new(device) }?;
        let visible_instances = unsafe {
            create_buffer(
                device,
                u64::from(MAX_VISIBLE_INSTANCES) * size_of::<u32>() as u64,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let draw_arguments = unsafe {
            create_buffer(
                device,
                size_of::<D3D12_DRAW_ARGUMENTS>() as u64,
                D3D12_HEAP_TYPE_DEFAULT,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            )
        }?;
        let query_heap = unsafe { create_query_heap(device) }?;
        let timestamp_readback = unsafe {
            create_buffer(
                device,
                u64::from(QUERY_COUNT) * size_of::<u64>() as u64,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        let argument_readback = unsafe {
            create_buffer(
                device,
                size_of::<D3D12_DRAW_ARGUMENTS>() as u64,
                D3D12_HEAP_TYPE_READBACK,
                D3D12_RESOURCE_STATE_COPY_DEST,
                D3D12_RESOURCE_FLAG_NONE,
            )
        }?;
        Ok(Self {
            pipeline,
            visible_instances,
            draw_arguments,
            query_heap,
            timestamp_readback,
            argument_readback,
            timestamp_frequency,
            width,
            height,
            config: None,
        })
    }

    pub fn configure(&mut self, config: LoadConfig) {
        self.config = Some(config);
    }

    pub fn disable(&mut self) {
        self.config = None;
    }

    pub fn config(&self) -> Option<LoadConfig> {
        self.config
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        scene: &SceneState,
        render_targets: [D3D12_CPU_DESCRIPTOR_HANDLE; 2],
        depth_target: D3D12_CPU_DESCRIPTOR_HANDLE,
        probe: bool,
    ) -> Result<()> {
        let config = self.config.context("load renderer is not configured")?;
        let constants = load_constants(scene, config, self.width, self.height);
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 0) };
        }
        unsafe {
            command_list.SetComputeRootSignature(&self.pipeline.compute_root);
            command_list.SetComputeRoot32BitConstants(
                0,
                LOAD_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetComputeRootUnorderedAccessView(
                1,
                self.visible_instances.GetGPUVirtualAddress(),
            );
            command_list
                .SetComputeRootUnorderedAccessView(2, self.draw_arguments.GetGPUVirtualAddress());
            let [groups_x, groups_y, groups_z] = config.dispatch();
            let iterations = if probe { PROBE_ITERATIONS } else { 1 };
            for _ in 0..iterations {
                command_list.SetPipelineState(&self.pipeline.reset);
                command_list.Dispatch(1, 1, 1);
                uav_barrier(command_list, &self.draw_arguments);
                command_list.SetPipelineState(&self.pipeline.cull);
                command_list.Dispatch(groups_x, groups_y, groups_z);
                uav_barrier(command_list, &self.visible_instances);
                uav_barrier(command_list, &self.draw_arguments);
            }
        }
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 1) };
        }

        unsafe {
            transition(
                command_list,
                &self.visible_instances,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &self.draw_arguments,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            );
            command_list.OMSetRenderTargets(
                2,
                Some(render_targets.as_ptr()),
                false,
                Some(&depth_target),
            );
            command_list.ClearRenderTargetView(render_targets[1], &[0.0; 4], None);
            command_list.ClearDepthStencilView(depth_target, D3D12_CLEAR_FLAG_DEPTH, 0.0, 0, None);
            set_viewport(command_list, self.width, self.height);
            command_list.SetGraphicsRootSignature(&self.pipeline.graphics_root);
            command_list.SetPipelineState(&self.pipeline.graphics);
            command_list.SetGraphicsRoot32BitConstants(
                0,
                LOAD_CONSTANT_COUNT,
                constants.as_ptr().cast(),
                0,
            );
            command_list.SetGraphicsRootShaderResourceView(
                1,
                self.visible_instances.GetGPUVirtualAddress(),
            );
            command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        }
        if probe {
            unsafe { command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 2) };
        }
        let iterations = if probe { PROBE_ITERATIONS } else { 1 };
        unsafe {
            for _ in 0..iterations {
                command_list.ClearDepthStencilView(
                    depth_target,
                    D3D12_CLEAR_FLAG_DEPTH,
                    0.0,
                    0,
                    None,
                );
                command_list.ExecuteIndirect(
                    &self.pipeline.command_signature,
                    1,
                    &self.draw_arguments,
                    0,
                    None,
                    0,
                );
            }
        }
        if probe {
            unsafe {
                command_list.EndQuery(&self.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 3);
                command_list.ResolveQueryData(
                    &self.query_heap,
                    D3D12_QUERY_TYPE_TIMESTAMP,
                    0,
                    QUERY_COUNT,
                    &self.timestamp_readback,
                    0,
                );
                transition(
                    command_list,
                    &self.draw_arguments,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.argument_readback,
                    0,
                    &self.draw_arguments,
                    0,
                    size_of::<D3D12_DRAW_ARGUMENTS>() as u64,
                );
                transition(
                    command_list,
                    &self.draw_arguments,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
        } else {
            unsafe {
                transition(
                    command_list,
                    &self.draw_arguments,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
        }
        unsafe {
            transition(
                command_list,
                &self.visible_instances,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
        }
        Ok(())
    }

    pub unsafe fn read_probe(&self) -> Result<LoadProbe> {
        let config = self.config.context("load renderer is not configured")?;
        let timestamps =
            unsafe { read_values::<u64>(&self.timestamp_readback, QUERY_COUNT as usize) }?;
        let arguments = unsafe { read_values::<u32>(&self.argument_readback, 4) }?;
        if arguments[0] != 6 || arguments[2] != 0 || arguments[3] != 0 {
            bail!("indirect draw arguments are invalid: {arguments:?}");
        }
        if arguments[1] > config.candidate_instance_count() {
            bail!("visible instance count exceeds active candidates");
        }
        let milliseconds = |start: usize, end: usize| {
            timestamps[end].saturating_sub(timestamps[start]) as f64 * 1_000.0
                / self.timestamp_frequency as f64
                / PROBE_ITERATIONS as f64
        };
        Ok(LoadProbe {
            config,
            logical_instance_count: config.logical_instance_count(),
            active_region_count: config.active_region_count(),
            candidate_instance_count: config.candidate_instance_count(),
            dispatch: config.dispatch(),
            indirect_draw_count: 1,
            probe_iterations: PROBE_ITERATIONS,
            visible_instance_count: arguments[1],
            gpu_compaction_ms: milliseconds(0, 1),
            gpu_draw_ms: milliseconds(2, 3),
            gpu_total_ms: milliseconds(0, 3),
        })
    }
}

fn load_constants(
    scene: &SceneState,
    config: LoadConfig,
    width: u32,
    height: u32,
) -> [u32; LOAD_CONSTANT_COUNT as usize] {
    let mut constants = [0u32; LOAD_CONSTANT_COUNT as usize];
    for (destination, value) in constants[..16].iter_mut().zip(
        scene
            .view_projection(width as f32 / height as f32)
            .to_cols_array(),
    ) {
        *destination = value.to_bits();
    }
    constants[16] = config.world_region_side;
    constants[17] = config.active_center_x;
    constants[18] = config.active_center_z;
    constants[19] = config.active_radius;
    constants[20] = MAX_VISIBLE_INSTANCES;
    constants
}

unsafe fn create_query_heap(device: &ID3D12Device) -> Result<ID3D12QueryHeap> {
    let desc = D3D12_QUERY_HEAP_DESC {
        Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
        Count: QUERY_COUNT,
        NodeMask: 0,
    };
    let mut heap = None;
    unsafe { device.CreateQueryHeap(&desc, &mut heap) }
        .context("load query heap creation failed")?;
    heap.context("load query heap creation returned no heap")
}

unsafe fn create_buffer(
    device: &ID3D12Device,
    size: u64,
    heap_type: D3D12_HEAP_TYPE,
    initial_state: D3D12_RESOURCE_STATES,
    flags: D3D12_RESOURCE_FLAGS,
) -> Result<ID3D12Resource> {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: heap_type,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Alignment: 0,
        Width: size,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_UNKNOWN,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        Flags: flags,
    };
    let mut resource = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            initial_state,
            None,
            &mut resource,
        )
    }
    .context("load buffer allocation failed")?;
    resource.context("load buffer allocation returned no resource")
}

unsafe fn read_values<T: Copy>(resource: &ID3D12Resource, count: usize) -> Result<Vec<T>> {
    let byte_count = count * size_of::<T>();
    let mut mapped = ptr::null_mut();
    let range = D3D12_RANGE {
        Begin: 0,
        End: byte_count,
    };
    unsafe { resource.Map(0, Some(&range), Some(&mut mapped)) }
        .context("load readback map failed")?;
    let mut values = Vec::<T>::with_capacity(count);
    unsafe {
        ptr::copy_nonoverlapping(mapped.cast::<T>(), values.as_mut_ptr(), count);
        values.set_len(count);
    }
    unsafe { resource.Unmap(0, Some(&D3D12_RANGE { Begin: 0, End: 0 })) };
    Ok(values)
}

unsafe fn transition(
    command_list: &ID3D12GraphicsCommandList,
    resource: &ID3D12Resource,
    before: D3D12_RESOURCE_STATES,
    after: D3D12_RESOURCE_STATES,
) {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource: ManuallyDrop::new(Some(resource.clone())),
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    };
    unsafe { command_list.ResourceBarrier(std::slice::from_ref(&barrier)) };
    unsafe {
        let transition = &mut *barrier.Anonymous.Transition;
        ManuallyDrop::drop(&mut transition.pResource);
    }
}

unsafe fn uav_barrier(command_list: &ID3D12GraphicsCommandList, resource: &ID3D12Resource) {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            UAV: ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                pResource: ManuallyDrop::new(Some(resource.clone())),
            }),
        },
    };
    unsafe { command_list.ResourceBarrier(std::slice::from_ref(&barrier)) };
    unsafe {
        let uav = &mut *barrier.Anonymous.UAV;
        ManuallyDrop::drop(&mut uav.pResource);
    }
}

unsafe fn set_viewport(command_list: &ID3D12GraphicsCommandList, width: u32, height: u32) {
    unsafe {
        command_list.RSSetViewports(&[D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width as f32,
            Height: height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        }]);
        command_list.RSSetScissorRects(&[RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        }]);
    }
}
