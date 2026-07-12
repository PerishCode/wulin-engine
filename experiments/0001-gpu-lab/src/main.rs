use std::cmp::min;
use std::ffi::CStr;
use std::fs;
use std::mem::{ManuallyDrop, size_of};
use std::path::PathBuf;
use std::ptr;
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Graphics::Direct3D::{D3D_FEATURE_LEVEL_12_1, ID3DBlob};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows::Win32::System::Threading::{CreateEventW, INFINITE, WaitForSingleObject};
use windows::core::{Interface, PCWSTR, w};

const THREAD_GROUP_SIZE: u32 = 256;
const MAX_DISPATCH_GROUPS: u32 = 65_535;
const NVIDIA_VENDOR_ID: u32 = 0x10de;
const SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fill.dxil"));

unsafe extern "C" {
    fn gpu_lab_link_agility_exports();
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Correctness,
    Benchmark,
}

#[derive(Debug)]
struct Config {
    mode: Mode,
    elements: u32,
    warmup: u32,
    iterations: u32,
    seed: u32,
    report: Option<PathBuf>,
}

#[derive(Serialize)]
struct Report {
    schema_version: u32,
    outcome: &'static str,
    mode: Mode,
    revision: &'static str,
    rustc: &'static str,
    dxc: &'static str,
    agility_package: &'static str,
    agility_sdk: &'static str,
    d3d12_core_path: String,
    adapter: String,
    dedicated_video_memory_bytes: usize,
    elements: u32,
    warmup_iterations: u32,
    measured_iterations: u32,
    seed: u32,
    debug_layer_enabled: bool,
    gpu_validation_enabled: bool,
    enhanced_barriers: bool,
    timestamp_frequency_hz: u64,
    cpu_record_ms: f64,
    cpu_submit_wait_ms: f64,
    cpu_validate_ms: f64,
    gpu_ms: Distribution,
    actual_checksum: String,
    expected_checksum: String,
    mismatch_count: usize,
    d3d12_errors: Vec<String>,
}

#[derive(Serialize)]
struct Distribution {
    minimum: f64,
    median: f64,
    p95: f64,
    p99: f64,
    maximum: f64,
}

fn main() -> Result<()> {
    let config = parse_args()?;
    let report_path = config.report.clone();
    let report = unsafe { run(&config) }?;
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");

    if let Some(path) = report_path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&path, format!("{json}\n"))
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    if report.outcome != "pass" {
        bail!("GPU laboratory acceptance criteria failed");
    }
    Ok(())
}

unsafe fn run(config: &Config) -> Result<Report> {
    unsafe { gpu_lab_link_agility_exports() };
    let _debug = if matches!(config.mode, Mode::Correctness) {
        unsafe { enable_debug_layer()? }
    } else {
        None
    };

    let factory_flags = if matches!(config.mode, Mode::Correctness) {
        DXGI_CREATE_FACTORY_DEBUG
    } else {
        DXGI_CREATE_FACTORY_FLAGS(0)
    };
    let factory: IDXGIFactory6 =
        unsafe { CreateDXGIFactory2(factory_flags) }.context("CreateDXGIFactory2 failed")?;
    let (adapter, adapter_desc) = unsafe { select_reference_adapter(&factory) }?;
    let adapter_name = wide_string(&adapter_desc.Description);

    let mut device = None;
    unsafe { D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_1, &mut device) }
        .context("D3D12CreateDevice failed")?;
    let device: ID3D12Device = device.context("D3D12CreateDevice returned no device")?;
    let d3d12_core_path = unsafe { loaded_module_path("D3D12Core.dll") }?;

    let mut options12 = D3D12_FEATURE_DATA_D3D12_OPTIONS12::default();
    unsafe {
        device.CheckFeatureSupport(
            D3D12_FEATURE_D3D12_OPTIONS12,
            ptr::addr_of_mut!(options12).cast(),
            size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS12>() as u32,
        )
    }
    .context("querying D3D12_OPTIONS12 failed")?;
    if !options12.EnhancedBarriersSupported.as_bool() {
        bail!("the reference adapter does not support Enhanced Barriers");
    }

    let info_queue = if matches!(config.mode, Mode::Correctness) {
        let queue: ID3D12InfoQueue = device.cast().context("ID3D12InfoQueue is unavailable")?;
        unsafe { queue.ClearStoredMessages() };
        Some(queue)
    } else {
        None
    };

    let queue_desc = D3D12_COMMAND_QUEUE_DESC {
        Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
        Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
        Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
        NodeMask: 0,
    };
    let queue: ID3D12CommandQueue =
        unsafe { device.CreateCommandQueue(&queue_desc) }.context("CreateCommandQueue failed")?;
    let timestamp_frequency =
        unsafe { queue.GetTimestampFrequency() }.context("GetTimestampFrequency failed")?;

    let root_signature = unsafe { create_root_signature(&device) }?;
    let pipeline = unsafe { create_pipeline(&device, &root_signature) }?;
    let output_size = u64::from(config.elements)
        .checked_mul(size_of::<u32>() as u64)
        .context("output buffer size overflow")?;
    let output = unsafe {
        create_buffer(
            &device,
            output_size,
            D3D12_HEAP_TYPE_DEFAULT,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
        )
    }?;
    let output_readback = unsafe {
        create_buffer(
            &device,
            output_size,
            D3D12_HEAP_TYPE_READBACK,
            D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_FLAG_NONE,
        )
    }?;

    let total_iterations = config
        .warmup
        .checked_add(config.iterations)
        .context("iteration count overflow")?;
    let query_count = total_iterations
        .checked_mul(2)
        .context("query count overflow")?;
    let query_heap_desc = D3D12_QUERY_HEAP_DESC {
        Type: D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
        Count: query_count,
        NodeMask: 0,
    };
    let mut query_heap = None;
    unsafe { device.CreateQueryHeap(&query_heap_desc, &mut query_heap) }
        .context("CreateQueryHeap failed")?;
    let query_heap: ID3D12QueryHeap =
        query_heap.context("CreateQueryHeap returned no query heap")?;
    let timestamp_size = u64::from(query_count) * size_of::<u64>() as u64;
    let timestamp_readback = unsafe {
        create_buffer(
            &device,
            timestamp_size,
            D3D12_HEAP_TYPE_READBACK,
            D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_FLAG_NONE,
        )
    }?;

    let allocator: ID3D12CommandAllocator =
        unsafe { device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT) }
            .context("CreateCommandAllocator failed")?;
    let command_list: ID3D12GraphicsCommandList7 = unsafe {
        device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &allocator, &pipeline)
    }
    .context("CreateCommandList failed")?;

    let (groups_x, groups_y, row_stride) = dispatch_shape(config.elements)?;
    let record_start = Instant::now();
    unsafe {
        command_list.SetComputeRootSignature(&root_signature);
        command_list.SetPipelineState(&pipeline);
        command_list.SetComputeRootUnorderedAccessView(0, output.GetGPUVirtualAddress());
    }

    for iteration in 0..total_iterations {
        let constants = [
            config.elements,
            config.seed.wrapping_add(iteration),
            row_stride,
        ];
        let query = iteration * 2;
        unsafe {
            command_list.SetComputeRoot32BitConstants(
                1,
                constants.len() as u32,
                constants.as_ptr().cast(),
                0,
            );
            command_list.EndQuery(&query_heap, D3D12_QUERY_TYPE_TIMESTAMP, query);
            command_list.Dispatch(groups_x, groups_y, 1);
            command_list.EndQuery(&query_heap, D3D12_QUERY_TYPE_TIMESTAMP, query + 1);
        }
    }

    unsafe { transition_uav_to_copy(&command_list, &output, output_size) };
    unsafe {
        command_list.CopyBufferRegion(&output_readback, 0, &output, 0, output_size);
        command_list.ResolveQueryData(
            &query_heap,
            D3D12_QUERY_TYPE_TIMESTAMP,
            0,
            query_count,
            &timestamp_readback,
            0,
        );
        command_list.Close()
    }
    .context("closing the command list failed")?;
    let cpu_record_ms = record_start.elapsed().as_secs_f64() * 1_000.0;

    let fence: ID3D12Fence =
        unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }.context("CreateFence failed")?;
    let base_list: ID3D12CommandList = command_list.cast()?;
    let submit_start = Instant::now();
    unsafe {
        queue.ExecuteCommandLists(&[Some(base_list)]);
        queue.Signal(&fence, 1)
    }
    .context("queue signal failed")?;
    unsafe { wait_for_fence(&fence, 1) }?;
    unsafe { device.GetDeviceRemovedReason() }.context("device was removed")?;
    let cpu_submit_wait_ms = submit_start.elapsed().as_secs_f64() * 1_000.0;

    let timestamps = unsafe { readback_u64(&timestamp_readback, query_count as usize) }?;
    let measured_offset = config.warmup as usize * 2;
    let measured_gpu_ms = timestamps[measured_offset..]
        .chunks_exact(2)
        .map(|pair| pair[1].saturating_sub(pair[0]) as f64 * 1_000.0 / timestamp_frequency as f64)
        .collect::<Vec<_>>();
    let gpu_ms = distribution(&measured_gpu_ms)?;

    let validate_start = Instant::now();
    let actual = unsafe { readback_u32(&output_readback, config.elements as usize) }?;
    let final_seed = config.seed.wrapping_add(total_iterations - 1);
    let mut expected_checksum = Fnv64::new();
    let mut actual_checksum = Fnv64::new();
    let mut mismatch_count = 0usize;
    for (index, actual_value) in actual.iter().copied().enumerate() {
        let expected_value = hash_u32(index as u32 ^ final_seed);
        expected_checksum.add(expected_value);
        actual_checksum.add(actual_value);
        mismatch_count += usize::from(actual_value != expected_value);
    }
    let cpu_validate_ms = validate_start.elapsed().as_secs_f64() * 1_000.0;

    let d3d12_errors = if let Some(info_queue) = &info_queue {
        unsafe { collect_d3d12_errors(info_queue) }?
    } else {
        Vec::new()
    };
    let timestamps_valid = measured_gpu_ms
        .iter()
        .all(|value| value.is_finite() && *value > 0.0);
    let outcome = if mismatch_count == 0 && d3d12_errors.is_empty() && timestamps_valid {
        "pass"
    } else {
        "fail"
    };

    Ok(Report {
        schema_version: 1,
        outcome,
        mode: config.mode,
        revision: env!("GPU_LAB_GIT_REVISION"),
        rustc: env!("GPU_LAB_RUSTC_VERSION"),
        dxc: env!("GPU_LAB_DXC_VERSION"),
        agility_package: env!("GPU_LAB_AGILITY_PACKAGE"),
        agility_sdk: env!("GPU_LAB_AGILITY_SDK"),
        d3d12_core_path,
        adapter: adapter_name,
        dedicated_video_memory_bytes: adapter_desc.DedicatedVideoMemory,
        elements: config.elements,
        warmup_iterations: config.warmup,
        measured_iterations: config.iterations,
        seed: config.seed,
        debug_layer_enabled: info_queue.is_some(),
        gpu_validation_enabled: info_queue.is_some(),
        enhanced_barriers: options12.EnhancedBarriersSupported.as_bool(),
        timestamp_frequency_hz: timestamp_frequency,
        cpu_record_ms,
        cpu_submit_wait_ms,
        cpu_validate_ms,
        gpu_ms,
        actual_checksum: format!("{:016x}", actual_checksum.finish()),
        expected_checksum: format!("{:016x}", expected_checksum.finish()),
        mismatch_count,
        d3d12_errors,
    })
}

unsafe fn loaded_module_path(name: &str) -> Result<String> {
    let module = match name {
        "D3D12Core.dll" => unsafe { GetModuleHandleW(w!("D3D12Core.dll")) },
        _ => unreachable!(),
    }
    .with_context(|| format!("{name} is not loaded"))?;
    let mut path = vec![0u16; 32_768];
    let length = unsafe { GetModuleFileNameW(Some(module), &mut path) } as usize;
    if length == 0 || length == path.len() {
        bail!("failed to resolve the loaded {name} path");
    }
    Ok(String::from_utf16_lossy(&path[..length]))
}

unsafe fn enable_debug_layer() -> Result<Option<ID3D12Debug1>> {
    let mut debug = None;
    unsafe { D3D12GetDebugInterface(&mut debug) }.context("D3D12 debug layer is unavailable")?;
    let debug: ID3D12Debug = debug.context("D3D12 debug interface was empty")?;
    let debug: ID3D12Debug1 = debug
        .cast()
        .context("ID3D12Debug1 is unavailable from the debug layer")?;
    unsafe {
        debug.EnableDebugLayer();
        debug.SetEnableGPUBasedValidation(true);
        debug.SetEnableSynchronizedCommandQueueValidation(true);
    }
    Ok(Some(debug))
}

unsafe fn select_reference_adapter(
    factory: &IDXGIFactory6,
) -> Result<(IDXGIAdapter4, DXGI_ADAPTER_DESC3)> {
    for index in 0..16 {
        let adapter = unsafe {
            factory.EnumAdapterByGpuPreference::<IDXGIAdapter4>(
                index,
                DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE,
            )
        };
        let Ok(adapter) = adapter else {
            break;
        };
        let desc = unsafe { adapter.GetDesc3() }?;
        if desc.VendorId == NVIDIA_VENDOR_ID {
            return Ok((adapter, desc));
        }
    }
    bail!("no NVIDIA adapter was found on the reference platform")
}

unsafe fn create_root_signature(device: &ID3D12Device) -> Result<ID3D12RootSignature> {
    let parameters = [
        D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_UAV,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Descriptor: D3D12_ROOT_DESCRIPTOR {
                    ShaderRegister: 0,
                    RegisterSpace: 0,
                },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
        },
        D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Constants: D3D12_ROOT_CONSTANTS {
                    ShaderRegister: 0,
                    RegisterSpace: 0,
                    Num32BitValues: 3,
                },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
        },
    ];
    let desc = D3D12_ROOT_SIGNATURE_DESC {
        NumParameters: parameters.len() as u32,
        pParameters: parameters.as_ptr(),
        NumStaticSamplers: 0,
        pStaticSamplers: ptr::null(),
        Flags: D3D12_ROOT_SIGNATURE_FLAG_NONE,
    };
    let mut blob = None;
    let mut errors = None;
    unsafe {
        D3D12SerializeRootSignature(
            &desc,
            D3D_ROOT_SIGNATURE_VERSION_1,
            &mut blob,
            Some(&mut errors),
        )
    }
    .map_err(|error| {
        let details = errors
            .and_then(|blob: ID3DBlob| unsafe { blob_text(&blob).ok() })
            .unwrap_or_else(|| "no serializer diagnostics".into());
        anyhow!("root signature serialization failed: {error}: {details}")
    })?;
    let blob = blob.context("root signature serialization returned no blob")?;
    let bytes = unsafe {
        std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize())
    };
    unsafe { device.CreateRootSignature(0, bytes) }.context("CreateRootSignature failed")
}

unsafe fn create_pipeline(
    device: &ID3D12Device,
    root_signature: &ID3D12RootSignature,
) -> Result<ID3D12PipelineState> {
    let mut desc = D3D12_COMPUTE_PIPELINE_STATE_DESC {
        pRootSignature: ManuallyDrop::new(Some(root_signature.clone())),
        CS: D3D12_SHADER_BYTECODE {
            pShaderBytecode: SHADER.as_ptr().cast(),
            BytecodeLength: SHADER.len(),
        },
        ..Default::default()
    };
    let result = unsafe { device.CreateComputePipelineState(&desc) };
    unsafe { ManuallyDrop::drop(&mut desc.pRootSignature) };
    result.context("CreateComputePipelineState failed")
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
    .context("CreateCommittedResource failed")?;
    resource.context("CreateCommittedResource returned no resource")
}

unsafe fn transition_uav_to_copy(
    command_list: &ID3D12GraphicsCommandList7,
    resource: &ID3D12Resource,
    size: u64,
) {
    let mut barrier = D3D12_BUFFER_BARRIER {
        SyncBefore: D3D12_BARRIER_SYNC_COMPUTE_SHADING,
        SyncAfter: D3D12_BARRIER_SYNC_COPY,
        AccessBefore: D3D12_BARRIER_ACCESS_UNORDERED_ACCESS,
        AccessAfter: D3D12_BARRIER_ACCESS_COPY_SOURCE,
        pResource: ManuallyDrop::new(Some(resource.clone())),
        Offset: 0,
        Size: size,
    };
    let group = D3D12_BARRIER_GROUP {
        Type: D3D12_BARRIER_TYPE_BUFFER,
        NumBarriers: 1,
        Anonymous: D3D12_BARRIER_GROUP_0 {
            pBufferBarriers: &barrier,
        },
    };
    unsafe { command_list.Barrier(&[group]) };
    unsafe { ManuallyDrop::drop(&mut barrier.pResource) };
}

unsafe fn wait_for_fence(fence: &ID3D12Fence, value: u64) -> Result<()> {
    if unsafe { fence.GetCompletedValue() } >= value {
        return Ok(());
    }
    let event: HANDLE = unsafe { CreateEventW(None, false, false, PCWSTR::null()) }
        .context("CreateEventW failed")?;
    let wait_result = (|| -> Result<()> {
        unsafe { fence.SetEventOnCompletion(value, event) }
            .context("SetEventOnCompletion failed")?;
        let result = unsafe { WaitForSingleObject(event, INFINITE) };
        if result != WAIT_OBJECT_0 {
            bail!("WaitForSingleObject returned {result:?}");
        }
        Ok(())
    })();
    unsafe { CloseHandle(event) }.context("CloseHandle failed")?;
    wait_result
}

unsafe fn readback_u32(resource: &ID3D12Resource, count: usize) -> Result<Vec<u32>> {
    let mut data = ptr::null_mut();
    let range = D3D12_RANGE {
        Begin: 0,
        End: count * size_of::<u32>(),
    };
    unsafe { resource.Map(0, Some(&range), Some(&mut data)) }.context("mapping output failed")?;
    let mut output = vec![0u32; count];
    unsafe { ptr::copy_nonoverlapping(data.cast::<u32>(), output.as_mut_ptr(), count) };
    let written = D3D12_RANGE { Begin: 0, End: 0 };
    unsafe { resource.Unmap(0, Some(&written)) };
    Ok(output)
}

unsafe fn readback_u64(resource: &ID3D12Resource, count: usize) -> Result<Vec<u64>> {
    let mut data = ptr::null_mut();
    let range = D3D12_RANGE {
        Begin: 0,
        End: count * size_of::<u64>(),
    };
    unsafe { resource.Map(0, Some(&range), Some(&mut data)) }
        .context("mapping timestamps failed")?;
    let mut output = vec![0u64; count];
    unsafe { ptr::copy_nonoverlapping(data.cast::<u64>(), output.as_mut_ptr(), count) };
    let written = D3D12_RANGE { Begin: 0, End: 0 };
    unsafe { resource.Unmap(0, Some(&written)) };
    Ok(output)
}

unsafe fn collect_d3d12_errors(info_queue: &ID3D12InfoQueue) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    let count = unsafe { info_queue.GetNumStoredMessages() };
    for index in 0..count {
        let mut byte_count = 0usize;
        unsafe { info_queue.GetMessage(index, None, &mut byte_count) }?;
        let words = byte_count.div_ceil(size_of::<usize>());
        let mut storage = vec![0usize; words];
        let message = storage.as_mut_ptr().cast::<D3D12_MESSAGE>();
        unsafe { info_queue.GetMessage(index, Some(message), &mut byte_count) }?;
        let message = unsafe { &*message };
        if message.Severity == D3D12_MESSAGE_SEVERITY_ERROR
            || message.Severity == D3D12_MESSAGE_SEVERITY_CORRUPTION
        {
            let description = if message.pDescription.is_null() {
                "<no description>".into()
            } else {
                unsafe { CStr::from_ptr(message.pDescription.cast()) }
                    .to_string_lossy()
                    .into_owned()
            };
            errors.push(format!(
                "{:?} {:?}: {description}",
                message.Severity, message.ID
            ));
        }
    }
    Ok(errors)
}

unsafe fn blob_text(blob: &ID3DBlob) -> Result<String> {
    let bytes = unsafe {
        std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize())
    };
    Ok(String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .to_owned())
}

fn dispatch_shape(elements: u32) -> Result<(u32, u32, u32)> {
    let group_count = elements.div_ceil(THREAD_GROUP_SIZE);
    let groups_x = min(group_count, MAX_DISPATCH_GROUPS);
    let groups_y = group_count.div_ceil(groups_x);
    if groups_y > MAX_DISPATCH_GROUPS {
        bail!("element count exceeds the two-dimensional dispatch capacity");
    }
    Ok((groups_x, groups_y, groups_x * THREAD_GROUP_SIZE))
}

fn distribution(values: &[f64]) -> Result<Distribution> {
    if values.is_empty() {
        bail!("at least one measured iteration is required");
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Ok(Distribution {
        minimum: sorted[0],
        median: percentile(&sorted, 0.50),
        p95: percentile(&sorted, 0.95),
        p99: percentile(&sorted, 0.99),
        maximum: *sorted.last().unwrap(),
    })
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    let index = ((sorted.len() - 1) as f64 * percentile).ceil() as usize;
    sorted[index]
}

fn hash_u32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    value
}

struct Fnv64(u64);

impl Fnv64 {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn add(&mut self, value: u32) {
        for byte in value.to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

fn wide_string(value: &[u16]) -> String {
    let end = value
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..end])
}

fn parse_args() -> Result<Config> {
    let mut mode = Mode::Correctness;
    let mut elements = 1_048_576u32;
    let mut warmup = None;
    let mut iterations = None;
    let mut seed = 0x1234_5678u32;
    let mut report = None;
    let mut args = std::env::args().skip(1);

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--mode" => {
                mode = match next_value(&mut args, "--mode")?.as_str() {
                    "correctness" => Mode::Correctness,
                    "benchmark" => Mode::Benchmark,
                    value => bail!("unsupported mode {value:?}"),
                };
            }
            "--elements" => elements = parse_value(&mut args, "--elements")?,
            "--warmup" => warmup = Some(parse_value(&mut args, "--warmup")?),
            "--iterations" => iterations = Some(parse_value(&mut args, "--iterations")?),
            "--seed" => seed = parse_u32(&next_value(&mut args, "--seed")?)?,
            "--report" => report = Some(PathBuf::from(next_value(&mut args, "--report")?)),
            "--help" | "-h" => {
                println!(
                    "gpu-lab [--mode correctness|benchmark] [--elements N] [--warmup N] \\\n+                     [--iterations N] [--seed N|0xHEX] [--report PATH]"
                );
                std::process::exit(0);
            }
            value => bail!("unknown argument {value:?}"),
        }
    }

    if elements == 0 {
        bail!("--elements must be greater than zero");
    }
    let default_warmup = if matches!(mode, Mode::Benchmark) {
        10
    } else {
        1
    };
    let default_iterations = if matches!(mode, Mode::Benchmark) {
        100
    } else {
        1
    };
    let warmup = warmup.unwrap_or(default_warmup);
    let iterations = iterations.unwrap_or(default_iterations);
    if iterations == 0 {
        bail!("--iterations must be greater than zero");
    }

    Ok(Config {
        mode,
        elements,
        warmup,
        iterations,
        seed,
        report,
    })
}

fn next_value(args: &mut impl Iterator<Item = String>, option: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{option} requires a value"))
}

fn parse_value<T>(args: &mut impl Iterator<Item = String>, option: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    next_value(args, option)?
        .parse()
        .with_context(|| format!("invalid value for {option}"))
}

fn parse_u32(value: &str) -> Result<u32> {
    if let Some(hex) = value.strip_prefix("0x") {
        Ok(u32::from_str_radix(hex, 16)?)
    } else {
        Ok(value.parse()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_shape_stays_within_d3d12_limits() {
        assert_eq!(dispatch_shape(1).unwrap(), (1, 1, THREAD_GROUP_SIZE));
        assert_eq!(
            dispatch_shape(MAX_DISPATCH_GROUPS * THREAD_GROUP_SIZE).unwrap(),
            (
                MAX_DISPATCH_GROUPS,
                1,
                MAX_DISPATCH_GROUPS * THREAD_GROUP_SIZE
            )
        );
        assert_eq!(
            dispatch_shape(MAX_DISPATCH_GROUPS * THREAD_GROUP_SIZE + 1).unwrap(),
            (
                MAX_DISPATCH_GROUPS,
                2,
                MAX_DISPATCH_GROUPS * THREAD_GROUP_SIZE
            )
        );
    }

    #[test]
    fn hash_and_distribution_are_stable() {
        assert_eq!(hash_u32(0), 0);
        assert_ne!(hash_u32(1), hash_u32(2));
        let values = [4.0, 1.0, 3.0, 2.0];
        let result = distribution(&values).unwrap();
        assert_eq!(result.minimum, 1.0);
        assert_eq!(result.median, 3.0);
        assert_eq!(result.maximum, 4.0);
    }

    #[test]
    fn seed_parser_accepts_decimal_and_hex() {
        assert_eq!(parse_u32("42").unwrap(), 42);
        assert_eq!(parse_u32("0x2a").unwrap(), 42);
    }
}
