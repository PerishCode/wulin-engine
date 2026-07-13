static const uint REGION_SLOT_CAPACITY = 50;
static const uint ACTIVE_REGION_CAPACITY = 25;
static const uint PATCHES_PER_REGION = 16;
static const uint PATCH_GROUP_COUNT = ACTIVE_REGION_CAPACITY * PATCHES_PER_REGION;
static const uint CELL_SIDE = 32;
static const uint SAMPLE_SIDE = 33;
static const uint HEIGHT_OFFSET = 16;
static const uint MATERIAL_OFFSET = 2196;
static const uint WORLD_REGION_SIDE = 128;
static const uint TERRAIN_OBJECT_ID_BASE = 32768;

cbuffer TerrainConstants : register(b0)
{
    column_major float4x4 view_projection;
    uint4 terrain_shape;
    uint4 active_mapping[7];
    int2 lod_camera_patch;
    uint2 lod_radii;
    uint4 lod_mode;
};

ByteAddressBuffer terrain_tiles[REGION_SLOT_CAPACITY] : register(t0);
RWByteAddressBuffer terrain_stats : register(u0);
RWByteAddressBuffer seam_stats : register(u1);
RWByteAddressBuffer lod_stats : register(u2);

uint mapping_at(uint index)
{
    return active_mapping[index / 4][index % 4];
}

uint mapping_slot(uint mapping)
{
    return mapping & 63u;
}

uint mapping_region(uint mapping)
{
    return mapping >> 6u;
}

int load_height(uint slot, uint x, uint z)
{
    uint offset = HEIGHT_OFFSET + (z * SAMPLE_SIDE + x) * 2u;
    uint packed = terrain_tiles[NonUniformResourceIndex(slot)].Load(offset & ~3u);
    uint value = (packed >> ((offset & 2u) * 8u)) & 0xffffu;
    return int(value << 16u) >> 16;
}

uint load_material(uint slot, uint x, uint z)
{
    uint offset = MATERIAL_OFFSET + z * CELL_SIDE + x;
    uint packed = terrain_tiles[NonUniformResourceIndex(slot)].Load(offset & ~3u);
    return (packed >> ((offset & 3u) * 8u)) & 255u;
}

float3 terrain_position(uint region_id, uint x, uint z, int height)
{
    uint region_x = region_id % WORLD_REGION_SIDE;
    uint region_z = region_id / WORLD_REGION_SIDE;
    return float3(
        (int(region_x) - 64) * 16.0 - 8.0 + float(x) * 0.5,
        float(height) / 256.0,
        (int(region_z) - 64) * 16.0 - 8.0 + float(z) * 0.5
    );
}

uint select_patch_lod(int patch_x, int patch_z)
{
    if (lod_mode.x == 0)
    {
        return 0;
    }
    if (lod_mode.y != 0)
    {
        return lod_mode.y - 1u;
    }
    uint distance = max(abs(patch_x - lod_camera_patch.x), abs(patch_z - lod_camera_patch.y));
    return distance <= lod_radii.x ? 0u : (distance <= lod_radii.y ? 1u : 2u);
}

void patch_address(
    uint group_index,
    out uint active_index,
    out uint patch_index,
    out uint grid_x,
    out uint grid_z,
    out int global_x,
    out int global_z)
{
    active_index = group_index / PATCHES_PER_REGION;
    patch_index = group_index % PATCHES_PER_REGION;
    uint region_side = terrain_shape.y;
    uint region_x = active_index % region_side;
    uint region_z = active_index / region_side;
    uint local_x = patch_index % 4u;
    uint local_z = patch_index / 4u;
    grid_x = region_x * 4u + local_x;
    grid_z = region_z * 4u + local_z;
    uint region_id = mapping_region(mapping_at(active_index));
    global_x = int(region_id % WORLD_REGION_SIDE) * 4 + int(local_x);
    global_z = int(region_id / WORLD_REGION_SIDE) * 4 + int(local_z);
}

void grid_patch(uint grid_x, uint grid_z, out uint mapping, out uint patch_index)
{
    uint region_side = terrain_shape.y;
    uint active_index = (grid_z / 4u) * region_side + grid_x / 4u;
    mapping = mapping_at(active_index);
    patch_index = (grid_z % 4u) * 4u + grid_x % 4u;
}

[numthreads(1, 1, 1)]
void reset_main()
{
    terrain_stats.Store(0, PATCH_GROUP_COUNT);
    terrain_stats.Store(4, 0);
    terrain_stats.Store(8, 0);
    terrain_stats.Store(12, 0);
    terrain_stats.Store(16, 0);
    terrain_stats.Store(20, terrain_shape.x);
    terrain_stats.Store(24, 0x7fffffffu);
    terrain_stats.Store(28, 0x80000000u);
    seam_stats.Store(0, 0);
    seam_stats.Store(4, 0);
    seam_stats.Store(8, 0);
    seam_stats.Store(12, 0xffffffffu);
    [unroll]
    for (uint index = 0; index < 16; ++index)
    {
        lod_stats.Store(index * 4u, index == 10u ? 0xffffffffu : 0u);
    }
}

[numthreads(64, 1, 1)]
void seam_main(uint3 group_id : SV_GroupID, uint group_thread : SV_GroupIndex)
{
    uint active_index = group_id.x;
    uint axis = group_id.y;
    if (active_index >= terrain_shape.x)
    {
        return;
    }
    uint side = terrain_shape.y;
    uint row = active_index / side;
    uint column = active_index % side;
    bool has_neighbor = axis == 0 ? column + 1 < side : row + 1 < side;
    if (!has_neighbor)
    {
        return;
    }
    uint neighbor_index = active_index + (axis == 0 ? 1u : side);
    uint mapping = mapping_at(active_index);
    uint neighbor_mapping = mapping_at(neighbor_index);
    if (group_thread == 0)
    {
        uint ignored;
        seam_stats.InterlockedAdd(0, 1, ignored);
    }
    if (group_thread >= SAMPLE_SIDE)
    {
        return;
    }
    uint x = axis == 0 ? CELL_SIDE : group_thread;
    uint z = axis == 0 ? group_thread : CELL_SIDE;
    uint neighbor_x = axis == 0 ? 0u : group_thread;
    uint neighbor_z = axis == 0 ? group_thread : 0u;
    int value = load_height(mapping_slot(mapping), x, z);
    int neighbor_value = load_height(mapping_slot(neighbor_mapping), neighbor_x, neighbor_z);
    uint ignored;
    seam_stats.InterlockedAdd(4, 1, ignored);
    if (value != neighbor_value)
    {
        seam_stats.InterlockedAdd(8, 1, ignored);
        uint mismatch_key = (active_index << 8u) | (axis << 7u) | group_thread;
        seam_stats.InterlockedMin(12, mismatch_key, ignored);
    }
}

int edge_numerator(
    uint mapping,
    uint patch_index,
    uint axis,
    bool positive_edge,
    uint sample,
    uint step)
{
    uint patch_x = (patch_index % 4u) * 8u;
    uint patch_z = (patch_index / 4u) * 8u;
    uint base = sample / step * step;
    uint next = min(base + step, 8u);
    uint remainder = sample - base;
    uint fixed_axis = positive_edge ? 8u : 0u;
    uint x0 = patch_x + (axis == 0 ? fixed_axis : base);
    uint z0 = patch_z + (axis == 0 ? base : fixed_axis);
    uint x1 = patch_x + (axis == 0 ? fixed_axis : next);
    uint z1 = patch_z + (axis == 0 ? next : fixed_axis);
    int height0 = load_height(mapping_slot(mapping), x0, z0);
    int height1 = load_height(mapping_slot(mapping), x1, z1);
    return height0 * int(step - remainder) + height1 * int(remainder);
}

[numthreads(16, 1, 1)]
void lod_seam_main(
    uint3 group_id : SV_GroupID,
    uint group_thread : SV_GroupIndex)
{
    uint active_index;
    uint patch_index;
    uint grid_x;
    uint grid_z;
    int global_x;
    int global_z;
    patch_address(
        group_id.x,
        active_index,
        patch_index,
        grid_x,
        grid_z,
        global_x,
        global_z
    );
    if (active_index >= terrain_shape.x)
    {
        return;
    }
    uint patch_side = terrain_shape.y * 4u;
    uint axis = group_id.y;
    bool has_neighbor = axis == 0 ? grid_x + 1u < patch_side : grid_z + 1u < patch_side;
    if (!has_neighbor)
    {
        return;
    }
    uint neighbor_mapping;
    uint neighbor_patch_index;
    grid_patch(
        grid_x + (axis == 0 ? 1u : 0u),
        grid_z + (axis == 0 ? 0u : 1u),
        neighbor_mapping,
        neighbor_patch_index
    );
    uint mapping = mapping_at(active_index);
    uint lod = select_patch_lod(global_x, global_z);
    uint neighbor_lod = select_patch_lod(
        global_x + (axis == 0 ? 1 : 0),
        global_z + (axis == 0 ? 0 : 1)
    );
    uint delta = lod > neighbor_lod ? lod - neighbor_lod : neighbor_lod - lod;
    uint coarse_step = 1u << max(lod, neighbor_lod);
    if (group_thread == 0)
    {
        uint ignored;
        lod_stats.InterlockedAdd(12, 1, ignored);
        lod_stats.InterlockedAdd(delta == 0 ? 16 : 20, 1, ignored);
        lod_stats.InterlockedMax(32, delta, ignored);
        if (delta != 0)
        {
            uint fine_step = 1u << min(lod, neighbor_lod);
            uint adjusted = 0;
            for (uint sample = 0; sample <= 8u; sample += fine_step)
            {
                adjusted += sample % coarse_step != 0 ? 1u : 0u;
            }
            lod_stats.InterlockedAdd(24, adjusted, ignored);
        }
    }
    if (group_thread > 8u)
    {
        return;
    }
    int value = edge_numerator(
        mapping,
        patch_index,
        axis,
        true,
        group_thread,
        coarse_step
    );
    int neighbor_value = edge_numerator(
        neighbor_mapping,
        neighbor_patch_index,
        axis,
        false,
        group_thread,
        coarse_step
    );
    uint ignored;
    lod_stats.InterlockedAdd(28, 1, ignored);
    if (value != neighbor_value)
    {
        lod_stats.InterlockedAdd(36, 1, ignored);
        uint mismatch_key = group_id.x * 32u + axis * 16u + group_thread;
        lod_stats.InterlockedMin(40, mismatch_key, ignored);
    }
}

struct TerrainPayload
{
    uint mapping;
    uint patch_index;
    uint lod;
    uint4 neighbor_lod;
};

groupshared TerrainPayload amplification_payload;

[numthreads(1, 1, 1)]
void as_main(uint3 group_id : SV_GroupID)
{
    uint active_index;
    uint patch_index;
    uint grid_x;
    uint grid_z;
    int global_x;
    int global_z;
    patch_address(
        group_id.x,
        active_index,
        patch_index,
        grid_x,
        grid_z,
        global_x,
        global_z
    );
    bool active = active_index < terrain_shape.x;
    uint ignored;
    if (active)
    {
        uint lod = select_patch_lod(global_x, global_z);
        uint cell_side = 8u >> lod;
        terrain_stats.InterlockedAdd(4, 1, ignored);
        terrain_stats.InterlockedAdd(12, (cell_side + 1u) * (cell_side + 1u), ignored);
        terrain_stats.InterlockedAdd(16, cell_side * cell_side * 2u, ignored);
        lod_stats.InterlockedAdd(lod * 4u, 1, ignored);
        amplification_payload.mapping = mapping_at(active_index);
        amplification_payload.patch_index = patch_index;
        amplification_payload.lod = lod;
        uint patch_side = terrain_shape.y * 4u;
        amplification_payload.neighbor_lod = uint4(
            grid_x > 0 ? select_patch_lod(global_x - 1, global_z) : lod,
            grid_x + 1u < patch_side ? select_patch_lod(global_x + 1, global_z) : lod,
            grid_z > 0 ? select_patch_lod(global_x, global_z - 1) : lod,
            grid_z + 1u < patch_side ? select_patch_lod(global_x, global_z + 1) : lod
        );
    }
    else
    {
        terrain_stats.InterlockedAdd(8, 1, ignored);
    }
    DispatchMesh(active ? 1u : 0u, 1, 1, amplification_payload);
}

struct TerrainVertexOutput
{
    float4 position : SV_POSITION;
    float3 normal : NORMAL0;
    nointerpolation uint object_id : TEXCOORD0;
};

struct TerrainPrimitiveOutput
{
    nointerpolation uint material : TEXCOORD1;
};

[outputtopology("triangle")]
[numthreads(128, 1, 1)]
void ms_main(
    uint group_thread : SV_GroupIndex,
    in payload TerrainPayload payload,
    out vertices TerrainVertexOutput output_vertices[81],
    out primitives TerrainPrimitiveOutput output_primitives[128],
    out indices uint3 output_triangles[128])
{
    uint step = 1u << payload.lod;
    uint cell_side = 8u >> payload.lod;
    uint vertex_side = cell_side + 1u;
    uint vertex_count = vertex_side * vertex_side;
    uint triangle_count = cell_side * cell_side * 2u;
    SetMeshOutputCounts(vertex_count, triangle_count);
    uint slot = mapping_slot(payload.mapping);
    uint region_id = mapping_region(payload.mapping);
    uint patch_x = (payload.patch_index % 4u) * 8u;
    uint patch_z = (payload.patch_index / 4u) * 8u;
    if (group_thread < vertex_count)
    {
        uint local_x = (group_thread % vertex_side) * step;
        uint local_z = (group_thread / vertex_side) * step;
        uint x = patch_x + local_x;
        uint z = patch_z + local_z;
        int height = load_height(slot, x, z);
        int left = load_height(slot, x < step ? 0 : x - step, z);
        int right = load_height(slot, x + step > CELL_SIDE ? CELL_SIDE : x + step, z);
        int top = load_height(slot, x, z < step ? 0 : z - step);
        int bottom = load_height(slot, x, z + step > CELL_SIDE ? CELL_SIDE : z + step);
        float tangent_span = float(step);
        float3 tangent_x = float3(x < step || x + step > CELL_SIDE ? tangent_span * 0.5 : tangent_span, float(right - left) / 256.0, 0.0);
        float3 tangent_z = float3(0.0, float(bottom - top) / 256.0, z < step || z + step > CELL_SIDE ? tangent_span * 0.5 : tangent_span);
        TerrainVertexOutput output;
        output.position = mul(view_projection, float4(terrain_position(region_id, x, z, height), 1.0));
        uint coarse_lod = payload.lod;
        uint along = 0;
        if (local_x == 0 && payload.neighbor_lod.x > coarse_lod)
        {
            coarse_lod = payload.neighbor_lod.x;
            along = local_z;
        }
        else if (local_x == 8u && payload.neighbor_lod.y > coarse_lod)
        {
            coarse_lod = payload.neighbor_lod.y;
            along = local_z;
        }
        else if (local_z == 0 && payload.neighbor_lod.z > coarse_lod)
        {
            coarse_lod = payload.neighbor_lod.z;
            along = local_x;
        }
        else if (local_z == 8u && payload.neighbor_lod.w > coarse_lod)
        {
            coarse_lod = payload.neighbor_lod.w;
            along = local_x;
        }
        uint coarse_step = 1u << coarse_lod;
        if (coarse_lod > payload.lod && along % coarse_step != 0)
        {
            uint base = along / coarse_step * coarse_step;
            uint next = min(base + coarse_step, 8u);
            uint x0 = patch_x + (local_x == 0 || local_x == 8u ? local_x : base);
            uint z0 = patch_z + (local_z == 0 || local_z == 8u ? local_z : base);
            uint x1 = patch_x + (local_x == 0 || local_x == 8u ? local_x : next);
            uint z1 = patch_z + (local_z == 0 || local_z == 8u ? local_z : next);
            int height0 = load_height(slot, x0, z0);
            int height1 = load_height(slot, x1, z1);
            float4 clip0 = mul(view_projection, float4(terrain_position(region_id, x0, z0, height0), 1.0));
            float4 clip1 = mul(view_projection, float4(terrain_position(region_id, x1, z1, height1), 1.0));
            output.position = lerp(clip0, clip1, float(along - base) / float(coarse_step));
        }
        output.normal = normalize(cross(tangent_z, tangent_x));
        output.object_id = TERRAIN_OBJECT_ID_BASE + region_id + 1u;
        output_vertices[group_thread] = output;
        uint ignored;
        terrain_stats.InterlockedMin(24, height, ignored);
        terrain_stats.InterlockedMax(28, height, ignored);
    }
    if (group_thread < triangle_count)
    {
        uint cell = group_thread / 2u;
        uint cell_x = cell % cell_side;
        uint cell_z = cell / cell_side;
        uint v00 = cell_z * vertex_side + cell_x;
        uint v10 = v00 + 1u;
        uint v01 = v00 + vertex_side;
        uint v11 = v01 + 1u;
        output_triangles[group_thread] = group_thread % 2u == 0
            ? uint3(v00, v01, v10)
            : uint3(v10, v01, v11);
        output_primitives[group_thread].material = load_material(
            slot,
            patch_x + cell_x * step,
            patch_z + cell_z * step
        );
    }
}

struct TerrainPixelOutput
{
    float4 color : SV_TARGET0;
    uint object_id : SV_TARGET1;
};

TerrainPixelOutput ps_main(TerrainVertexOutput input, TerrainPrimitiveOutput primitive)
{
    const float3 palette[8] = {
        float3(0.12, 0.34, 0.19), float3(0.20, 0.46, 0.25),
        float3(0.38, 0.52, 0.24), float3(0.50, 0.45, 0.28),
        float3(0.35, 0.38, 0.42), float3(0.48, 0.52, 0.55),
        float3(0.16, 0.30, 0.34), float3(0.55, 0.58, 0.47)
    };
    float3 light_direction = normalize(float3(-0.45, 0.8, 0.3));
    float lighting = 0.28 + 0.72 * saturate(dot(normalize(input.normal), light_direction));
    TerrainPixelOutput output;
    output.color = float4(palette[primitive.material & 7u] * lighting, 1.0);
    output.object_id = input.object_id;
    return output;
}
