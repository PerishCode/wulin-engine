static const uint REGION_SLOT_CAPACITY = 50;
static const uint INSTANCES_PER_REGION = 1024;
static const uint REGION_OBJECT_ID_BASE = 65536;
static const uint MAX_BONES = 128;
static const uint MAX_POSE_KEYS = 512;
static const uint POSE_WORDS = MAX_POSE_KEYS / 32;
static const uint COUNTER_DWORDS = 20;
static const uint TERRAIN_HEIGHT_OFFSET = 16;
static const uint TERRAIN_SAMPLE_SIDE = 33;

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
    uint reserved;
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

struct BoneMeta
{
    uint parent;
    uint depth;
    float3 local_translation;
    float reserved;
};

struct AffineTransform
{
    float4 row0;
    float4 row1;
    float4 row2;
};

struct SkinBinding
{
    uint indices;
    uint weights;
};

cbuffer SkeletalSceneConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint4 load_shape;
    uint4 active_slot_groups[7];
    uint4 animation_shape;
    uint4 pose_shape;
};

StructuredBuffer<InstanceRecord> region_instances[REGION_SLOT_CAPACITY] : register(t0);
StructuredBuffer<float4> catalog_vertices : register(t51);
StructuredBuffer<MeshletDescriptor> catalog_meshlets : register(t52);
StructuredBuffer<uint> catalog_meshlet_vertices : register(t53);
StructuredBuffer<uint> catalog_primitives : register(t54);
StructuredBuffer<LodDescriptor> catalog_lods : register(t55);
StructuredBuffer<BoneMeta> catalog_bones : register(t56);
StructuredBuffer<AffineTransform> catalog_inverse_bind : register(t57);
StructuredBuffer<AffineTransform> catalog_samples : register(t58);
StructuredBuffer<SkinBinding> catalog_skin : register(t59);
StructuredBuffer<VisibleObject> draw_objects : register(t50);
StructuredBuffer<AffineTransform> palette_in : register(t62);
ByteAddressBuffer terrain_tiles[REGION_SLOT_CAPACITY] : register(t63);
StructuredBuffer<int> ground_numerators_in : register(t113);

RWStructuredBuffer<VisibleObject> visible_objects : register(u0);
RWByteAddressBuffer indirect_and_counters : register(u1);
RWStructuredBuffer<uint> animated_visible_indices : register(u2);
RWByteAddressBuffer pose_bitset : register(u3);
RWStructuredBuffer<uint> active_pose_keys : register(u4);
RWStructuredBuffer<AffineTransform> palette_out : register(u5);
RWByteAddressBuffer validation_sample : register(u6);
RWStructuredBuffer<int> ground_numerators_out : register(u7);

int terrain_height(uint slot, uint x, uint z)
{
    uint offset = TERRAIN_HEIGHT_OFFSET + (z * TERRAIN_SAMPLE_SIDE + x) * 2u;
    uint packed = terrain_tiles[NonUniformResourceIndex(slot)].Load(offset & ~3u);
    uint value = (packed >> ((offset & 2u) * 8u)) & 0xffffu;
    return int(value << 16u) >> 16;
}

float3 object_position(InstanceRecord instance, uint semantic_region)
{
    if (semantic_region == 0u)
    {
        return instance.position;
    }
    int region_x = int(semantic_region % 128u);
    int region_z = int(semantic_region / 128u);
    return instance.position + float3(
        float((region_x - 64) * 16),
        0.0,
        float((region_z - 64) * 16)
    );
}

uint object_stable_key(InstanceRecord instance, uint local_index, uint semantic_region)
{
    return semantic_region == 0u
        ? instance.region_id * INSTANCES_PER_REGION + local_index
        : instance.region_id ^ (local_index * 747796405u);
}

int terrain_ground_q16(
    uint slot,
    InstanceRecord instance,
    uint semantic_region,
    float3 position)
{
    uint region_id = semantic_region == 0u ? instance.region_id : semantic_region;
    int region_x = int(region_id % 128u);
    int region_z = int(region_id / 128u);
    float minimum_x = float((region_x - 64) * 16 - 8);
    float minimum_z = float((region_z - 64) * 16 - 8);
    uint x_q9 = uint(clamp(int(round((position.x - minimum_x) * 512.0)), 0, 8192));
    uint z_q9 = uint(clamp(int(round((position.z - minimum_z) * 512.0)), 0, 8192));
    uint cell_x = min(x_q9 >> 8u, 31u);
    uint cell_z = min(z_q9 >> 8u, 31u);
    uint u = x_q9 - cell_x * 256u;
    uint v = z_q9 - cell_z * 256u;
    uint sum = u + v;
    int h00 = terrain_height(slot, cell_x, cell_z);
    int h10 = terrain_height(slot, cell_x + 1u, cell_z);
    int h01 = terrain_height(slot, cell_x, cell_z + 1u);
    int h11 = terrain_height(slot, cell_x + 1u, cell_z + 1u);
    return sum <= 256u
        ? h00 * int(256u - sum) + h10 * int(u) + h01 * int(v)
        : h10 * int(256u - v) + h01 * int(256u - u) + h11 * int(sum - 256u);
}

uint pose_phase(uint stable_key)
{
    uint phase_count = animation_shape.z;
    uint bucket = ((stable_key >> 3) + animation_shape.w) % phase_count;
    return bucket * (64u / phase_count);
}

[numthreads(64, 1, 1)]
void reset_main(uint group_thread : SV_GroupIndex)
{
    if (group_thread < COUNTER_DWORDS)
    {
        indirect_and_counters.Store(group_thread * 4, 0);
    }
    if (group_thread < POSE_WORDS)
    {
        pose_bitset.Store(group_thread * 4, 0);
    }
    if (group_thread < 56)
    {
        validation_sample.Store(group_thread * 4, 0);
    }
    GroupMemoryBarrierWithGroupSync();
    if (group_thread == 0)
    {
        indirect_and_counters.Store(4, 1);
        indirect_and_counters.Store(8, 1);
        indirect_and_counters.Store(60, 1);
        indirect_and_counters.Store(64, 1);
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
    uint packed_slots = active_slot_groups[group_id.x / 4][group_id.x % 4];
    uint slot = packed_slots & 63u;
    uint semantic_region = packed_slots >> 12u;
    InstanceRecord instance = region_instances[NonUniformResourceIndex(slot)][local_index];
    float3 position = object_position(instance, semantic_region);
    uint logical_index = group_id.x * INSTANCES_PER_REGION + local_index;
    float ground = 0.0;
    if (pose_shape.w != 0)
    {
        uint terrain_slot = (packed_slots >> 6u) & 63u;
        int ground_value;
        if (pose_shape.w == 1u)
        {
            uint cell_x = local_index % 32u;
            uint cell_z = local_index / 32u;
            ground_value = terrain_height(terrain_slot, cell_x + 1u, cell_z)
                + terrain_height(terrain_slot, cell_x, cell_z + 1u);
            ground = float(ground_value) / 512.0;
        }
        else
        {
            ground_value = terrain_ground_q16(terrain_slot, instance, semantic_region, position);
            ground = float(ground_value) / 65536.0;
        }
        ground_numerators_out[logical_index] = ground_value;
    }
    float3 center = position + float3(0.0, ground + instance.height * 0.5, 0.0);
    float4 clip = mul(view_projection, float4(center, 1.0));
    bool in_frustum = clip.w > 0.0
        && abs(clip.x) <= clip.w
        && abs(clip.y) <= clip.w
        && clip.z >= 0.0
        && clip.z <= clip.w;
    uint stable_key = object_stable_key(instance, local_index, semantic_region);
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
    indirect_and_counters.InterlockedAdd(28 + lod * 4, 1);
    indirect_and_counters.InterlockedAdd(40, descriptor.meshlet_count);
    indirect_and_counters.InterlockedAdd(44, descriptor.vertex_count);
    indirect_and_counters.InterlockedAdd(48, descriptor.primitive_count);
    indirect_and_counters.InterlockedOr(52, 1u << archetype);

    bool animated = stable_key % 100u < animation_shape.x;
    uint pose_slot = 0xffffffffu;
    if (animated)
    {
        uint animated_index;
        indirect_and_counters.InterlockedAdd(20, 1, animated_index);
        indirect_and_counters.InterlockedAdd(76, descriptor.vertex_count * 4u);
        animated_visible_indices[animated_index] = visible_index;
        if (pose_shape.x != 0)
        {
            pose_slot = animated_index;
            indirect_and_counters.InterlockedAdd(56, 1);
            indirect_and_counters.InterlockedAdd(68, 1);
        }
        else
        {
            pose_slot = archetype * 64u + pose_phase(stable_key);
            uint ignored;
            pose_bitset.InterlockedOr((pose_slot / 32u) * 4u, 1u << (pose_slot % 32u), ignored);
        }
    }
    else
    {
        indirect_and_counters.InterlockedAdd(24, 1);
    }

    if (visible_index >= load_shape.y)
    {
        indirect_and_counters.InterlockedAdd(72, 1);
        return;
    }
    VisibleObject visible;
    visible.physical_index = slot * INSTANCES_PER_REGION + local_index;
    visible.archetype = archetype;
    visible.lod = lod;
    visible.stable_key = stable_key;
    visible.pose_slot = pose_slot;
    visible.reserved = logical_index;
    visible_objects[visible_index] = visible;
}

[numthreads(256, 1, 1)]
void compact_main(uint group_thread : SV_GroupIndex)
{
    if (pose_shape.x != 0)
    {
        return;
    }
    for (uint key = group_thread; key < MAX_POSE_KEYS; key += 256)
    {
        uint word = pose_bitset.Load((key / 32u) * 4u);
        if ((word & (1u << (key % 32u))) != 0)
        {
            uint active_index;
            indirect_and_counters.InterlockedAdd(68, 1, active_index);
            indirect_and_counters.InterlockedAdd(56, 1);
            active_pose_keys[active_index] = key;
        }
    }
}

AffineTransform compose_affine(AffineTransform parent, AffineTransform child)
{
    AffineTransform result;
    result.row0 = float4(
        dot(parent.row0.xyz, float3(child.row0.x, child.row1.x, child.row2.x)),
        dot(parent.row0.xyz, float3(child.row0.y, child.row1.y, child.row2.y)),
        dot(parent.row0.xyz, float3(child.row0.z, child.row1.z, child.row2.z)),
        dot(parent.row0.xyz, float3(child.row0.w, child.row1.w, child.row2.w)) + parent.row0.w
    );
    result.row1 = float4(
        dot(parent.row1.xyz, float3(child.row0.x, child.row1.x, child.row2.x)),
        dot(parent.row1.xyz, float3(child.row0.y, child.row1.y, child.row2.y)),
        dot(parent.row1.xyz, float3(child.row0.z, child.row1.z, child.row2.z)),
        dot(parent.row1.xyz, float3(child.row0.w, child.row1.w, child.row2.w)) + parent.row1.w
    );
    result.row2 = float4(
        dot(parent.row2.xyz, float3(child.row0.x, child.row1.x, child.row2.x)),
        dot(parent.row2.xyz, float3(child.row0.y, child.row1.y, child.row2.y)),
        dot(parent.row2.xyz, float3(child.row0.z, child.row1.z, child.row2.z)),
        dot(parent.row2.xyz, float3(child.row0.w, child.row1.w, child.row2.w)) + parent.row2.w
    );
    return result;
}

float centered_byte(uint value)
{
    return float(value & 255u) / 255.0 - 0.5;
}

AffineTransform apply_variant(AffineTransform local, uint seed, uint bone)
{
    if (seed == 0)
    {
        return local;
    }
    uint first = seed * 747796405u + bone * 2891336453u;
    uint rotated = (first << 13u) | (first >> 19u);
    uint second = rotated * 2246822519u;
    local.row0.w += centered_byte(first) * 0.012;
    local.row2.w += centered_byte(second) * 0.012;
    return local;
}

groupshared AffineTransform pose_globals[MAX_BONES];

[numthreads(128, 1, 1)]
void pose_main(uint3 group_id : SV_GroupID, uint group_thread : SV_GroupIndex)
{
    uint pose_slot;
    uint clip;
    uint phase;
    uint variant;
    if (pose_shape.x != 0)
    {
        uint visible_index = animated_visible_indices[group_id.x];
        VisibleObject visible = visible_objects[visible_index];
        pose_slot = group_id.x;
        clip = visible.archetype;
        phase = pose_phase(visible.stable_key);
        variant = visible.stable_key;
    }
    else
    {
        uint key = active_pose_keys[group_id.x];
        pose_slot = key;
        clip = key / 64u;
        phase = key % 64u;
        variant = 0;
    }

    uint bone_count = animation_shape.y;
    BoneMeta bone;
    AffineTransform local;
    if (group_thread < bone_count)
    {
        bone = catalog_bones[group_thread];
        uint sample_index = (clip * 64u + phase) * MAX_BONES + group_thread;
        local = apply_variant(catalog_samples[sample_index], variant, group_thread);
    }
    [unroll]
    for (uint depth = 0; depth < 8; depth++)
    {
        if (group_thread < bone_count && bone.depth == depth)
        {
            if (bone.parent == 0xffffffffu)
            {
                pose_globals[group_thread] = local;
            }
            else
            {
                pose_globals[group_thread] = compose_affine(pose_globals[bone.parent], local);
            }
        }
        GroupMemoryBarrierWithGroupSync();
    }
    if (group_thread < bone_count)
    {
        AffineTransform skin = compose_affine(
            pose_globals[group_thread],
            catalog_inverse_bind[group_thread]
        );
        palette_out[pose_slot * MAX_BONES + group_thread] = skin;
        if (group_id.x == 0 && group_thread < 4)
        {
            uint base = 32u + group_thread * 48u;
            validation_sample.Store4(base, asuint(skin.row0));
            validation_sample.Store4(base + 16u, asuint(skin.row1));
            validation_sample.Store4(base + 32u, asuint(skin.row2));
        }
    }
    if (group_id.x == 0 && group_thread == 0)
    {
        validation_sample.Store(0, clip);
        validation_sample.Store(4, phase);
        validation_sample.Store(8, bone_count);
        validation_sample.Store(12, variant);
        validation_sample.Store(16, pose_slot);
    }
}

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

float3 transform_point(AffineTransform transform, float3 local_position)
{
    float4 homogeneous = float4(local_position, 1.0);
    return float3(
        dot(transform.row0, homogeneous),
        dot(transform.row1, homogeneous),
        dot(transform.row2, homogeneous)
    );
}

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
    uint active_index = visible.reserved / INSTANCES_PER_REGION;
    uint packed_slots = active_slot_groups[active_index / 4][active_index % 4];
    uint semantic_region = packed_slots >> 12u;
    float3 position = object_position(instance, semantic_region);
    float ground = pose_shape.w == 0u
        ? 0.0
        : float(ground_numerators_in[visible.reserved])
            / (pose_shape.w == 1u ? 512.0 : 65536.0);
    float angle = float((visible.stable_key * 747796405u) & 65535u) * 6.28318530718 / 65536.0;
    float sine;
    float cosine;
    sincos(angle, sine, cosine);
    if (group_thread < meshlet.vertex_count)
    {
        uint vertex_index = catalog_meshlet_vertices[meshlet.vertex_offset + group_thread];
        float3 local = catalog_vertices[vertex_index].xyz;
        local.y *= instance.height;
        if (visible.pose_slot != 0xffffffffu)
        {
            SkinBinding binding = catalog_skin[vertex_index];
            float3 skinned = 0.0;
            [unroll]
            for (uint influence = 0; influence < 4; influence++)
            {
                uint bone = ((binding.indices >> (influence * 8u)) & 255u) % animation_shape.y;
                float weight = float((binding.weights >> (influence * 8u)) & 255u) / 255.0;
                skinned += transform_point(
                    palette_in[visible.pose_slot * MAX_BONES + bone],
                    local
                ) * weight;
            }
            local = skinned;
        }
        float3 rotated = float3(
            local.x * cosine - local.z * sine,
            local.y,
            local.x * sine + local.z * cosine
        );
        float3 colors[8] = {
            float3(0.91, 0.24, 0.18), float3(0.12, 0.68, 0.36),
            float3(0.16, 0.42, 0.92), float3(0.92, 0.66, 0.12),
            float3(0.68, 0.24, 0.82), float3(0.08, 0.72, 0.76),
            float3(0.88, 0.38, 0.58), float3(0.52, 0.72, 0.14)
        };
        MeshVertexOutput output;
        output.position = mul(
            view_projection,
            float4(position + float3(0.0, ground, 0.0) + rotated, 1.0)
        );
        output.color = colors[visible.archetype] * (1.0 - 0.12 * visible.lod);
        uint object_region = semantic_region == 0u ? instance.region_id : semantic_region;
        output.object_id = REGION_OBJECT_ID_BASE + object_region + 1;
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
