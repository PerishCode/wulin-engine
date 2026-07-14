static const uint REGION_SLOT_CAPACITY = 50;
static const uint INSTANCES_PER_REGION = 1024;
static const uint CANDIDATE_CAPACITY = 25600;
static const uint HIERARCHY_MIPS = 11;

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
    uint pose_slot;
    uint candidate_index;
    uint material;
    uint yaw_q16;
    uint animation;
};

struct LodDescriptor
{
    uint meshlet_offset;
    uint meshlet_count;
    uint vertex_count;
    uint primitive_count;
};

cbuffer SurfaceResolveConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint4 surface_shape;
    float4 background_color;
    uint4 surface_animation;
    uint4 occlusion_shape;
    uint4 hierarchy_shape;
    float4 occlusion_params;
    float4 occlusion_bias;
};

StructuredBuffer<InstanceRecord> region_instances[REGION_SLOT_CAPACITY] : register(t0);
StructuredBuffer<LodDescriptor> catalog_lods : register(t55);
StructuredBuffer<VisibleObject> source_visible : register(t60);
ByteAddressBuffer source_counters : register(t61);
Texture2D<uint2> winner_source : register(t69);
Texture2D<uint> hierarchy_source : register(t70);
StructuredBuffer<int> ground_numerators : register(t113);

RWStructuredBuffer<VisibleObject> filtered_visible : register(u12);
RWByteAddressBuffer occlusion_counters : register(u13);
RWTexture2D<uint> hierarchy_mips[HIERARCHY_MIPS] : register(u14);
RWStructuredBuffer<uint> candidate_mask : register(u25);
RWStructuredBuffer<uint> group_offsets : register(u26);

groupshared uint group_survivors;
groupshared uint compaction_scan[256];

uint active_side()
{
    return uint(round(sqrt(float(surface_animation.y))));
}

float3 canonical_position(InstanceRecord instance, uint candidate_index)
{
    uint side = active_side();
    uint active_index = candidate_index / INSTANCES_PER_REGION;
    int2 offset = int2(
        int(active_index % side) - int(side / 2u),
        int(active_index / side) - int(side / 2u)
    );
    float ground = float(ground_numerators[candidate_index]) / float(surface_animation.z);
    return instance.position + float3(float(offset.x * 16), ground, float(offset.y * 16));
}

bool query_occluded(VisibleObject visible)
{
    uint slot = visible.physical_index / INSTANCES_PER_REGION;
    uint local_index = visible.physical_index % INSTANCES_PER_REGION;
    InstanceRecord instance = region_instances[NonUniformResourceIndex(slot)][local_index];
    float half_xz = occlusion_params.x * instance.height + occlusion_params.y;
    float half_y = instance.height * 0.5 + occlusion_params.z;
    float3 center = canonical_position(instance, visible.candidate_index)
        + float3(0.0, instance.height * 0.5, 0.0);
    float2 minimum_pixel = float2(surface_shape.z, surface_shape.w);
    float2 maximum_pixel = 0.0;
    float nearest_depth = 0.0;

    [unroll]
    for (uint corner = 0; corner < 8; corner++)
    {
        float3 sign_value = float3(
            (corner & 1u) == 0 ? -1.0 : 1.0,
            (corner & 2u) == 0 ? -1.0 : 1.0,
            (corner & 4u) == 0 ? -1.0 : 1.0
        );
        float3 extent = float3(half_xz, half_y, half_xz);
        float4 clip = mul(view_projection, float4(center + sign_value * extent, 1.0));
        if (clip.w <= 0.0)
        {
            return false;
        }
        float3 ndc = clip.xyz / clip.w;
        if (ndc.z < 0.0 || ndc.z > 1.0)
        {
            return false;
        }
        float2 pixel = float2(
            (ndc.x * 0.5 + 0.5) * float(surface_shape.z),
            (-ndc.y * 0.5 + 0.5) * float(surface_shape.w)
        );
        minimum_pixel = min(minimum_pixel, pixel);
        maximum_pixel = max(maximum_pixel, pixel);
        nearest_depth = max(nearest_depth, ndc.z);
    }

    int expansion = int(occlusion_params.w);
    int2 minimum = int2(floor(minimum_pixel)) - expansion;
    int2 maximum = int2(ceil(maximum_pixel)) + expansion;
    int2 last_pixel = int2(surface_shape.z - 1u, surface_shape.w - 1u);
    minimum = clamp(minimum, int2(0, 0), last_pixel);
    maximum = clamp(maximum, int2(0, 0), last_pixel);
    if (any(maximum < minimum))
    {
        return false;
    }

    uint2 extent = uint2(maximum - minimum + 1);
    uint largest = max(extent.x, extent.y);
    uint mip = largest <= 1u ? 0u : firstbithigh(largest - 1u) + 1u;
    mip = min(mip, occlusion_shape.y - 1u);
    uint2 mip_last = uint2(
        max(surface_shape.z >> mip, 1u) - 1u,
        max(surface_shape.w >> mip, 1u) - 1u
    );
    uint2 low = min(uint2(minimum) >> mip, mip_last);
    uint2 high = min(uint2(maximum) >> mip, mip_last);
    uint minimum_depth_bits = min(
        min(
            hierarchy_source.Load(int3(low, mip)),
            hierarchy_source.Load(int3(uint2(high.x, low.y), mip))
        ),
        min(
            hierarchy_source.Load(int3(uint2(low.x, high.y), mip)),
            hierarchy_source.Load(int3(high, mip))
        )
    );
    if (minimum_depth_bits == 0u)
    {
        return false;
    }
    return nearest_depth + occlusion_bias.x < asfloat(minimum_depth_bits);
}

[numthreads(256, 1, 1)]
void occlusion_classify_main(
    uint3 dispatch_thread : SV_DispatchThreadID,
    uint3 group_thread : SV_GroupThreadID,
    uint3 group_id : SV_GroupID
)
{
    uint source_count = source_counters.Load(0);
    uint source_index = dispatch_thread.x;
    if (group_thread.x == 0u)
    {
        group_survivors = 0u;
    }
    if (source_index == 0u)
    {
        occlusion_counters.Store(4, 1u);
        occlusion_counters.Store(8, 1u);
        occlusion_counters.Store(12, source_count);
        occlusion_counters.Store(72, occlusion_shape.x);
    }
    GroupMemoryBarrierWithGroupSync();

    if (source_index < source_count)
    {
        VisibleObject visible = source_visible[source_index];
        LodDescriptor lod = catalog_lods[visible.archetype * 3u + visible.lod];
        occlusion_counters.InterlockedAdd(40, lod.meshlet_count);
        occlusion_counters.InterlockedAdd(48, lod.vertex_count);
        occlusion_counters.InterlockedAdd(56, lod.primitive_count);
        if (visible.pose_slot != 0xffffffffu)
        {
            occlusion_counters.InterlockedAdd(64, lod.vertex_count * 4u);
        }

        bool queried = occlusion_shape.x != 0u;
        bool occluded = queried && query_occluded(visible);
        if (queried)
        {
            occlusion_counters.InterlockedAdd(24, 1u);
        }
        else
        {
            occlusion_counters.InterlockedAdd(28, 1u);
        }
        if (occluded)
        {
            occlusion_counters.InterlockedAdd(20, 1u);
            candidate_mask[visible.candidate_index] = 2u;
        }
        else
        {
            InterlockedAdd(group_survivors, 1u);
            occlusion_counters.InterlockedAdd(16, 1u);
            candidate_mask[visible.candidate_index] = 1u;
            occlusion_counters.InterlockedAdd(44, lod.meshlet_count);
            occlusion_counters.InterlockedAdd(52, lod.vertex_count);
            occlusion_counters.InterlockedAdd(60, lod.primitive_count);
            if (visible.pose_slot != 0xffffffffu)
            {
                occlusion_counters.InterlockedAdd(68, lod.vertex_count * 4u);
            }
        }
    }

    GroupMemoryBarrierWithGroupSync();
    if (group_thread.x == 0u)
    {
        group_offsets[group_id.x] = group_survivors;
    }
}

[numthreads(128, 1, 1)]
void occlusion_prefix_main(uint3 group_thread : SV_GroupThreadID)
{
    uint thread_index = group_thread.x;
    uint count = thread_index < 100u ? group_offsets[thread_index] : 0u;
    compaction_scan[thread_index] = count;
    GroupMemoryBarrierWithGroupSync();
    for (uint offset = 1u; offset < 128u; offset <<= 1u)
    {
        uint value = compaction_scan[thread_index];
        if (thread_index >= offset)
        {
            value += compaction_scan[thread_index - offset];
        }
        GroupMemoryBarrierWithGroupSync();
        compaction_scan[thread_index] = value;
        GroupMemoryBarrierWithGroupSync();
    }
    if (thread_index < 100u)
    {
        group_offsets[thread_index] = compaction_scan[thread_index] - count;
    }
    if (thread_index == 99u)
    {
        occlusion_counters.Store(0, compaction_scan[thread_index]);
    }
}

[numthreads(256, 1, 1)]
void occlusion_scatter_main(
    uint3 dispatch_thread : SV_DispatchThreadID,
    uint3 group_thread : SV_GroupThreadID,
    uint3 group_id : SV_GroupID
)
{
    uint source_count = source_counters.Load(0);
    uint source_index = dispatch_thread.x;
    VisibleObject visible;
    uint keep = 0u;
    if (source_index < source_count)
    {
        visible = source_visible[source_index];
        keep = candidate_mask[visible.candidate_index] == 1u ? 1u : 0u;
    }
    compaction_scan[group_thread.x] = keep;
    GroupMemoryBarrierWithGroupSync();
    for (uint offset = 1u; offset < 256u; offset <<= 1u)
    {
        uint value = compaction_scan[group_thread.x];
        if (group_thread.x >= offset)
        {
            value += compaction_scan[group_thread.x - offset];
        }
        GroupMemoryBarrierWithGroupSync();
        compaction_scan[group_thread.x] = value;
        GroupMemoryBarrierWithGroupSync();
    }
    if (keep != 0u)
    {
        uint output_index = group_offsets[group_id.x]
            + compaction_scan[group_thread.x] - 1u;
        if (output_index < CANDIDATE_CAPACITY)
        {
            filtered_visible[output_index] = visible;
        }
        else
        {
            occlusion_counters.InterlockedAdd(36, 1u);
        }
    }
}

[numthreads(8, 8, 1)]
void hiz_mip0_main(uint3 dispatch_thread : SV_DispatchThreadID)
{
    if (dispatch_thread.x >= surface_shape.z || dispatch_thread.y >= surface_shape.w)
    {
        return;
    }
    hierarchy_mips[0][dispatch_thread.xy] = winner_source[dispatch_thread.xy].x;
}

[numthreads(8, 8, 1)]
void hiz_reduce_main(uint3 dispatch_thread : SV_DispatchThreadID)
{
    uint source_mip = hierarchy_shape.x;
    uint destination_mip = hierarchy_shape.y;
    uint2 destination_size = hierarchy_shape.zw;
    if (dispatch_thread.x >= destination_size.x || dispatch_thread.y >= destination_size.y)
    {
        return;
    }
    uint2 source_size = uint2(
        max(surface_shape.z >> source_mip, 1u),
        max(surface_shape.w >> source_mip, 1u)
    );
    uint2 source = dispatch_thread.xy * 2u;
    uint value = 0xffffffffu;
    [unroll]
    for (uint y = 0; y < 2; y++)
    {
        [unroll]
        for (uint x = 0; x < 2; x++)
        {
            uint2 coordinate = source + uint2(x, y);
            if (all(coordinate < source_size))
            {
                value = min(value, hierarchy_mips[source_mip][coordinate]);
            }
        }
    }
    hierarchy_mips[destination_mip][dispatch_thread.xy] = value;
}
