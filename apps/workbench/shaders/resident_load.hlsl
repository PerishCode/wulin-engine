static const uint INSTANCES_PER_REGION = 1024;
static const uint MAX_REGION_SIDE = 128;
static const uint REGION_OBJECT_ID_BASE = 65536;

struct InstanceRecord
{
    float3 position;
    float height;
    uint region_id;
};

struct ActiveRegion
{
    uint slot;
    uint region_id;
};

cbuffer ResidentConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint active_region_count;
    uint max_visible_instances;
};

StructuredBuffer<InstanceRecord> instances : register(t0);
StructuredBuffer<ActiveRegion> active_regions : register(t1);
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
    if (group_id.x >= active_region_count)
    {
        return;
    }
    uint local_index = group_id.y * 256 + group_thread;
    if (local_index >= INSTANCES_PER_REGION)
    {
        return;
    }
    ActiveRegion active = active_regions[group_id.x];
    uint physical_index = active.slot * INSTANCES_PER_REGION + local_index;
    InstanceRecord instance = instances[physical_index];
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
    if (compacted_index < max_visible_instances)
    {
        visible_instances[compacted_index] = physical_index;
    }
}

StructuredBuffer<uint> draw_instances : register(t2);

struct ResidentVertexOutput
{
    float4 position : SV_POSITION;
    nointerpolation float3 color : COLOR0;
    nointerpolation uint object_id : TEXCOORD0;
};

ResidentVertexOutput vs_main(uint vertex_id : SV_VertexID, uint instance_id : SV_InstanceID)
{
    static const float2 corners[6] = {
        float2(-0.5, 0.0), float2(-0.5, 1.0), float2(0.5, 1.0),
        float2(-0.5, 0.0), float2(0.5, 1.0), float2(0.5, 0.0)
    };
    InstanceRecord instance = instances[draw_instances[instance_id]];
    float3 position = instance.position;
    position.x += corners[vertex_id].x * 0.34;
    position.y += corners[vertex_id].y * instance.height;
    uint region_x = instance.region_id % MAX_REGION_SIDE;
    uint region_z = instance.region_id / MAX_REGION_SIDE;
    int relative_x = int(region_x) - int(MAX_REGION_SIDE / 2);
    int relative_z = int(region_z) - int(MAX_REGION_SIDE / 2);
    uint color_key = uint((relative_x + 8) * 17 + (relative_z + 8) * 31);

    ResidentVertexOutput output;
    output.position = mul(view_projection, float4(position, 1.0));
    output.color = 0.25 + 0.7 * float3(
        float((color_key * 37u) & 255u),
        float((color_key * 73u) & 255u),
        float((color_key * 109u) & 255u)
    ) / 255.0;
    output.object_id = REGION_OBJECT_ID_BASE + instance.region_id + 1;
    return output;
}

struct ResidentPixelOutput
{
    float4 color : SV_TARGET0;
    uint object_id : SV_TARGET1;
};

ResidentPixelOutput ps_main(ResidentVertexOutput input)
{
    ResidentPixelOutput output;
    output.color = float4(input.color, 1.0);
    output.object_id = input.object_id;
    return output;
}
