use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr;

use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT, DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R32_UINT,
    DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC,
};
use windows::core::Interface;

const RESET_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/terrain.reset.dxil"));
const SEAM_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/terrain.seam.dxil"));
const AMPLIFICATION_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/terrain.as.dxil"));
const MESH_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/terrain.ms.dxil"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/terrain.ps.dxil"));

pub const TERRAIN_CONSTANT_COUNT: u32 = 48;

pub struct TerrainPipeline {
    pub root: ID3D12RootSignature,
    pub reset: ID3D12PipelineState,
    pub seam: ID3D12PipelineState,
    pub graphics: ID3D12PipelineState,
}

impl TerrainPipeline {
    pub unsafe fn new(device: &ID3D12Device) -> Result<Self> {
        let root = unsafe { create_root(device) }?;
        let reset = unsafe { create_compute_pipeline(device, &root, RESET_SHADER) }?;
        let seam = unsafe { create_compute_pipeline(device, &root, SEAM_SHADER) }?;
        let graphics = unsafe { create_mesh_pipeline(device, &root) }?;
        Ok(Self {
            root,
            reset,
            seam,
            graphics,
        })
    }
}

unsafe fn create_root(device: &ID3D12Device) -> Result<ID3D12RootSignature> {
    let ranges = [
        D3D12_DESCRIPTOR_RANGE {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            NumDescriptors: 50,
            BaseShaderRegister: 0,
            RegisterSpace: 0,
            OffsetInDescriptorsFromTableStart: 0,
        },
        D3D12_DESCRIPTOR_RANGE {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: 2,
            BaseShaderRegister: 0,
            RegisterSpace: 0,
            OffsetInDescriptorsFromTableStart: 50,
        },
    ];
    let parameters = [
        D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Constants: D3D12_ROOT_CONSTANTS {
                    ShaderRegister: 0,
                    RegisterSpace: 0,
                    Num32BitValues: TERRAIN_CONSTANT_COUNT,
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
    .context("terrain root signature serialization failed")?;
    let blob: ID3DBlob = blob.context("terrain root signature returned no blob")?;
    let bytes = unsafe {
        std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize())
    };
    unsafe { device.CreateRootSignature(0, bytes) }
        .context("terrain root signature creation failed")
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
    result.context("terrain compute pipeline creation failed")
}

#[repr(C, align(8))]
struct Subobject<T> {
    kind: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    value: T,
}

impl<T> Subobject<T> {
    fn new(kind: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE, value: T) -> Self {
        Self { kind, value }
    }
}

#[repr(C)]
struct MeshPipelineStream {
    root: Subobject<*mut c_void>,
    amplification: Subobject<D3D12_SHADER_BYTECODE>,
    mesh: Subobject<D3D12_SHADER_BYTECODE>,
    pixel: Subobject<D3D12_SHADER_BYTECODE>,
    blend: Subobject<D3D12_BLEND_DESC>,
    sample_mask: Subobject<u32>,
    rasterizer: Subobject<D3D12_RASTERIZER_DESC>,
    depth_stencil: Subobject<D3D12_DEPTH_STENCIL_DESC>,
    topology: Subobject<D3D12_PRIMITIVE_TOPOLOGY_TYPE>,
    render_targets: Subobject<D3D12_RT_FORMAT_ARRAY>,
    depth_format: Subobject<DXGI_FORMAT>,
    sample: Subobject<DXGI_SAMPLE_DESC>,
}

unsafe fn create_mesh_pipeline(
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
    let mut stream = MeshPipelineStream {
        root: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_ROOT_SIGNATURE,
            root.as_raw(),
        ),
        amplification: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_AS,
            shader_bytecode(AMPLIFICATION_SHADER),
        ),
        mesh: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_MS,
            shader_bytecode(MESH_SHADER),
        ),
        pixel: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PS,
            shader_bytecode(PIXEL_SHADER),
        ),
        blend: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_BLEND,
            D3D12_BLEND_DESC {
                AlphaToCoverageEnable: false.into(),
                IndependentBlendEnable: false.into(),
                RenderTarget: [target_blend; 8],
            },
        ),
        sample_mask: Subobject::new(D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_MASK, u32::MAX),
        rasterizer: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER,
            D3D12_RASTERIZER_DESC {
                FillMode: D3D12_FILL_MODE_SOLID,
                CullMode: D3D12_CULL_MODE_NONE,
                DepthClipEnable: true.into(),
                ..Default::default()
            },
        ),
        depth_stencil: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL,
            D3D12_DEPTH_STENCIL_DESC {
                DepthEnable: true.into(),
                DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
                DepthFunc: D3D12_COMPARISON_FUNC_GREATER,
                StencilEnable: false.into(),
                ..Default::default()
            },
        ),
        topology: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY,
            D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        ),
        render_targets: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RENDER_TARGET_FORMATS,
            D3D12_RT_FORMAT_ARRAY {
                RTFormats: formats,
                NumRenderTargets: 2,
            },
        ),
        depth_format: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT,
            DXGI_FORMAT_D32_FLOAT,
        ),
        sample: Subobject::new(
            D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_DESC,
            DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
        ),
    };
    let desc = D3D12_PIPELINE_STATE_STREAM_DESC {
        SizeInBytes: size_of::<MeshPipelineStream>(),
        pPipelineStateSubobjectStream: (&raw mut stream).cast(),
    };
    let device: ID3D12Device2 = device.cast().context("ID3D12Device2 is unavailable")?;
    unsafe { device.CreatePipelineState(&raw const desc) }
        .context("terrain graphics pipeline creation failed")
}

fn shader_bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}
