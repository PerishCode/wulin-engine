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

cbuffer AsyncResidentConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint4 load_shape;
    uint4 active_slot_groups[7];
};

StructuredBuffer<InstanceRecord> region_instances[REGION_SLOT_CAPACITY] : register(t0);
RWStructuredBuffer<uint> visible_instances : register(u0);
RWByteAddressBuffer draw_arguments : register(u1);

[numthreads(1, 1, 1)]
void reset_main()
{
    draw_arguments.Store(0, 6);
    draw_arguments.Store(4, 0);
    draw_arguments.Store(8, 0);
    draw_arguments.Store(12, 0);
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
    float4 clip = mul(
        view_projection,
        float4(instance.position + float3(0.0, instance.height * 0.5, 0.0), 1.0)
    );
    bool visible = clip.w > 0.0
        && abs(clip.x) <= clip.w
        && abs(clip.y) <= clip.w
        && clip.z >= 0.0
        && clip.z <= clip.w;
    if (!visible)
    {
        return;
    }
    uint compacted_index;
    draw_arguments.InterlockedAdd(4, 1, compacted_index);
    if (compacted_index < load_shape.y)
    {
        visible_instances[compacted_index] = slot * INSTANCES_PER_REGION + local_index;
    }
}

StructuredBuffer<uint> draw_instances : register(t50);

struct AsyncResidentVertexOutput
{
    float4 position : SV_POSITION;
    nointerpolation float3 color : COLOR0;
    nointerpolation uint object_id : TEXCOORD0;
};

AsyncResidentVertexOutput vs_main(uint vertex_id : SV_VertexID, uint instance_id : SV_InstanceID)
{
    static const float2 corners[6] = {
        float2(-0.5, 0.0), float2(-0.5, 1.0), float2(0.5, 1.0),
        float2(-0.5, 0.0), float2(0.5, 1.0), float2(0.5, 0.0)
    };
    uint physical_index = draw_instances[instance_id];
    uint slot = physical_index / INSTANCES_PER_REGION;
    uint local_index = physical_index % INSTANCES_PER_REGION;
    InstanceRecord instance = region_instances[NonUniformResourceIndex(slot)][local_index];
    float3 position = instance.position;
    position.x += corners[vertex_id].x * 0.34;
    position.y += corners[vertex_id].y * instance.height;
    uint region_x = instance.region_id % MAX_REGION_SIDE;
    uint region_z = instance.region_id / MAX_REGION_SIDE;
    int relative_x = int(region_x) - int(MAX_REGION_SIDE / 2);
    int relative_z = int(region_z) - int(MAX_REGION_SIDE / 2);
    uint color_key = uint((relative_x + 8) * 17 + (relative_z + 8) * 31);

    AsyncResidentVertexOutput output;
    output.position = mul(view_projection, float4(position, 1.0));
    output.color = 0.25 + 0.7 * float3(
        float((color_key * 37u) & 255u),
        float((color_key * 73u) & 255u),
        float((color_key * 109u) & 255u)
    ) / 255.0;
    output.object_id = REGION_OBJECT_ID_BASE + instance.region_id + 1;
    return output;
}

struct AsyncResidentPixelOutput
{
    float4 color : SV_TARGET0;
    uint object_id : SV_TARGET1;
};

AsyncResidentPixelOutput ps_main(AsyncResidentVertexOutput input)
{
    AsyncResidentPixelOutput output;
    output.color = float4(input.color, 1.0);
    output.object_id = input.object_id;
    return output;
}
