static const uint REGION_SLOT_CAPACITY = 50;
static const uint INSTANCES_PER_REGION = 1024;
static const uint MAX_REGION_SIDE = 128;
static const uint REGION_OBJECT_ID_BASE = 65536;

struct InstanceRecord
{
    float3 position;
    float height;
    uint region_id;
};

struct VisibleObject
{
    uint physical_index;
    uint archetype;
    uint lod;
    uint stable_key;
};

struct LodDescriptor
{
    uint meshlet_offset;
    uint meshlet_count;
    uint vertex_count;
    uint primitive_count;
};

struct MeshletDescriptor
{
    uint vertex_offset;
    uint vertex_count;
    uint primitive_offset;
    uint primitive_count;
};

cbuffer MeshletSceneConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint4 load_shape;
    uint4 active_slot_groups[7];
};

StructuredBuffer<InstanceRecord> region_instances[REGION_SLOT_CAPACITY] : register(t0);
StructuredBuffer<uint> region_local_ids[REGION_SLOT_CAPACITY] : register(t56);
RWStructuredBuffer<VisibleObject> visible_objects : register(u0);
RWByteAddressBuffer indirect_and_counters : register(u1);
StructuredBuffer<LodDescriptor> catalog_lods : register(t55);

[numthreads(1, 1, 1)]
void reset_main()
{
    indirect_and_counters.Store(0, 0);
    indirect_and_counters.Store(4, 1);
    indirect_and_counters.Store(8, 1);
    [unroll]
    for (uint offset = 12; offset < 64; offset += 4)
    {
        indirect_and_counters.Store(offset, 0);
    }
}

[numthreads(256, 1, 1)]
void cull_main(uint3 group_id : SV_GroupID, uint group_thread : SV_GroupIndex)
{
    if (group_id.x >= load_shape.x)
    {
        return;
    }
    uint local_index = group_id.y * 256 + group_thread;
    if (local_index >= INSTANCES_PER_REGION)
    {
        return;
    }
    uint slot = active_slot_groups[group_id.x / 4][group_id.x % 4];
    InstanceRecord instance = region_instances[NonUniformResourceIndex(slot)][local_index];
    uint local_id = region_local_ids[NonUniformResourceIndex(slot)][local_index];
    float3 center = instance.position + float3(0.0, instance.height * 0.5, 0.0);
    float4 clip = mul(view_projection, float4(center, 1.0));
    bool in_frustum = clip.w > 0.0
        && abs(clip.x) <= clip.w
        && abs(clip.y) <= clip.w
        && clip.z >= 0.0
        && clip.z <= clip.w;
    uint stable_key = instance.region_id * INSTANCES_PER_REGION + local_id;
    uint archetype = stable_key & 7u;
    if (!in_frustum || (load_shape.z & (1u << archetype)) == 0)
    {
        indirect_and_counters.InterlockedAdd(16, 1);
        return;
    }

    uint lod = clip.w < 42.0 ? 0u : (clip.w < 70.0 ? 1u : 2u);
    if (load_shape.w < 3u)
    {
        lod = load_shape.w;
    }
    LodDescriptor descriptor = catalog_lods[archetype * 3u + lod];
    uint visible_index;
    indirect_and_counters.InterlockedAdd(0, 1, visible_index);
    indirect_and_counters.InterlockedAdd(12, 1);
    indirect_and_counters.InterlockedAdd(20 + lod * 4, 1);
    indirect_and_counters.InterlockedAdd(32, descriptor.meshlet_count);
    indirect_and_counters.InterlockedAdd(36, descriptor.vertex_count);
    indirect_and_counters.InterlockedAdd(40, descriptor.primitive_count);
    indirect_and_counters.InterlockedOr(44, 1u << archetype);
    if (visible_index < load_shape.y)
    {
        VisibleObject visible;
        visible.physical_index = slot * INSTANCES_PER_REGION + local_index;
        visible.archetype = archetype;
        visible.lod = lod;
        visible.stable_key = stable_key;
        visible_objects[visible_index] = visible;
    }
}

StructuredBuffer<VisibleObject> draw_objects : register(t50);
StructuredBuffer<float4> catalog_vertices : register(t51);
StructuredBuffer<MeshletDescriptor> catalog_meshlets : register(t52);
StructuredBuffer<uint> catalog_meshlet_vertices : register(t53);
StructuredBuffer<uint> catalog_primitives : register(t54);

struct MeshPayload
{
    uint visible_index;
    uint meshlet_offset;
};

groupshared MeshPayload amplification_payload;

[numthreads(1, 1, 1)]
void as_main(uint3 group_id : SV_GroupID)
{
    VisibleObject visible = draw_objects[group_id.x];
    LodDescriptor descriptor = catalog_lods[visible.archetype * 3u + visible.lod];
    amplification_payload.visible_index = group_id.x;
    amplification_payload.meshlet_offset = descriptor.meshlet_offset;
    DispatchMesh(descriptor.meshlet_count, 1, 1, amplification_payload);
}

struct MeshVertexOutput
{
    float4 position : SV_POSITION;
    nointerpolation float3 color : COLOR0;
    nointerpolation uint object_id : TEXCOORD0;
};

[outputtopology("triangle")]
[numthreads(64, 1, 1)]
void ms_main(
    uint group_thread : SV_GroupIndex,
    uint3 group_id : SV_GroupID,
    in payload MeshPayload payload,
    out vertices MeshVertexOutput output_vertices[64],
    out indices uint3 output_triangles[126])
{
    VisibleObject visible = draw_objects[payload.visible_index];
    MeshletDescriptor meshlet = catalog_meshlets[payload.meshlet_offset + group_id.x];
    SetMeshOutputCounts(meshlet.vertex_count, meshlet.primitive_count);

    uint slot = visible.physical_index / INSTANCES_PER_REGION;
    uint local_index = visible.physical_index % INSTANCES_PER_REGION;
    InstanceRecord instance = region_instances[NonUniformResourceIndex(slot)][local_index];
    float angle = float((visible.stable_key * 747796405u) & 65535u) * 6.28318530718 / 65536.0;
    float sine;
    float cosine;
    sincos(angle, sine, cosine);
    if (group_thread < meshlet.vertex_count)
    {
        uint vertex_index = catalog_meshlet_vertices[meshlet.vertex_offset + group_thread];
        float3 local = catalog_vertices[vertex_index].xyz;
        float3 rotated = float3(
            local.x * cosine - local.z * sine,
            local.y * instance.height,
            local.x * sine + local.z * cosine
        );
        float3 palette[8] = {
            float3(0.91, 0.24, 0.18), float3(0.12, 0.68, 0.36),
            float3(0.16, 0.42, 0.92), float3(0.92, 0.66, 0.12),
            float3(0.68, 0.24, 0.82), float3(0.08, 0.72, 0.76),
            float3(0.88, 0.38, 0.58), float3(0.52, 0.72, 0.14)
        };
        MeshVertexOutput output;
        output.position = mul(view_projection, float4(instance.position + rotated, 1.0));
        output.color = palette[visible.archetype] * (1.0 - 0.12 * visible.lod);
        output.object_id = REGION_OBJECT_ID_BASE + instance.region_id + 1;
        output_vertices[group_thread] = output;
    }
    for (uint primitive_index = group_thread; primitive_index < meshlet.primitive_count; primitive_index += 64)
    {
        uint primitive = catalog_primitives[meshlet.primitive_offset + primitive_index];
        output_triangles[primitive_index] = uint3(
            primitive & 0xffu,
            (primitive >> 8) & 0xffu,
            (primitive >> 16) & 0xffu
        );
    }
}

struct MeshPixelOutput
{
    float4 color : SV_TARGET0;
    uint object_id : SV_TARGET1;
};

MeshPixelOutput ps_main(MeshVertexOutput input)
{
    MeshPixelOutput output;
    output.color = float4(input.color, 1.0);
    output.object_id = input.object_id;
    return output;
}
