use std::mem::ManuallyDrop;
use std::ptr;

use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32_UINT, DXGI_FORMAT_UNKNOWN,
    DXGI_SAMPLE_DESC,
};

const RESET_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resident_load.reset.dxil"));
const CULL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resident_load.cull.dxil"));
const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resident_load.vs.dxil"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resident_load.ps.dxil"));
pub const RESIDENT_CONSTANT_COUNT: u32 = 18;

pub struct ResidentPipeline {
    pub compute_root: ID3D12RootSignature,
    pub reset: ID3D12PipelineState,
    pub cull: ID3D12PipelineState,
    pub graphics_root: ID3D12RootSignature,
    pub graphics: ID3D12PipelineState,
    pub command_signature: ID3D12CommandSignature,
}

impl ResidentPipeline {
    pub unsafe fn new(device: &ID3D12Device) -> Result<Self> {
        let compute_root = unsafe { create_compute_root(device) }?;
        let reset = unsafe { create_compute_pipeline(device, &compute_root, RESET_SHADER) }?;
        let cull = unsafe { create_compute_pipeline(device, &compute_root, CULL_SHADER) }?;
        let graphics_root = unsafe { create_graphics_root(device) }?;
        let graphics = unsafe { create_graphics_pipeline(device, &graphics_root) }?;
        let command_signature = unsafe { create_command_signature(device) }?;
        Ok(Self {
            compute_root,
            reset,
            cull,
            graphics_root,
            graphics,
            command_signature,
        })
    }
}

unsafe fn create_compute_root(device: &ID3D12Device) -> Result<ID3D12RootSignature> {
    let parameters = [
        root_constants(0, RESIDENT_CONSTANT_COUNT),
        root_descriptor(D3D12_ROOT_PARAMETER_TYPE_SRV, 0),
        root_descriptor(D3D12_ROOT_PARAMETER_TYPE_SRV, 1),
        root_descriptor(D3D12_ROOT_PARAMETER_TYPE_UAV, 0),
        root_descriptor(D3D12_ROOT_PARAMETER_TYPE_UAV, 1),
    ];
    unsafe { serialize_root(device, &parameters, D3D12_ROOT_SIGNATURE_FLAG_NONE) }
}

unsafe fn create_graphics_root(device: &ID3D12Device) -> Result<ID3D12RootSignature> {
    let parameters = [
        root_constants(0, RESIDENT_CONSTANT_COUNT),
        root_descriptor(D3D12_ROOT_PARAMETER_TYPE_SRV, 0),
        root_descriptor(D3D12_ROOT_PARAMETER_TYPE_SRV, 2),
    ];
    unsafe {
        serialize_root(
            device,
            &parameters,
            D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        )
    }
}

fn root_constants(register: u32, count: u32) -> D3D12_ROOT_PARAMETER {
    D3D12_ROOT_PARAMETER {
        ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
        Anonymous: D3D12_ROOT_PARAMETER_0 {
            Constants: D3D12_ROOT_CONSTANTS {
                ShaderRegister: register,
                RegisterSpace: 0,
                Num32BitValues: count,
            },
        },
        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
    }
}

fn root_descriptor(kind: D3D12_ROOT_PARAMETER_TYPE, register: u32) -> D3D12_ROOT_PARAMETER {
    D3D12_ROOT_PARAMETER {
        ParameterType: kind,
        Anonymous: D3D12_ROOT_PARAMETER_0 {
            Descriptor: D3D12_ROOT_DESCRIPTOR {
                ShaderRegister: register,
                RegisterSpace: 0,
            },
        },
        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
    }
}

unsafe fn serialize_root(
    device: &ID3D12Device,
    parameters: &[D3D12_ROOT_PARAMETER],
    flags: D3D12_ROOT_SIGNATURE_FLAGS,
) -> Result<ID3D12RootSignature> {
    let desc = D3D12_ROOT_SIGNATURE_DESC {
        NumParameters: parameters.len() as u32,
        pParameters: parameters.as_ptr(),
        NumStaticSamplers: 0,
        pStaticSamplers: ptr::null(),
        Flags: flags,
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
    .context("resident root signature serialization failed")?;
    let blob: ID3DBlob = blob.context("resident root signature serialization returned no blob")?;
    let bytes = unsafe {
        std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize())
    };
    unsafe { device.CreateRootSignature(0, bytes) }
        .context("resident root signature creation failed")
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
    result.context("resident compute pipeline creation failed")
}

unsafe fn create_graphics_pipeline(
    device: &ID3D12Device,
    root: &ID3D12RootSignature,
) -> Result<ID3D12PipelineState> {
    let target_blend = D3D12_RENDER_TARGET_BLEND_DESC {
        BlendEnable: false.into(),
        LogicOpEnable: false.into(),
        SrcBlend: D3D12_BLEND_ONE,
        DestBlend: D3D12_BLEND_ZERO,
        BlendOp: D3D12_BLEND_OP_ADD,
        SrcBlendAlpha: D3D12_BLEND_ONE,
        DestBlendAlpha: D3D12_BLEND_ZERO,
        BlendOpAlpha: D3D12_BLEND_OP_ADD,
        LogicOp: D3D12_LOGIC_OP_NOOP,
        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
    };
    let mut formats = [DXGI_FORMAT_UNKNOWN; 8];
    formats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
    formats[1] = DXGI_FORMAT_R32_UINT;
    let mut desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
        pRootSignature: ManuallyDrop::new(Some(root.clone())),
        VS: shader_bytecode(VERTEX_SHADER),
        PS: shader_bytecode(PIXEL_SHADER),
        BlendState: D3D12_BLEND_DESC {
            AlphaToCoverageEnable: false.into(),
            IndependentBlendEnable: false.into(),
            RenderTarget: [target_blend; 8],
        },
        SampleMask: u32::MAX,
        RasterizerState: D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_NONE,
            DepthClipEnable: true.into(),
            ..Default::default()
        },
        DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: true.into(),
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_GREATER,
            StencilEnable: false.into(),
            ..Default::default()
        },
        PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        NumRenderTargets: 2,
        RTVFormats: formats,
        DSVFormat: DXGI_FORMAT_D32_FLOAT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        ..Default::default()
    };
    let result = unsafe { device.CreateGraphicsPipelineState(&desc) };
    unsafe { ManuallyDrop::drop(&mut desc.pRootSignature) };
    result.context("resident graphics pipeline creation failed")
}

unsafe fn create_command_signature(device: &ID3D12Device) -> Result<ID3D12CommandSignature> {
    let argument = D3D12_INDIRECT_ARGUMENT_DESC {
        Type: D3D12_INDIRECT_ARGUMENT_TYPE_DRAW,
        ..Default::default()
    };
    let desc = D3D12_COMMAND_SIGNATURE_DESC {
        ByteStride: std::mem::size_of::<D3D12_DRAW_ARGUMENTS>() as u32,
        NumArgumentDescs: 1,
        pArgumentDescs: &argument,
        NodeMask: 0,
    };
    let mut signature = None;
    unsafe { device.CreateCommandSignature(&desc, None, &mut signature) }
        .context("resident command signature creation failed")?;
    signature.context("resident command signature creation returned no signature")
}

fn shader_bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}
