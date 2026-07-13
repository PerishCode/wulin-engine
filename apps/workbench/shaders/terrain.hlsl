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
};

ByteAddressBuffer terrain_tiles[REGION_SLOT_CAPACITY] : register(t0);
RWByteAddressBuffer terrain_stats : register(u0);
RWByteAddressBuffer seam_stats : register(u1);

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

struct TerrainPayload
{
    uint mapping;
    uint patch_index;
};

groupshared TerrainPayload amplification_payload;

[numthreads(1, 1, 1)]
void as_main(uint3 group_id : SV_GroupID)
{
    uint active_index = group_id.x / PATCHES_PER_REGION;
    bool active = active_index < terrain_shape.x;
    uint ignored;
    if (active)
    {
        terrain_stats.InterlockedAdd(4, 1, ignored);
        terrain_stats.InterlockedAdd(12, 81, ignored);
        terrain_stats.InterlockedAdd(16, 128, ignored);
        amplification_payload.mapping = mapping_at(active_index);
        amplification_payload.patch_index = group_id.x % PATCHES_PER_REGION;
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
    SetMeshOutputCounts(81, 128);
    uint slot = mapping_slot(payload.mapping);
    uint region_id = mapping_region(payload.mapping);
    uint patch_x = (payload.patch_index % 4u) * 8u;
    uint patch_z = (payload.patch_index / 4u) * 8u;
    if (group_thread < 81)
    {
        uint local_x = group_thread % 9u;
        uint local_z = group_thread / 9u;
        uint x = patch_x + local_x;
        uint z = patch_z + local_z;
        int height = load_height(slot, x, z);
        int left = load_height(slot, x == 0 ? 0 : x - 1, z);
        int right = load_height(slot, x == CELL_SIDE ? CELL_SIDE : x + 1, z);
        int top = load_height(slot, x, z == 0 ? 0 : z - 1);
        int bottom = load_height(slot, x, z == CELL_SIDE ? CELL_SIDE : z + 1);
        float3 tangent_x = float3(x == 0 || x == CELL_SIDE ? 0.5 : 1.0, float(right - left) / 256.0, 0.0);
        float3 tangent_z = float3(0.0, float(bottom - top) / 256.0, z == 0 || z == CELL_SIDE ? 0.5 : 1.0);
        TerrainVertexOutput output;
        output.position = mul(view_projection, float4(terrain_position(region_id, x, z, height), 1.0));
        output.normal = normalize(cross(tangent_z, tangent_x));
        output.object_id = TERRAIN_OBJECT_ID_BASE + region_id + 1u;
        output_vertices[group_thread] = output;
        uint ignored;
        terrain_stats.InterlockedMin(24, height, ignored);
        terrain_stats.InterlockedMax(28, height, ignored);
    }
    if (group_thread < 128)
    {
        uint cell = group_thread / 2u;
        uint cell_x = cell % 8u;
        uint cell_z = cell / 8u;
        uint v00 = cell_z * 9u + cell_x;
        uint v10 = v00 + 1u;
        uint v01 = v00 + 9u;
        uint v11 = v01 + 1u;
        output_triangles[group_thread] = group_thread % 2u == 0
            ? uint3(v00, v01, v10)
            : uint3(v10, v01, v11);
        output_primitives[group_thread].material = load_material(
            slot,
            patch_x + cell_x,
            patch_z + cell_z
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
