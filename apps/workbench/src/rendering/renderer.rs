use anyhow::{Context, Result, bail};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_12_1;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_ALPHA_MODE_UNSPECIFIED, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
};
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::Interface;

use crate::async_resident::AsyncTransactionReport;
use crate::cooked::CookedStreamer;
use crate::load::LoadConfig;
use crate::resident::StreamReport;
use crate::scene::SceneState;

use super::async_resident::AsyncResidentRenderer;
use super::device::{enable_debug_layer, select_reference_adapter, transition};
use super::gpu_capture::{CapturedPixels, Readback};
use super::load::{LoadProbe, LoadRenderer};
use super::resident::ResidentRenderer;
use super::scene_renderer::SceneRenderer;

const BUFFER_COUNT: usize = 2;

unsafe extern "C" {
    fn workbench_link_agility_exports();
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
    scene_renderer: SceneRenderer,
    pub(super) load_renderer: LoadRenderer,
    pub(super) resident_renderer: ResidentRenderer,
    pub(super) cooked_streamer: CookedStreamer,
    pub(super) async_resident_renderer: AsyncResidentRenderer,
    adapter_name: String,
    debug_layer: bool,
}

pub struct CapturedFrame {
    pub color: CapturedPixels,
    pub object_ids: Option<CapturedPixels>,
}

pub struct RenderOutcome {
    pub capture: Option<CapturedFrame>,
    pub load_probe: Option<LoadProbe>,
    pub resident_stream: Option<StreamReport>,
}

impl Renderer {
    pub unsafe fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        unsafe { workbench_link_agility_exports() };
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
        let scene_renderer = unsafe { SceneRenderer::new(&device, width, height) }?;
        let object_id_capture =
            unsafe { Readback::new(&device, scene_renderer.object_id_resource()) }?;
        let load_renderer =
            unsafe { LoadRenderer::new(&device, timestamp_frequency, width, height) }?;
        let resident_renderer =
            unsafe { ResidentRenderer::new(&device, timestamp_frequency, width, height) }?;
        let async_resident_renderer =
            unsafe { AsyncResidentRenderer::new(&device, timestamp_frequency, width, height) }?;

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
            scene_renderer,
            load_renderer,
            resident_renderer,
            cooked_streamer: CookedStreamer::default(),
            async_resident_renderer,
            adapter_name,
            debug_layer,
        })
    }

    pub unsafe fn render(
        &mut self,
        color: [f32; 4],
        capture: bool,
        capture_object_ids: bool,
        probe_load: bool,
        scene: &SceneState,
    ) -> Result<RenderOutcome> {
        debug_assert!(!capture_object_ids || capture);
        debug_assert!(!probe_load || self.load_config().is_some());
        unsafe { self.poll_cooked_completion()? };
        let stream_resident = self.resident_renderer.has_pending_stream();
        let index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() } as usize;
        unsafe { self.wait_for_buffer(index)? };
        unsafe { self.allocators[index].Reset() }.context("command allocator reset failed")?;
        unsafe { self.command_list.Reset(&self.allocators[index], None) }
            .context("command list reset failed")?;
        let publication = unsafe {
            self.async_resident_renderer
                .prepare_frame(&self.command_list)
        };
        if let Some(report) = publication.as_ref()
            && self.cooked_streamer.has_pending()
        {
            self.cooked_streamer.mark_published(report)?;
        }

        unsafe {
            transition(
                &self.command_list,
                &self.back_buffers[index],
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            );
        }
        let handle = self.rtv_handle(index);
        unsafe {
            self.command_list
                .OMSetRenderTargets(1, Some(&handle), true, None);
            self.command_list
                .ClearRenderTargetView(handle, &color, None);
            if self.async_resident_renderer.config().is_some() {
                self.async_resident_renderer.record(
                    &self.command_list,
                    scene,
                    [handle, self.scene_renderer.object_id_handle()],
                    self.scene_renderer.depth_handle(),
                    probe_load,
                )?;
            } else if self.resident_renderer.config().is_some() {
                self.resident_renderer.record(
                    &self.command_list,
                    scene,
                    [handle, self.scene_renderer.object_id_handle()],
                    self.scene_renderer.depth_handle(),
                    probe_load,
                )?;
            } else if self.load_renderer.config().is_some() {
                self.load_renderer.record(
                    &self.command_list,
                    scene,
                    [handle, self.scene_renderer.object_id_handle()],
                    self.scene_renderer.depth_handle(),
                    probe_load,
                )?;
            } else {
                self.scene_renderer
                    .record(&self.command_list, scene, handle);
            }
            if capture_object_ids {
                transition(
                    &self.command_list,
                    self.scene_renderer.object_id_resource(),
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                self.object_id_capture
                    .record(&self.command_list, self.scene_renderer.object_id_resource());
                transition(
                    &self.command_list,
                    self.scene_renderer.object_id_resource(),
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                );
            }
            if capture {
                transition(
                    &self.command_list,
                    &self.back_buffers[index],
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                self.capture
                    .record(&self.command_list, &self.back_buffers[index]);
                transition(
                    &self.command_list,
                    &self.back_buffers[index],
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_PRESENT,
                );
            } else {
                transition(
                    &self.command_list,
                    &self.back_buffers[index],
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_PRESENT,
                );
            }
            self.command_list.Close()
        }
        .context("command list close failed")?;

        let list: ID3D12CommandList = self.command_list.cast()?;
        unsafe {
            self.queue.ExecuteCommandLists(&[Some(list)]);
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()
        }
        .context("Present failed")?;

        let signal = self.next_fence_value;
        self.next_fence_value += 1;
        unsafe { self.queue.Signal(&self.fence, signal) }.context("queue signal failed")?;
        self.fence_values[index] = signal;
        if capture || probe_load || stream_resident {
            unsafe { self.wait_for_value(signal)? };
            let captured_frame = if capture {
                let color = unsafe { self.capture.read() }?;
                let object_ids = if capture_object_ids {
                    Some(unsafe { self.object_id_capture.read() }?)
                } else {
                    None
                };
                Some(CapturedFrame { color, object_ids })
            } else {
                None
            };
            let load_probe = if probe_load && self.async_resident_renderer.config().is_some() {
                Some(unsafe { self.async_resident_renderer.read_probe() }?)
            } else if probe_load && self.resident_renderer.config().is_some() {
                Some(unsafe { self.resident_renderer.read_probe() }?)
            } else if probe_load {
                Some(unsafe { self.load_renderer.read_probe() }?)
            } else {
                None
            };
            let resident_stream = if stream_resident {
                Some(self.resident_renderer.complete_stream()?)
            } else {
                None
            };
            return Ok(RenderOutcome {
                capture: captured_frame,
                load_probe,
                resident_stream,
            });
        }
        Ok(RenderOutcome {
            capture: None,
            load_probe: None,
            resident_stream: None,
        })
    }

    pub fn configure_load(&mut self, config: LoadConfig) -> Result<()> {
        self.async_resident_renderer.disable()?;
        self.resident_renderer.disable();
        self.load_renderer.configure(config);
        Ok(())
    }

    pub unsafe fn stream_resident(&mut self, config: LoadConfig) -> Result<()> {
        self.async_resident_renderer.disable()?;
        self.load_renderer.disable();
        unsafe { self.resident_renderer.prepare_stream(config) }
    }

    pub fn disable_load(&mut self) -> Result<()> {
        self.async_resident_renderer.disable()?;
        self.load_renderer.disable();
        self.resident_renderer.disable();
        Ok(())
    }

    pub fn load_config(&self) -> Option<LoadConfig> {
        self.async_resident_renderer
            .config()
            .or_else(|| self.resident_renderer.config())
            .or_else(|| self.load_renderer.config())
    }

    pub unsafe fn stream_async_resident(
        &mut self,
        config: LoadConfig,
    ) -> Result<AsyncTransactionReport> {
        self.load_renderer.disable();
        self.resident_renderer.disable();
        let release_fence = self.next_fence_value;
        self.next_fence_value += 1;
        unsafe {
            self.async_resident_renderer
                .schedule(config, &self.queue, &self.fence, release_fence)
        }
    }

    pub fn async_resident_status(&self) -> serde_json::Value {
        self.async_resident_renderer.status_json()
    }

    pub fn async_resident_config(&self) -> Option<LoadConfig> {
        self.async_resident_renderer.config()
    }

    pub fn async_resident_enabled(&self) -> bool {
        self.async_resident_renderer.is_enabled()
    }

    pub fn arm_async_copy_gate(&mut self) -> Result<u64> {
        self.async_resident_renderer.arm_gate()
    }

    pub unsafe fn release_async_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.async_resident_renderer.release_gate() }
    }

    pub fn resident_config(&self) -> Option<LoadConfig> {
        self.resident_renderer.config()
    }

    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub fn debug_layer(&self) -> bool {
        self.debug_layer
    }

    pub unsafe fn wait_idle(&mut self) -> Result<()> {
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
