use std::mem::ManuallyDrop;
use std::ptr;

use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D12::*;

const RESET_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/skeletal_scene.reset.dxil"));
const CULL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/skeletal_scene.cull.dxil"));
const COMPACT_SHADER: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/skeletal_scene.compact.dxil"));
const POSE_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/skeletal_scene.pose.dxil"));

pub const SKELETAL_CONSTANT_COUNT: u32 = 56;

pub struct SkeletalPipeline {
    pub root: ID3D12RootSignature,
    pub reset: ID3D12PipelineState,
    pub cull: ID3D12PipelineState,
    pub compact: ID3D12PipelineState,
    pub pose: ID3D12PipelineState,
    pub dispatch_signature: ID3D12CommandSignature,
}

impl SkeletalPipeline {
    pub unsafe fn new(device: &ID3D12Device) -> Result<Self> {
        let root = unsafe { create_root(device) }?;
        let reset = unsafe { create_compute_pipeline(device, &root, RESET_SHADER) }?;
        let cull = unsafe { create_compute_pipeline(device, &root, CULL_SHADER) }?;
        let compact = unsafe { create_compute_pipeline(device, &root, COMPACT_SHADER) }?;
        let pose = unsafe { create_compute_pipeline(device, &root, POSE_SHADER) }?;
        let dispatch_signature =
            unsafe { create_command_signature(device, D3D12_INDIRECT_ARGUMENT_TYPE_DISPATCH, 12) }?;
        Ok(Self {
            root,
            reset,
            cull,
            compact,
            pose,
            dispatch_signature,
        })
    }
}

unsafe fn create_root(device: &ID3D12Device) -> Result<ID3D12RootSignature> {
    let ranges = [
        descriptor_range(D3D12_DESCRIPTOR_RANGE_TYPE_SRV, 50, 0, 0),
        descriptor_range(D3D12_DESCRIPTOR_RANGE_TYPE_SRV, 4, 55, 55),
        descriptor_range(D3D12_DESCRIPTOR_RANGE_TYPE_UAV, 7, 0, 61),
        descriptor_range(D3D12_DESCRIPTOR_RANGE_TYPE_SRV, 50, 63, 68),
        descriptor_range(D3D12_DESCRIPTOR_RANGE_TYPE_UAV, 1, 7, 119),
        descriptor_range(D3D12_DESCRIPTOR_RANGE_TYPE_SRV, 50, 114, 120),
        descriptor_range(D3D12_DESCRIPTOR_RANGE_TYPE_SRV, 50, 164, 170),
    ];
    let parameters = [
        D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Constants: D3D12_ROOT_CONSTANTS {
                    ShaderRegister: 0,
                    RegisterSpace: 0,
                    Num32BitValues: SKELETAL_CONSTANT_COUNT,
                },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
        },
        D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                    NumDescriptorRanges: ranges.len() as u32,
                    pDescriptorRanges: ranges.as_ptr(),
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
    .context("skeletal root signature serialization failed")?;
    let blob: ID3DBlob = blob.context("skeletal root signature returned no blob")?;
    let bytes = unsafe {
        std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize())
    };
    unsafe { device.CreateRootSignature(0, bytes) }
        .context("skeletal root signature creation failed")
}

fn descriptor_range(
    kind: D3D12_DESCRIPTOR_RANGE_TYPE,
    count: u32,
    register: u32,
    offset: u32,
) -> D3D12_DESCRIPTOR_RANGE {
    D3D12_DESCRIPTOR_RANGE {
        RangeType: kind,
        NumDescriptors: count,
        BaseShaderRegister: register,
        RegisterSpace: 0,
        OffsetInDescriptorsFromTableStart: offset,
    }
}

unsafe fn create_compute_pipeline(
    device: &ID3D12Device,
    root: &ID3D12RootSignature,
    shader: &[u8],
) -> Result<ID3D12PipelineState> {
    let mut desc = D3D12_COMPUTE_PIPELINE_STATE_DESC {
        pRootSignature: ManuallyDrop::new(Some(root.clone())),
        CS: shader_bytecode(shader),
        ..Default::default()
    };
    let result = unsafe { device.CreateComputePipelineState(&desc) };
    unsafe { ManuallyDrop::drop(&mut desc.pRootSignature) };
    result.context("skeletal compute pipeline creation failed")
}

unsafe fn create_command_signature(
    device: &ID3D12Device,
    kind: D3D12_INDIRECT_ARGUMENT_TYPE,
    stride: u32,
) -> Result<ID3D12CommandSignature> {
    let argument = D3D12_INDIRECT_ARGUMENT_DESC {
        Type: kind,
        ..Default::default()
    };
    let desc = D3D12_COMMAND_SIGNATURE_DESC {
        ByteStride: stride,
        NumArgumentDescs: 1,
        pArgumentDescs: &argument,
        NodeMask: 0,
    };
    let mut signature = None;
    unsafe { device.CreateCommandSignature(&desc, None, &mut signature) }
        .context("skeletal command signature creation failed")?;
    signature.context("skeletal command signature returned no signature")
}

fn shader_bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}
