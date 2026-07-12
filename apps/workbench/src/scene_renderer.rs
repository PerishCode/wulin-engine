use std::mem::{ManuallyDrop, size_of};
use std::ptr;

use anyhow::{Context, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D::{D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST, ID3DBlob};
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R16_UINT,
    DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC,
};
use windows::core::PCSTR;

use crate::scene::{MeshKind, OBJECTS, SceneState};

const VERTEX_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/calibration.vs.dxil"));
const PIXEL_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/calibration.ps.dxil"));
const FRAME_CONSTANT_COUNT: u32 = 16;
const DRAW_CONSTANT_COUNT: u32 = 21;

pub struct SceneRenderer {
    root_signature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    geometry: Geometry,
    _depth: ID3D12Resource,
    dsv_heap: ID3D12DescriptorHeap,
    width: u32,
    height: u32,
}

struct Geometry {
    _vertices: ID3D12Resource,
    _indices: ID3D12Resource,
    vertex_view: D3D12_VERTEX_BUFFER_VIEW,
    index_view: D3D12_INDEX_BUFFER_VIEW,
    plane: MeshRange,
    cube: MeshRange,
}

#[derive(Clone, Copy)]
struct MeshRange {
    index_count: u32,
    start_index: u32,
    base_vertex: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl SceneRenderer {
    pub unsafe fn new(device: &ID3D12Device, width: u32, height: u32) -> Result<Self> {
        let root_signature = unsafe { create_root_signature(device) }?;
        let pipeline = unsafe { create_pipeline(device, &root_signature) }?;
        let geometry = unsafe { Geometry::new(device) }?;
        let (_depth, dsv_heap) = unsafe { create_depth(device, width, height) }?;
        Ok(Self {
            root_signature,
            pipeline,
            geometry,
            _depth,
            dsv_heap,
            width,
            height,
        })
    }

    pub unsafe fn record(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        scene: &SceneState,
        render_target: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        let depth_target = unsafe { self.dsv_heap.GetCPUDescriptorHandleForHeapStart() };
        let viewport = D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: self.width as f32,
            Height: self.height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        let scissor = RECT {
            left: 0,
            top: 0,
            right: self.width as i32,
            bottom: self.height as i32,
        };
        let view_projection = scene
            .view_projection(self.width as f32 / self.height as f32)
            .to_cols_array();
        unsafe {
            command_list.SetGraphicsRootSignature(&self.root_signature);
            command_list.SetPipelineState(&self.pipeline);
            command_list.RSSetViewports(&[viewport]);
            command_list.RSSetScissorRects(&[scissor]);
            command_list.OMSetRenderTargets(1, Some(&render_target), true, Some(&depth_target));
            command_list.ClearDepthStencilView(depth_target, D3D12_CLEAR_FLAG_DEPTH, 0.0, 0, None);
            command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            command_list.IASetVertexBuffers(0, Some(&[self.geometry.vertex_view]));
            command_list.IASetIndexBuffer(Some(&self.geometry.index_view));
            command_list.SetGraphicsRoot32BitConstants(
                0,
                FRAME_CONSTANT_COUNT,
                view_projection.as_ptr().cast(),
                0,
            );
        }

        for object in &OBJECTS {
            let mut constants = [0u32; DRAW_CONSTANT_COUNT as usize];
            for (destination, value) in constants[..16]
                .iter_mut()
                .zip(object.model_matrix().to_cols_array())
            {
                *destination = value.to_bits();
            }
            for (destination, value) in constants[16..20].iter_mut().zip(object.color) {
                *destination = value.to_bits();
            }
            constants[20] = object.material;
            let mesh = match object.mesh {
                MeshKind::Plane => self.geometry.plane,
                MeshKind::Cube => self.geometry.cube,
            };
            unsafe {
                command_list.SetGraphicsRoot32BitConstants(
                    1,
                    DRAW_CONSTANT_COUNT,
                    constants.as_ptr().cast(),
                    0,
                );
                command_list.DrawIndexedInstanced(
                    mesh.index_count,
                    1,
                    mesh.start_index,
                    mesh.base_vertex,
                    0,
                );
            }
        }
    }
}

impl Geometry {
    unsafe fn new(device: &ID3D12Device) -> Result<Self> {
        let (vertices, indices) = geometry_data();
        let vertex_bytes = as_bytes(&vertices);
        let index_bytes = as_bytes(&indices);
        let vertex_resource = unsafe { create_upload_buffer(device, vertex_bytes) }?;
        let index_resource = unsafe { create_upload_buffer(device, index_bytes) }?;
        Ok(Self {
            vertex_view: D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: unsafe { vertex_resource.GetGPUVirtualAddress() },
                SizeInBytes: vertex_bytes.len() as u32,
                StrideInBytes: size_of::<Vertex>() as u32,
            },
            index_view: D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: unsafe { index_resource.GetGPUVirtualAddress() },
                SizeInBytes: index_bytes.len() as u32,
                Format: DXGI_FORMAT_R16_UINT,
            },
            _vertices: vertex_resource,
            _indices: index_resource,
            plane: MeshRange {
                index_count: 6,
                start_index: 0,
                base_vertex: 0,
            },
            cube: MeshRange {
                index_count: 36,
                start_index: 6,
                base_vertex: 4,
            },
        })
    }
}

unsafe fn create_root_signature(device: &ID3D12Device) -> Result<ID3D12RootSignature> {
    let parameters = [
        root_constants(0, FRAME_CONSTANT_COUNT),
        root_constants(1, DRAW_CONSTANT_COUNT),
    ];
    let desc = D3D12_ROOT_SIGNATURE_DESC {
        NumParameters: parameters.len() as u32,
        pParameters: parameters.as_ptr(),
        NumStaticSamplers: 0,
        pStaticSamplers: ptr::null(),
        Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
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
    .context("calibration root signature serialization failed")?;
    let blob: ID3DBlob = blob.context("root signature serialization returned no blob")?;
    let bytes = unsafe {
        std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize())
    };
    unsafe { device.CreateRootSignature(0, bytes) }.context("CreateRootSignature failed")
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

unsafe fn create_pipeline(
    device: &ID3D12Device,
    root_signature: &ID3D12RootSignature,
) -> Result<ID3D12PipelineState> {
    let input = [
        input_element(b"POSITION\0", DXGI_FORMAT_R32G32B32_FLOAT, 0),
        input_element(b"NORMAL\0", DXGI_FORMAT_R32G32B32_FLOAT, 12),
    ];
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
    let mut desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
        pRootSignature: ManuallyDrop::new(Some(root_signature.clone())),
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
            FrontCounterClockwise: true.into(),
            DepthClipEnable: true.into(),
            ..Default::default()
        },
        DepthStencilState: D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: true.into(),
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_GREATER,
            StencilEnable: false.into(),
            StencilReadMask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
            StencilWriteMask: D3D12_DEFAULT_STENCIL_WRITE_MASK as u8,
            ..Default::default()
        },
        InputLayout: D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: input.as_ptr(),
            NumElements: input.len() as u32,
        },
        PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        NumRenderTargets: 1,
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
    result.context("CreateGraphicsPipelineState failed")
}

fn input_element(
    semantic: &'static [u8],
    format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT,
    offset: u32,
) -> D3D12_INPUT_ELEMENT_DESC {
    D3D12_INPUT_ELEMENT_DESC {
        SemanticName: PCSTR(semantic.as_ptr()),
        SemanticIndex: 0,
        Format: format,
        InputSlot: 0,
        AlignedByteOffset: offset,
        InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    }
}

fn shader_bytecode(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE {
        pShaderBytecode: bytes.as_ptr().cast(),
        BytecodeLength: bytes.len(),
    }
}

unsafe fn create_depth(
    device: &ID3D12Device,
    width: u32,
    height: u32,
) -> Result<(ID3D12Resource, ID3D12DescriptorHeap)> {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_DEFAULT,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Alignment: 0,
        Width: u64::from(width),
        Height: height,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_D32_FLOAT,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
        Flags: D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
    };
    let clear = D3D12_CLEAR_VALUE {
        Format: DXGI_FORMAT_D32_FLOAT,
        Anonymous: D3D12_CLEAR_VALUE_0 {
            DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                Depth: 0.0,
                Stencil: 0,
            },
        },
    };
    let mut depth = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            D3D12_RESOURCE_STATE_DEPTH_WRITE,
            Some(&clear),
            &mut depth,
        )
    }
    .context("depth allocation failed")?;
    let depth = depth.context("depth allocation returned no resource")?;
    let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
        Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        NumDescriptors: 1,
        Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        NodeMask: 0,
    };
    let dsv_heap: ID3D12DescriptorHeap = unsafe { device.CreateDescriptorHeap(&heap_desc) }
        .context("depth descriptor heap creation failed")?;
    unsafe {
        device.CreateDepthStencilView(&depth, None, dsv_heap.GetCPUDescriptorHandleForHeapStart());
    }
    Ok((depth, dsv_heap))
}

unsafe fn create_upload_buffer(device: &ID3D12Device, bytes: &[u8]) -> Result<ID3D12Resource> {
    let heap = D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_UPLOAD,
        ..Default::default()
    };
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Alignment: 0,
        Width: bytes.len() as u64,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_UNKNOWN,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        Flags: D3D12_RESOURCE_FLAG_NONE,
    };
    let mut resource = None;
    unsafe {
        device.CreateCommittedResource(
            &heap,
            D3D12_HEAP_FLAG_NONE,
            &desc,
            D3D12_RESOURCE_STATE_GENERIC_READ,
            None,
            &mut resource,
        )
    }
    .context("upload buffer allocation failed")?;
    let resource: ID3D12Resource =
        resource.context("upload buffer allocation returned no resource")?;
    let mut mapped: *mut std::ffi::c_void = ptr::null_mut();
    let no_read = D3D12_RANGE { Begin: 0, End: 0 };
    unsafe { resource.Map(0, Some(&no_read), Some(&mut mapped)) }
        .context("upload buffer map failed")?;
    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.cast::<u8>(), bytes.len()) };
    let written = D3D12_RANGE {
        Begin: 0,
        End: bytes.len(),
    };
    unsafe { resource.Unmap(0, Some(&written)) };
    Ok(resource)
}

fn as_bytes<T>(values: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values)) }
}

fn geometry_data() -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = vec![
        vertex([-0.5, 0.0, -0.5], [0.0, 1.0, 0.0]),
        vertex([-0.5, 0.0, 0.5], [0.0, 1.0, 0.0]),
        vertex([0.5, 0.0, 0.5], [0.0, 1.0, 0.0]),
        vertex([0.5, 0.0, -0.5], [0.0, 1.0, 0.0]),
    ];
    for position in [
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ] {
        let normal = glam::Vec3::from_array(position).normalize().to_array();
        vertices.push(vertex(position, normal));
    }
    let indices = vec![
        0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 1, 0, 3, 1, 3, 2, 0, 4, 7, 0, 7, 3, 5, 1, 2, 5, 2, 6,
        3, 7, 6, 3, 6, 2, 0, 1, 5, 0, 5, 4,
    ];
    (vertices, indices)
}

fn vertex(position: [f32; 3], normal: [f32; 3]) -> Vertex {
    Vertex { position, normal }
}
