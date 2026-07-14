use windows::Win32::Graphics::Direct3D12::*;

use crate::rendering::meshlet_scene::skeletal::pipeline::SKELETAL_CONSTANT_COUNT;
use crate::rendering::meshlet_scene::skeletal::resources::{ExecutionResources, QUERY_COUNT};
use crate::rendering::meshlet_scene::skeletal::surface::occlusion::{
    FILTERED_VISIBLE_BYTES, OCCLUSION_COUNTER_BYTES, OCCLUSION_GROUPS, OCCLUSION_MASK_BYTES,
};
use crate::rendering::resident::{set_viewport, transition, uav_barrier};

use super::{SurfaceFrame, SurfaceRenderer};

impl SurfaceRenderer {
    pub unsafe fn record(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        execution: &ExecutionResources,
        skeletal_constants: [u32; SKELETAL_CONSTANT_COUNT as usize],
        frame: SurfaceFrame<'_>,
    ) {
        let history_queried = self.history_signature.as_ref() == Some(&skeletal_constants);
        self.last_history_queried = history_queried;
        self.last_bypass_reason = if history_queried {
            "none"
        } else if self.history_signature.is_none() {
            self.pending_invalidation_reason
        } else {
            "execution-signature-changed"
        };
        let surface_constants =
            self.constants(skeletal_constants, frame.background_color, history_queried);
        let gpu_start = unsafe { self.resources.gpu_start() };
        let visibility_target = unsafe { self.resources.visibility_handle() };
        let shadow_target = unsafe { self.resources.shadow_handle() };
        unsafe {
            transition(
                command_list,
                &execution.visible,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &execution.palette,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            command_list.SetDescriptorHeaps(&[Some(self.resources.heap.clone())]);
            command_list.OMSetRenderTargets(0, None, false, Some(&shadow_target));
            command_list.ClearDepthStencilView(shadow_target, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, None);
            set_viewport(
                command_list,
                super::super::shadow::MAP_SIDE,
                super::super::shadow::MAP_SIDE,
            );
            self.bind_graphics(command_list, surface_constants, gpu_start);
            command_list.SetPipelineState(&self.pipeline.shadow);
            command_list.ExecuteIndirect(
                &self.pipeline.mesh_signature,
                1,
                &execution.counters,
                0,
                None,
                0,
            );
            if frame.probe {
                command_list.EndQuery(&execution.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 4);
            }
            transition(
                command_list,
                &self.resources.shadow,
                D3D12_RESOURCE_STATE_DEPTH_WRITE,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &execution.counters,
                D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            self.bind_compute(command_list, surface_constants, gpu_start);
            let (counter_gpu, counter_cpu) = self.resources.occlusion_counter_uav_handles();
            command_list.ClearUnorderedAccessViewUint(
                counter_gpu,
                counter_cpu,
                &self.resources.occlusion.counters,
                &[0; 4],
                &[],
            );
            let (mask_gpu, mask_cpu) = self.resources.occlusion_mask_uav_handles();
            command_list.ClearUnorderedAccessViewUint(
                mask_gpu,
                mask_cpu,
                &self.resources.occlusion.candidate_mask,
                &[0; 4],
                &[],
            );
            uav_barrier(command_list, &self.resources.occlusion.counters);
            uav_barrier(command_list, &self.resources.occlusion.candidate_mask);
            uav_barrier(command_list, &self.resources.occlusion.group_offsets);
            transition(
                command_list,
                &self.resources.occlusion.hierarchy,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            command_list.SetPipelineState(&self.pipeline.occlusion_classify);
            command_list.Dispatch(OCCLUSION_GROUPS, 1, 1);
            for resource in [
                &self.resources.occlusion.counters,
                &self.resources.occlusion.candidate_mask,
                &self.resources.occlusion.group_offsets,
            ] {
                uav_barrier(command_list, resource);
            }
            command_list.SetPipelineState(&self.pipeline.occlusion_prefix);
            command_list.Dispatch(1, 1, 1);
            uav_barrier(command_list, &self.resources.occlusion.counters);
            uav_barrier(command_list, &self.resources.occlusion.group_offsets);
            command_list.SetPipelineState(&self.pipeline.occlusion_scatter);
            command_list.Dispatch(OCCLUSION_GROUPS, 1, 1);
            uav_barrier(command_list, &self.resources.occlusion.filtered_visible);
            uav_barrier(command_list, &self.resources.occlusion.counters);
            transition(
                command_list,
                &self.resources.occlusion.hierarchy,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            if frame.probe {
                command_list.EndQuery(&execution.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 5);
            }
            transition(
                command_list,
                &self.resources.occlusion.filtered_visible,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &self.resources.occlusion.counters,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            );
            let (winner_gpu, winner_cpu) = self.resources.winner_uav_handles();
            command_list.ClearUnorderedAccessViewUint(
                winner_gpu,
                winner_cpu,
                &self.resources.visibility_winner,
                &[0; 4],
                &[],
            );
            uav_barrier(command_list, &self.resources.visibility_winner);
            let render_targets = [visibility_target, frame.object_id_target];
            command_list.OMSetRenderTargets(
                2,
                Some(render_targets.as_ptr()),
                false,
                Some(&frame.depth_target),
            );
            command_list.ClearRenderTargetView(visibility_target, &[0.0; 4], None);
            set_viewport(command_list, self.width, self.height);
            self.bind_graphics(command_list, surface_constants, gpu_start);
            command_list.SetPipelineState(&self.pipeline.visibility);
            command_list.ExecuteIndirect(
                &self.pipeline.mesh_signature,
                1,
                &self.resources.occlusion.counters,
                0,
                None,
                0,
            );
            uav_barrier(command_list, &self.resources.candidate_to_visible);
            uav_barrier(command_list, &self.resources.visibility_winner);
            if frame.probe {
                command_list.EndQuery(&execution.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 6);
            }

            transition(
                command_list,
                &self.resources.visibility,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            transition(
                command_list,
                &self.resources.candidate_to_visible,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            );
            let (stats_gpu, stats_cpu) = self.resources.stats_uav_handles();
            command_list.ClearUnorderedAccessViewUint(
                stats_gpu,
                stats_cpu,
                &self.resources.stats,
                &[0; 4],
                &[],
            );
            self.preserve_composed_color(command_list, frame.back_buffer);
            self.bind_compute(command_list, surface_constants, gpu_start);
            command_list.SetPipelineState(&self.pipeline.shade);
            command_list.Dispatch(self.width.div_ceil(8), self.height.div_ceil(8), 1);
            for resource in [
                &self.resources.color,
                &self.resources.stats,
                &self.resources.samples,
            ] {
                uav_barrier(command_list, resource);
            }
            self.publish_resolved_color(command_list, frame.back_buffer);
            if frame.probe {
                command_list.EndQuery(&execution.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 7);
            }
            self.build_hierarchy(command_list, surface_constants, gpu_start);
            if frame.probe {
                command_list.EndQuery(&execution.query_heap, D3D12_QUERY_TYPE_TIMESTAMP, 8);
            }
            if frame.probe {
                self.record_probe_copies(command_list);
            } else {
                transition(
                    command_list,
                    &self.resources.visibility,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                );
                transition(
                    command_list,
                    &self.resources.shadow,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_DEPTH_WRITE,
                );
            }
            transition(
                command_list,
                &self.resources.candidate_to_visible,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            if frame.probe {
                transition(
                    command_list,
                    &self.resources.occlusion.filtered_visible,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.resources.occlusion.order_readback,
                    FILTERED_VISIBLE_BYTES,
                    &self.resources.occlusion.filtered_visible,
                    0,
                    FILTERED_VISIBLE_BYTES,
                );
                transition(
                    command_list,
                    &self.resources.occlusion.filtered_visible,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
                transition(
                    command_list,
                    &execution.visible,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.resources.occlusion.order_readback,
                    0,
                    &execution.visible,
                    0,
                    FILTERED_VISIBLE_BYTES,
                );
                transition(
                    command_list,
                    &execution.visible,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                );
                transition(
                    command_list,
                    &self.resources.occlusion.counters,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.resources.occlusion.counter_readback,
                    0,
                    &self.resources.occlusion.counters,
                    0,
                    OCCLUSION_COUNTER_BYTES,
                );
                transition(
                    command_list,
                    &self.resources.occlusion.counters,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
                transition(
                    command_list,
                    &self.resources.occlusion.candidate_mask,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                command_list.CopyBufferRegion(
                    &self.resources.occlusion.mask_readback,
                    0,
                    &self.resources.occlusion.candidate_mask,
                    0,
                    OCCLUSION_MASK_BYTES,
                );
                transition(
                    command_list,
                    &self.resources.occlusion.candidate_mask,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            } else {
                transition(
                    command_list,
                    &self.resources.occlusion.filtered_visible,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
                transition(
                    command_list,
                    &self.resources.occlusion.counters,
                    D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                );
            }
            transition(
                command_list,
                &execution.visible,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            transition(
                command_list,
                &execution.palette,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            );
            transition(
                command_list,
                &execution.counters,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            );
            if frame.probe {
                command_list.ResolveQueryData(
                    &execution.query_heap,
                    D3D12_QUERY_TYPE_TIMESTAMP,
                    0,
                    QUERY_COUNT,
                    &execution.timestamp_readback,
                    0,
                );
            }
        }
        self.history_signature = Some(skeletal_constants);
        self.pending_invalidation_reason = "none";
    }
}
