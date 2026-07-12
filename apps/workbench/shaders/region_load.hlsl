static const uint MAX_REGION_SIDE = 128;
static const uint REGION_INSTANCE_SIDE = 32;
static const uint INSTANCES_PER_REGION = 1024;
static const float REGION_METERS = 16.0;
static const uint REGION_OBJECT_ID_BASE = 65536;

cbuffer LoadConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint world_region_side;
    uint active_center_x;
    uint active_center_z;
    uint active_radius;
    uint max_visible_instances;
};

RWStructuredBuffer<uint> visible_instances : register(u0);
RWByteAddressBuffer draw_arguments : register(u1);

float instance_height(uint reference)
{
    uint value = reference * 747796405u + 2891336453u;
    value = ((value >> ((value >> 28u) + 4u)) ^ value) * 277803737u;
    value = (value >> 22u) ^ value;
    return 0.7 + float(value & 1023u) / 1023.0 * 2.3;
}

float3 instance_position(uint region_x, uint region_z, uint local_index)
{
    uint local_x = local_index % REGION_INSTANCE_SIDE;
    uint local_z = local_index / REGION_INSTANCE_SIDE;
    float2 local = (float2(local_x, local_z) + 0.5) / float(REGION_INSTANCE_SIDE) - 0.5;
    float2 region_center = float2(
        int(region_x) - int(MAX_REGION_SIDE / 2),
        int(region_z) - int(MAX_REGION_SIDE / 2)
    ) * REGION_METERS;
    return float3(region_center.x + local.x * REGION_METERS, 0.0, region_center.y + local.y * REGION_METERS);
}

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
    uint diameter = active_radius * 2 + 1;
    int region_x = int(active_center_x) + int(group_id.x % diameter) - int(active_radius);
    int region_z = int(active_center_z) + int(group_id.x / diameter) - int(active_radius);
    uint world_start = (MAX_REGION_SIDE - world_region_side) / 2;
    uint world_end = world_start + world_region_side;
    if (region_x < int(world_start) || region_z < int(world_start)
        || region_x >= int(world_end) || region_z >= int(world_end))
    {
        return;
    }

    uint local_index = group_id.y * 256 + group_thread;
    if (local_index >= INSTANCES_PER_REGION)
    {
        return;
    }
    uint region_id = uint(region_z) * MAX_REGION_SIDE + uint(region_x);
    uint reference = region_id * INSTANCES_PER_REGION + local_index;
    float height = instance_height(reference);
    float3 position = instance_position(uint(region_x), uint(region_z), local_index);
    float4 clip = mul(view_projection, float4(position + float3(0.0, height * 0.5, 0.0), 1.0));
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
        visible_instances[compacted_index] = reference;
    }
}

StructuredBuffer<uint> draw_instances : register(t0);

struct LoadVertexOutput
{
    float4 position : SV_POSITION;
    nointerpolation float3 color : COLOR0;
    nointerpolation uint object_id : TEXCOORD0;
};

LoadVertexOutput vs_main(uint vertex_id : SV_VertexID, uint instance_id : SV_InstanceID)
{
    static const float2 corners[6] = {
        float2(-0.5, 0.0), float2(-0.5, 1.0), float2(0.5, 1.0),
        float2(-0.5, 0.0), float2(0.5, 1.0), float2(0.5, 0.0)
    };
    uint reference = draw_instances[instance_id];
    uint region_id = reference / INSTANCES_PER_REGION;
    uint local_index = reference % INSTANCES_PER_REGION;
    uint region_x = region_id % MAX_REGION_SIDE;
    uint region_z = region_id / MAX_REGION_SIDE;
    float height = instance_height(reference);
    float3 position = instance_position(region_x, region_z, local_index);
    position.x += corners[vertex_id].x * 0.34;
    position.y += corners[vertex_id].y * height;

    int relative_x = int(region_x) - int(MAX_REGION_SIDE / 2);
    int relative_z = int(region_z) - int(MAX_REGION_SIDE / 2);
    uint color_key = uint((relative_x + 8) * 17 + (relative_z + 8) * 31);
    LoadVertexOutput output;
    output.position = mul(view_projection, float4(position, 1.0));
    output.color = 0.25 + 0.7 * float3(
        float((color_key * 37u) & 255u),
        float((color_key * 73u) & 255u),
        float((color_key * 109u) & 255u)
    ) / 255.0;
    output.object_id = REGION_OBJECT_ID_BASE + region_id + 1;
    return output;
}

struct LoadPixelOutput
{
    float4 color : SV_TARGET0;
    uint object_id : SV_TARGET1;
};

LoadPixelOutput ps_main(LoadVertexOutput input)
{
    LoadPixelOutput output;
    output.color = float4(input.color, 1.0);
    output.object_id = input.object_id;
    return output;
}
