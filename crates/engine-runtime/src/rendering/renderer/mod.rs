use anyhow::{Context, Result, bail};
use serde_json::Value;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_12_1;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_UNSPECIFIED, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::objects::CookedObjectStreamer;
use crate::terrain::TerrainStreamer;
use crate::terrain_query::{TerrainHeight, TerrainQueryPosition};

use super::async_resident::AsyncResidentRenderer;
use super::composition::{CompositionCoordinator, CompositionProbe};
use super::device::{
    DeviceCapabilities, enable_debug_layer, query_required_capabilities, select_reference_adapter,
};
use super::frame_targets::FrameTargets;
use super::gpu_capture::{CapturedPixels, Readback};
use super::meshlet_scene::SkeletalSceneRenderer;
use super::terrain::TerrainRenderer;

mod frame;

const BUFFER_COUNT: usize = 2;

unsafe extern "C" {
    fn runtime_link_agility_exports();
}

pub struct Renderer {
    device: ID3D12Device,
    pub(super) queue: ID3D12CommandQueue,
    swap_chain: IDXGISwapChain3,
    rtv_heap: ID3D12DescriptorHeap,
    rtv_increment: usize,
    back_buffers: Vec<ID3D12Resource>,
    allocators: Vec<ID3D12CommandAllocator>,
    command_list: ID3D12GraphicsCommandList,
    pub(super) fence: ID3D12Fence,
    fence_event: HANDLE,
    fence_values: [u64; BUFFER_COUNT],
    pub(super) next_fence_value: u64,
    capture: Readback,
    object_id_capture: Readback,
    frame_targets: FrameTargets,
    pub(super) cooked_object_streamer: CookedObjectStreamer,
    pub(super) async_resident_renderer: AsyncResidentRenderer,
    pub(super) skeletal_scene_renderer: SkeletalSceneRenderer,
    pub(super) terrain_streamer: TerrainStreamer,
    pub(super) terrain_renderer: TerrainRenderer,
    pub(super) composition: CompositionCoordinator,
    adapter_name: String,
    debug_layer: bool,
    capabilities: DeviceCapabilities,
}

pub struct CapturedFrame {
    pub color: CapturedPixels,
    pub object_ids: Option<CapturedPixels>,
}

pub struct RenderOutcome {
    pub capture: Option<CapturedFrame>,
    pub composition_probe: Option<CompositionProbe>,
}

pub(crate) struct RenderFrame<'a> {
    pub color: [f32; 4],
    pub capture: bool,
    pub capture_object_ids: bool,
    pub probe: bool,
    pub presentation_tick: u32,
    pub presentation_status: Option<&'a Value>,
    pub scene: &'a mut crate::scene::SceneState,
}

impl Renderer {
    pub unsafe fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        unsafe { runtime_link_agility_exports() };
        let debug_layer = cfg!(debug_assertions);
        if debug_layer {
            unsafe { enable_debug_layer()? };
        }

        let factory_flags = if debug_layer {
            DXGI_CREATE_FACTORY_DEBUG
        } else {
            DXGI_CREATE_FACTORY_FLAGS(0)
        };
        let factory: IDXGIFactory6 =
            unsafe { CreateDXGIFactory2(factory_flags) }.context("CreateDXGIFactory2 failed")?;
        let (adapter, adapter_name) = unsafe { select_reference_adapter(&factory) }?;

        let mut device = None;
        unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_1, &mut device) }
            .context("D3D12CreateDevice failed")?;
        let device: ID3D12Device = device.context("D3D12CreateDevice returned no device")?;
        let capabilities = unsafe { query_required_capabilities(&device) }?;

        let queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        };
        let queue: ID3D12CommandQueue = unsafe { device.CreateCommandQueue(&queue_desc) }
            .context("CreateCommandQueue failed")?;
        let timestamp_frequency =
            unsafe { queue.GetTimestampFrequency() }.context("GetTimestampFrequency failed")?;

        let swap_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: BUFFER_COUNT as u32,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: 0,
        };
        let swap_chain: IDXGISwapChain3 =
            unsafe { factory.CreateSwapChainForHwnd(&queue, hwnd, &swap_desc, None, None) }
                .context("CreateSwapChainForHwnd failed")?
                .cast()
                .context("IDXGISwapChain3 is unavailable")?;
        unsafe { factory.MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER) }
            .context("MakeWindowAssociation failed")?;

        let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: BUFFER_COUNT as u32,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        let rtv_heap: ID3D12DescriptorHeap = unsafe { device.CreateDescriptorHeap(&heap_desc) }
            .context("CreateDescriptorHeap failed")?;
        let rtv_increment =
            unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) }
                as usize;
        let heap_start = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };
        let mut back_buffers = Vec::with_capacity(BUFFER_COUNT);
        for index in 0..BUFFER_COUNT {
            let buffer: ID3D12Resource = unsafe { swap_chain.GetBuffer(index as u32) }
                .with_context(|| format!("GetBuffer({index}) failed"))?;
            let handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: heap_start.ptr + index * rtv_increment,
            };
            unsafe { device.CreateRenderTargetView(&buffer, None, handle) };
            back_buffers.push(buffer);
        }
        let capture = unsafe { Readback::new(&device, &back_buffers[0]) }?;
        let frame_targets = unsafe { FrameTargets::new(&device, width, height) }?;
        let object_id_capture =
            unsafe { Readback::new(&device, frame_targets.semantic_resource()) }?;
        let async_resident_renderer = unsafe { AsyncResidentRenderer::new(&device) }?;
        let terrain_renderer =
            unsafe { TerrainRenderer::new(&device, timestamp_frequency, width, height) }?;
        let skeletal_scene_renderer = unsafe {
            SkeletalSceneRenderer::new(
                &device,
                &queue,
                async_resident_renderer.descriptor_heap(),
                terrain_renderer.descriptor_heap(),
                timestamp_frequency,
                width,
                height,
            )
        }?;

        let mut allocators = Vec::with_capacity(BUFFER_COUNT);
        for _ in 0..BUFFER_COUNT {
            allocators.push(
                unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
                    .context("CreateCommandAllocator failed")?,
            );
        }
        let command_list: ID3D12GraphicsCommandList = unsafe {
            device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocators[0], None)
        }
        .context("CreateCommandList failed")?;
        unsafe { command_list.Close() }.context("initial command list close failed")?;

        let fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
            .context("CreateFence failed")?;
        let fence_event =
            unsafe { CreateEventW(None, false, false, None) }.context("CreateEventW failed")?;

        Ok(Self {
            device,
            queue,
            swap_chain,
            rtv_heap,
            rtv_increment,
            back_buffers,
            allocators,
            command_list,
            fence,
            fence_event,
            fence_values: [0; BUFFER_COUNT],
            next_fence_value: 1,
            capture,
            object_id_capture,
            frame_targets,
            cooked_object_streamer: CookedObjectStreamer::default(),
            async_resident_renderer,
            skeletal_scene_renderer,
            terrain_streamer: TerrainStreamer::default(),
            terrain_renderer,
            composition: CompositionCoordinator::default(),
            adapter_name,
            debug_layer,
            capabilities,
        })
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn debug_layer(&self) -> bool {
        self.debug_layer
    }

    pub fn mesh_shader_tier(&self) -> u32 {
        self.capabilities.mesh_shader_tier
    }

    pub fn shader_model(&self) -> &str {
        self.capabilities.shader_model
    }

    pub fn barycentrics_supported(&self) -> bool {
        self.capabilities.barycentrics
    }

    pub fn rasterizer_ordered_views_supported(&self) -> bool {
        self.capabilities.rasterizer_ordered_views
    }

    pub fn visibility_format_supported(&self) -> bool {
        self.capabilities.visibility_format
    }

    pub fn color_uav_format_supported(&self) -> bool {
        self.capabilities.color_uav_format
    }

    pub fn query_terrain_height(&self, position: TerrainQueryPosition) -> Result<TerrainHeight> {
        self.terrain_renderer.query_height(position)
    }

    pub fn arm_async_copy_gate(&mut self) -> Result<u64> {
        self.async_resident_renderer.arm_gate()
    }

    pub unsafe fn release_async_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.async_resident_renderer.release_gate() }
    }

    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        unsafe { self.terrain_renderer.wait_idle() }?;
        unsafe { self.async_resident_renderer.wait_idle() }?;
        let signal = self.next_fence_value;
        self.next_fence_value += 1;
        unsafe { self.queue.Signal(&self.fence, signal) }.context("queue signal failed")?;
        unsafe { self.wait_for_value(signal) }
    }

    pub unsafe fn device_removed_reason(&self) -> Option<String> {
        unsafe { self.device.GetDeviceRemovedReason() }
            .err()
            .map(|error| error.to_string())
    }

    fn rtv_handle(&self, index: usize) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        let start = unsafe { self.rtv_heap.GetCPUDescriptorHandleForHeapStart() };
        D3D12_CPU_DESCRIPTOR_HANDLE {
            ptr: start.ptr + index * self.rtv_increment,
        }
    }

    unsafe fn wait_for_buffer(&self, index: usize) -> Result<()> {
        let value = self.fence_values[index];
        if value == 0 || unsafe { self.fence.GetCompletedValue() } >= value {
            return Ok(());
        }
        unsafe { self.wait_for_value(value) }
    }

    unsafe fn wait_for_value(&self, value: u64) -> Result<()> {
        unsafe { self.fence.SetEventOnCompletion(value, self.fence_event) }
            .context("SetEventOnCompletion failed")?;
        let result = unsafe { WaitForSingleObject(self.fence_event, INFINITE) };
        if result != WAIT_OBJECT_0 {
            bail!("WaitForSingleObject returned {result:?}");
        }
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.wait_idle();
            let _ = CloseHandle(self.fence_event);
        }
    }
}
