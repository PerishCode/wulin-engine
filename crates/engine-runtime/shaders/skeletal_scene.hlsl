static const uint REGION_SLOT_CAPACITY = 50;
static const uint INSTANCES_PER_REGION = 1024;
static const uint MAX_BONES = 128;
static const uint RIG_COUNT = 2;
static const uint CLIPS_PER_RIG = 8;
static const uint POSE_KEYS_PER_RIG = CLIPS_PER_RIG * 64;
static const uint MAX_POSE_KEYS = RIG_COUNT * POSE_KEYS_PER_RIG;
static const uint POSE_WORDS = MAX_POSE_KEYS / 32;
static const uint COUNTER_DWORDS = 20;
static const uint TERRAIN_HEIGHT_OFFSET = 16;
static const uint TERRAIN_SAMPLE_SIDE = 33;
static const uint PRESENTATION_TIME_UNITS_PER_FRAME = 80;

struct InstanceRecord
{
    float3 position;
    float height;
    uint region_id;
};

struct PresentationRecord
{
    uint archetype;
    uint material;
    uint yaw_q16;
    uint animation;
};

struct VisibleObject
{
    float3 position;
    float height;
    uint semantic_region;
    uint archetype;
    uint lod;
    uint stable_identity_low;
    uint stable_identity_high;
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

cbuffer SkeletalSceneConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint4 load_shape;
    uint4 active_slot_groups[7];
    uint4 animation_shape;
    uint4 pose_shape;
    uint4 imported_source_clip_duration_units;
};

StructuredBuffer<InstanceRecord> region_instances[REGION_SLOT_CAPACITY] : register(t0);
StructuredBuffer<LodDescriptor> catalog_lods : register(t55);
StructuredBuffer<BoneMeta> catalog_bones : register(t56);
StructuredBuffer<AffineTransform> catalog_inverse_bind : register(t57);
StructuredBuffer<AffineTransform> catalog_samples : register(t58);
ByteAddressBuffer terrain_tiles[REGION_SLOT_CAPACITY] : register(t63);
StructuredBuffer<uint> region_local_ids[REGION_SLOT_CAPACITY] : register(t114);
StructuredBuffer<PresentationRecord> region_presentations[REGION_SLOT_CAPACITY] : register(t164);
StructuredBuffer<VisibleObject> actor_candidate : register(t214);

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

uint object_stable_key(InstanceRecord instance, uint local_id, uint semantic_region)
{
    return semantic_region == 0u
        ? instance.region_id * INSTANCES_PER_REGION + local_id
        : instance.region_id ^ (local_id * 747796405u);
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

uint presentation_rig(uint archetype)
{
    return archetype == 7u ? 1u : 0u;
}

uint presentation_pose_phase(uint archetype, uint animation)
{
    uint rig = presentation_rig(archetype);
    uint clip = animation & 255u;
    uint duration_units = rig == 0u
        ? animation_shape.z * PRESENTATION_TIME_UNITS_PER_FRAME
        : imported_source_clip_duration_units[clip % 3u];
    uint elapsed_units = (animation_shape.w * PRESENTATION_TIME_UNITS_PER_FRAME)
        % duration_units;
    uint phase = elapsed_units * animation_shape.z / duration_units;
    uint phase_offset = (animation >> 8u) & 255u;
    return (phase_offset + phase) % animation_shape.z;
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
    if (group_id.x > load_shape.x)
    {
        return;
    }
    bool dynamic_actor = group_id.x == load_shape.x;
    if (dynamic_actor && (group_id.y != 0u || group_thread != 0u))
    {
        return;
    }
    uint local_index = group_id.y * 256 + group_thread;
    if (!dynamic_actor && local_index >= INSTANCES_PER_REGION)
    {
        return;
    }

    float3 position;
    float instance_height;
    uint semantic_region;
    uint stable_identity_low;
    uint stable_identity_high;
    uint logical_index;
    PresentationRecord presentation;
    if (dynamic_actor)
    {
        VisibleObject candidate = actor_candidate[0];
        if (candidate.candidate_index == 0xffffffffu)
        {
            return;
        }
        position = candidate.position;
        instance_height = candidate.height;
        semantic_region = candidate.semantic_region;
        stable_identity_low = candidate.stable_identity_low;
        stable_identity_high = candidate.stable_identity_high;
        logical_index = candidate.candidate_index;
        presentation.archetype = candidate.archetype;
        presentation.material = candidate.material;
        presentation.yaw_q16 = candidate.yaw_q16;
        presentation.animation = candidate.animation;
    }
    else
    {
        uint packed_slots = active_slot_groups[group_id.x / 4][group_id.x % 4];
        uint slot = packed_slots & 63u;
        semantic_region = packed_slots >> 12u;
        InstanceRecord instance = region_instances[NonUniformResourceIndex(slot)][local_index];
        uint local_id = region_local_ids[NonUniformResourceIndex(slot)][local_index];
        presentation = region_presentations[NonUniformResourceIndex(slot)][local_index];
        position = object_position(instance, semantic_region);
        instance_height = instance.height;
        stable_identity_low = object_stable_key(instance, local_id, semantic_region);
        stable_identity_high = local_id;
        logical_index = group_id.x * INSTANCES_PER_REGION + local_index;
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
                ground_value = terrain_ground_q16(
                    terrain_slot,
                    instance,
                    semantic_region,
                    position
                );
                ground = float(ground_value) / 65536.0;
            }
            ground_numerators_out[logical_index] = ground_value;
        }
        position.y += ground;
        uint suppression = imported_source_clip_duration_units.w;
        if (
            (suppression & 0x80000000u) != 0u &&
            group_id.x == ((suppression >> 10u) & 31u) &&
            local_id == (suppression & 1023u)
        )
        {
            indirect_and_counters.InterlockedAdd(16, 1);
            return;
        }
    }
    float3 center = position + float3(0.0, instance_height * 0.5, 0.0);
    float4 clip = mul(view_projection, float4(center, 1.0));
    bool in_frustum = clip.w > 0.0
        && abs(clip.x) <= clip.w
        && abs(clip.y) <= clip.w
        && clip.z >= 0.0
        && clip.z <= clip.w;
    uint archetype = presentation.archetype;
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

    bool animated = presentation.animation != 0xffffffffu;
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
            uint clip = presentation.animation & 255u;
            uint rig = presentation_rig(archetype);
            pose_slot = rig * POSE_KEYS_PER_RIG
                + clip * 64u
                + presentation_pose_phase(archetype, presentation.animation);
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
    visible.position = position;
    visible.height = instance_height;
    visible.semantic_region = semantic_region;
    visible.archetype = archetype;
    visible.lod = lod;
    visible.stable_identity_low = stable_identity_low;
    visible.stable_identity_high = stable_identity_high;
    visible.pose_slot = pose_slot;
    visible.candidate_index = logical_index;
    visible.material = presentation.material;
    visible.yaw_q16 = presentation.yaw_q16;
    visible.animation = presentation.animation;
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
    uint rig;
    if (pose_shape.x != 0)
    {
        uint visible_index = animated_visible_indices[group_id.x];
        VisibleObject visible = visible_objects[visible_index];
        pose_slot = group_id.x;
        clip = visible.animation & 255u;
        phase = presentation_pose_phase(visible.archetype, visible.animation);
        variant = visible.animation >> 16u;
        rig = presentation_rig(visible.archetype);
    }
    else
    {
        uint key = active_pose_keys[group_id.x];
        pose_slot = key;
        rig = key / POSE_KEYS_PER_RIG;
        clip = (key % POSE_KEYS_PER_RIG) / 64u;
        phase = key % 64u;
        variant = 0;
    }

    uint bone_count = animation_shape.y;
    BoneMeta bone;
    AffineTransform local;
    if (group_thread < bone_count)
    {
        uint bone_index = rig * MAX_BONES + group_thread;
        bone = catalog_bones[bone_index];
        uint sample_index = ((rig * CLIPS_PER_RIG + clip) * 64u + phase)
            * MAX_BONES + group_thread;
        local = catalog_samples[sample_index];
        if (rig == 0u)
        {
            local = apply_variant(local, variant, group_thread);
        }
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
            catalog_inverse_bind[rig * MAX_BONES + group_thread]
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
        validation_sample.Store(20, rig);
    }
}
