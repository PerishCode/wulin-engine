static const uint REGION_OBJECT_ID_BASE = 65536;
static const uint ACTOR_OBJECT_ID = 98305;
static const uint ACTOR_CANDIDATE_INDEX = 25600;
static const uint MAX_BONES = 128;
static const uint MATERIAL_TEXTURE_SIDE = 64;
static const uint SAMPLE_COUNT = 6;
static const uint SHADOW_MAP_SIDE = 1024;
static const float SHADOW_RECEIVER_BIAS = 0.0015;

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

struct MeshletDescriptor
{
    uint vertex_offset;
    uint vertex_count;
    uint primitive_offset;
    uint primitive_count;
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

struct SurfaceVertex
{
    float4 oct_normal_uv;
};

struct SurfacePrimitive
{
    uint4 vertex_indices;
};

struct MaterialRecord
{
    float4 base_color;
    uint texture_layer;
    float roughness;
    float metallic;
    uint reserved;
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
    column_major float4x4 light_view_projection;
};

StructuredBuffer<VisibleObject> draw_objects : register(t50);
StructuredBuffer<float4> catalog_vertices : register(t51);
StructuredBuffer<MeshletDescriptor> catalog_meshlets : register(t52);
StructuredBuffer<uint> catalog_meshlet_vertices : register(t53);
StructuredBuffer<uint> catalog_primitives : register(t54);
StructuredBuffer<LodDescriptor> catalog_lods : register(t55);
StructuredBuffer<SkinBinding> catalog_skin : register(t59);
StructuredBuffer<AffineTransform> palette_in : register(t62);
StructuredBuffer<SurfaceVertex> surface_vertices : register(t63);
StructuredBuffer<SurfacePrimitive> surface_primitives : register(t64);
StructuredBuffer<MaterialRecord> surface_materials : register(t65);
Texture2D<uint2> visibility_texture : register(t66);
Texture2DArray<float4> material_texture : register(t67);
StructuredBuffer<uint> candidate_to_visible_in : register(t68);
StructuredBuffer<VisibleObject> source_visible : register(t60);
Texture2D<float> shadow_depth : register(t71);

RWStructuredBuffer<uint> candidate_to_visible_out : register(u7);
RWTexture2D<float4> resolved_color : register(u8);
RWByteAddressBuffer surface_stats : register(u9);
RWByteAddressBuffer surface_samples : register(u10);
RasterizerOrderedTexture2D<uint2> visibility_winner : register(u11);

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
    candidate_to_visible_out[visible.candidate_index] = group_id.x;
    amplification_payload.visible_index = group_id.x;
    amplification_payload.meshlet_offset = descriptor.meshlet_offset;
    DispatchMesh(descriptor.meshlet_count, 1, 1, amplification_payload);
}

struct MeshVertexOutput
{
    float4 position : SV_POSITION;
    nointerpolation uint candidate_index : TEXCOORD0;
    nointerpolation uint object_id : TEXCOORD1;
};

struct MeshPrimitiveOutput
{
    uint primitive_id : SV_PrimitiveID;
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

float3 resolve_world_position(
    uint vertex_index,
    VisibleObject visible,
    float sine,
    float cosine)
{
    float3 local = catalog_vertices[vertex_index].xyz;
    bool imported = visible.archetype == 7u;
    if (!imported)
    {
        local.y *= visible.height;
    }
    if (visible.pose_slot != 0xffffffffu)
    {
        SkinBinding binding = catalog_skin[vertex_index];
        float3 skinned = 0.0;
        [unroll]
        for (uint influence = 0; influence < 4; influence++)
        {
            uint bone = ((binding.indices >> (influence * 8u)) & 255u)
                % surface_animation.x;
            float weight = float((binding.weights >> (influence * 8u)) & 255u) / 255.0;
            skinned += transform_point(
                palette_in[visible.pose_slot * MAX_BONES + bone],
                local
            ) * weight;
        }
        local = skinned;
    }
    if (imported)
    {
        local.y *= visible.height;
    }
    float3 rotated = float3(
        local.x * cosine - local.z * sine,
        local.y,
        local.x * sine + local.z * cosine
    );
    return visible.position + rotated;
}

[outputtopology("triangle")]
[numthreads(64, 1, 1)]
void ms_main(
    uint group_thread : SV_GroupIndex,
    uint3 group_id : SV_GroupID,
    in payload MeshPayload payload,
    out vertices MeshVertexOutput output_vertices[64],
    out primitives MeshPrimitiveOutput output_primitives[126],
    out indices uint3 output_triangles[126])
{
    VisibleObject visible = draw_objects[payload.visible_index];
    MeshletDescriptor meshlet = catalog_meshlets[payload.meshlet_offset + group_id.x];
    SetMeshOutputCounts(meshlet.vertex_count, meshlet.primitive_count);

    float angle = float(visible.yaw_q16) * 6.28318530718 / 65536.0;
    float sine;
    float cosine;
    sincos(angle, sine, cosine);
    if (group_thread < meshlet.vertex_count)
    {
        uint vertex_index = catalog_meshlet_vertices[meshlet.vertex_offset + group_thread];
        float3 local = catalog_vertices[vertex_index].xyz;
        bool imported = visible.archetype == 7u;
        if (!imported)
        {
            local.y *= visible.height;
        }
        if (visible.pose_slot != 0xffffffffu)
        {
            SkinBinding binding = catalog_skin[vertex_index];
            float3 skinned = 0.0;
            [unroll]
            for (uint influence = 0; influence < 4; influence++)
            {
                uint bone = ((binding.indices >> (influence * 8u)) & 255u)
                    % surface_animation.x;
                float weight = float((binding.weights >> (influence * 8u)) & 255u) / 255.0;
                skinned += transform_point(
                    palette_in[visible.pose_slot * MAX_BONES + bone],
                    local
                ) * weight;
            }
            local = skinned;
        }
        if (imported)
        {
            local.y *= visible.height;
        }
        float3 rotated = float3(
            local.x * cosine - local.z * sine,
            local.y,
            local.x * sine + local.z * cosine
        );
        MeshVertexOutput output;
        output.position = mul(view_projection, float4(visible.position + rotated, 1.0));
        output.candidate_index = visible.candidate_index;
        output.object_id = visible.candidate_index == ACTOR_CANDIDATE_INDEX
            ? ACTOR_OBJECT_ID
            : REGION_OBJECT_ID_BASE + visible.semantic_region + 1;
        output_vertices[group_thread] = output;
    }
    for (
        uint primitive_index = group_thread;
        primitive_index < meshlet.primitive_count;
        primitive_index += 64
    )
    {
        uint primitive = catalog_primitives[meshlet.primitive_offset + primitive_index];
        output_triangles[primitive_index] = uint3(
            primitive & 0xffu,
            (primitive >> 8) & 0xffu,
            (primitive >> 16) & 0xffu
        );
        output_primitives[primitive_index].primitive_id =
            meshlet.primitive_offset + primitive_index;
    }
}

struct ShadowVertexOutput
{
    float4 position : SV_POSITION;
};

[numthreads(1, 1, 1)]
void shadow_as_main(uint3 group_id : SV_GroupID)
{
    VisibleObject visible = source_visible[group_id.x];
    LodDescriptor descriptor = catalog_lods[visible.archetype * 3u + visible.lod];
    amplification_payload.visible_index = group_id.x;
    amplification_payload.meshlet_offset = descriptor.meshlet_offset;
    DispatchMesh(descriptor.meshlet_count, 1, 1, amplification_payload);
}

[outputtopology("triangle")]
[numthreads(64, 1, 1)]
void shadow_ms_main(
    uint group_thread : SV_GroupIndex,
    uint3 group_id : SV_GroupID,
    in payload MeshPayload payload,
    out vertices ShadowVertexOutput output_vertices[64],
    out indices uint3 output_triangles[126])
{
    VisibleObject visible = source_visible[payload.visible_index];
    MeshletDescriptor meshlet = catalog_meshlets[payload.meshlet_offset + group_id.x];
    SetMeshOutputCounts(meshlet.vertex_count, meshlet.primitive_count);
    float angle = float(visible.yaw_q16) * 6.28318530718 / 65536.0;
    float sine;
    float cosine;
    sincos(angle, sine, cosine);
    if (group_thread < meshlet.vertex_count)
    {
        uint vertex_index = catalog_meshlet_vertices[meshlet.vertex_offset + group_thread];
        ShadowVertexOutput output;
        output.position = mul(
            light_view_projection,
            float4(resolve_world_position(vertex_index, visible, sine, cosine), 1.0)
        );
        output_vertices[group_thread] = output;
    }
    for (
        uint primitive_index = group_thread;
        primitive_index < meshlet.primitive_count;
        primitive_index += 64
    )
    {
        uint primitive = catalog_primitives[meshlet.primitive_offset + primitive_index];
        output_triangles[primitive_index] = uint3(
            primitive & 0xffu,
            (primitive >> 8) & 0xffu,
            (primitive >> 16) & 0xffu
        );
    }
}

struct VisibilityOutput
{
    uint2 visibility : SV_TARGET0;
    uint object_id : SV_TARGET1;
};

VisibilityOutput ps_main(
    MeshVertexOutput input,
    MeshPrimitiveOutput primitive,
    float3 barycentrics : SV_Barycentrics)
{
    uint candidate = input.candidate_index + 1u;
    uint packed_primitive = primitive.primitive_id << 15u;
    uint identity = candidate | packed_primitive;
    uint2 pixel = uint2(input.position.xy);
    uint2 winner = visibility_winner[pixel];
    uint2 key = uint2(asuint(input.position.z), ~identity);
    if (key.x < winner.x || (key.x == winner.x && key.y <= winner.y))
    {
        discard;
    }
    visibility_winner[pixel] = key;
    VisibilityOutput output;
    uint2 bary = uint2(round(saturate(barycentrics.xy) * 65535.0));
    output.visibility = uint2(
        identity,
        bary.x | (bary.y << 16u)
    );
    output.object_id = input.object_id;
    return output;
}

float3 decode_octahedral(float2 encoded)
{
    float3 normal = float3(encoded, 1.0 - abs(encoded.x) - abs(encoded.y));
    if (normal.z < 0.0)
    {
        float x = normal.x;
        normal.x = (1.0 - abs(normal.y)) * (x < 0.0 ? -1.0 : 1.0);
        normal.y = (1.0 - abs(x)) * (normal.y < 0.0 ? -1.0 : 1.0);
    }
    return normalize(normal);
}

float3 transform_vector(AffineTransform transform, float3 vector)
{
    return float3(
        dot(transform.row0.xyz, vector),
        dot(transform.row1.xyz, vector),
        dot(transform.row2.xyz, vector)
    );
}

void resolve_vertex(
    uint vertex_index,
    VisibleObject visible,
    float sine,
    float cosine,
    out float3 world_position,
    out float3 normal,
    out float2 uv)
{
    world_position = resolve_world_position(
        vertex_index,
        visible,
        sine,
        cosine
    );
    SurfaceVertex surface = surface_vertices[vertex_index];
    normal = decode_octahedral(surface.oct_normal_uv.xy);
    bool imported = visible.archetype == 7u;
    if (!imported)
    {
        normal.y /= max(visible.height, 0.001);
    }
    if (visible.pose_slot != 0xffffffffu)
    {
        SkinBinding binding = catalog_skin[vertex_index];
        float3 skinned = 0.0;
        [unroll]
        for (uint influence = 0; influence < 4; influence++)
        {
            uint bone = ((binding.indices >> (influence * 8u)) & 255u)
                % surface_animation.x;
            float weight = float((binding.weights >> (influence * 8u)) & 255u) / 255.0;
            skinned += transform_vector(
                palette_in[visible.pose_slot * MAX_BONES + bone],
                normal
            ) * weight;
        }
        normal = skinned;
    }
    if (imported)
    {
        normal.y /= max(visible.height, 0.001);
    }
    normal = normalize(float3(
        normal.x * cosine - normal.z * sine,
        normal.y,
        normal.x * sine + normal.z * cosine
    ));
    uv = surface.oct_normal_uv.zw;
}

uint pack_rgba8(float4 color)
{
    uint4 bytes = uint4(round(saturate(color) * 255.0));
    return bytes.x | (bytes.y << 8u) | (bytes.z << 16u) | (bytes.w << 24u);
}

int sample_index(uint2 pixel)
{
    const uint2 samples[SAMPLE_COUNT] = {
        uint2(640, 360), uint2(600, 600), uint2(320, 500),
        uint2(960, 500), uint2(480, 420), uint2(800, 420)
    };
    [unroll]
    for (uint index = 0; index < SAMPLE_COUNT; index++)
    {
        if (all(pixel == samples[index]))
        {
            return int(index);
        }
    }
    return -1;
}

groupshared uint group_visible;
groupshared uint group_background;
groupshared uint group_material_low;
groupshared uint group_material_high;
groupshared uint group_targeted;

[numthreads(8, 8, 1)]
void shade_main(
    uint3 dispatch_thread : SV_DispatchThreadID,
    uint group_thread : SV_GroupIndex)
{
    if (group_thread == 0)
    {
        group_visible = 0;
        group_background = 0;
        group_material_low = 0;
        group_material_high = 0;
        group_targeted = 0;
    }
    GroupMemoryBarrierWithGroupSync();
    if (dispatch_thread.x >= surface_shape.z || dispatch_thread.y >= surface_shape.w)
    {
        return;
    }

    uint2 pixel = dispatch_thread.xy;
    uint2 payload = visibility_texture.Load(int3(pixel, 0));
    float4 color = background_color;
    uint visible_index = 0xffffffffu;
    uint2 stable_identity = uint2(0xffffffffu, 0xffffffffu);
    uint material_index = 0xffffffffu;
    uint packed_texel = 0xffffffffu;
    uint shadowed = 0u;
    uint packed_shadow_texel = 0xffffffffu;
    float receiver_shadow_depth = 1.0;
    float stored_shadow_depth = 1.0;
    if ((payload.x & 0x7fffu) != 0)
    {
        uint candidate = (payload.x & 0x7fffu) - 1u;
        uint primitive_index = (payload.x >> 15u) & 0xffffu;
        visible_index = candidate_to_visible_in[candidate];
        VisibleObject visible = draw_objects[visible_index];
        SurfacePrimitive primitive = surface_primitives[primitive_index];
        float2 decoded = float2(payload.y & 0xffffu, payload.y >> 16u) / 65535.0;
        float3 bary = float3(decoded, max(0.0, 1.0 - decoded.x - decoded.y));
        bary /= max(dot(bary, 1.0), 0.00001);
        float angle = float(visible.yaw_q16) * 6.28318530718 / 65536.0;
        float sine;
        float cosine;
        sincos(angle, sine, cosine);
        float3 normals[3];
        float3 world_positions[3];
        float2 uvs[3];
        [unroll]
        for (uint vertex = 0; vertex < 3; vertex++)
        {
            resolve_vertex(
                primitive.vertex_indices[vertex],
                visible,
                sine,
                cosine,
                world_positions[vertex],
                normals[vertex],
                uvs[vertex]
            );
        }
        float3 normal = normalize(
            normals[0] * bary.x + normals[1] * bary.y + normals[2] * bary.z
        );
        float2 uv = uvs[0] * bary.x + uvs[1] * bary.y + uvs[2] * bary.z;
        float3 world_position = world_positions[0] * bary.x
            + world_positions[1] * bary.y
            + world_positions[2] * bary.z;
        stable_identity = uint2(visible.stable_identity_low, visible.stable_identity_high);
        material_index = visible.material;
        MaterialRecord material = surface_materials[material_index];
        uint mip = surface_shape.y;
        uint side = max(1u, MATERIAL_TEXTURE_SIDE >> mip);
        float2 wrapped_uv = frac(uv);
        wrapped_uv = float2(
            wrapped_uv.x > 0.99999 ? 0.0 : wrapped_uv.x,
            wrapped_uv.y > 0.99999 ? 0.0 : wrapped_uv.y
        );
        uint2 texel = min(uint2(wrapped_uv * float(side)), side - 1u);
        packed_texel = texel.x | (texel.y << 16u);
        float4 texture_value = material_texture.Load(
            int4(texel, material.texture_layer, mip)
        );
        float3 light_direction = normalize(float3(-0.45, 0.8, 0.3));
        float diffuse = saturate(dot(normal, light_direction));
        float4 shadow_clip = mul(light_view_projection, float4(world_position, 1.0));
        float3 shadow_ndc = shadow_clip.xyz / shadow_clip.w;
        bool shadow_address_valid = all(shadow_ndc.xy >= -1.0)
            && all(shadow_ndc.xy <= 1.0)
            && shadow_ndc.z >= 0.0
            && shadow_ndc.z <= 1.0;
        if (shadow_address_valid)
        {
            uint2 shadow_texel = min(
                uint2(
                    (shadow_ndc.x * 0.5 + 0.5) * float(SHADOW_MAP_SIDE),
                    (-shadow_ndc.y * 0.5 + 0.5) * float(SHADOW_MAP_SIDE)
                ),
                SHADOW_MAP_SIDE - 1u
            );
            packed_shadow_texel = shadow_texel.x | (shadow_texel.y << 16u);
            receiver_shadow_depth = shadow_ndc.z;
            stored_shadow_depth = shadow_depth.Load(int3(shadow_texel, 0));
            shadowed = receiver_shadow_depth > stored_shadow_depth + SHADOW_RECEIVER_BIAS;
        }
        float direct_visibility = shadowed == 0u ? 1.0 : 0.0;
        float lighting = 0.22
            + direct_visibility * diffuse * (0.78 - material.roughness * 0.18);
        float metallic_lift = direct_visibility
            * material.metallic * pow(saturate(normal.y), 4.0) * 0.25;
        color = float4(
            saturate(material.base_color.rgb * texture_value.rgb * lighting + metallic_lift),
            1.0
        );
        bool targeted = surface_animation.y != 0u
            && candidate != ACTOR_CANDIDATE_INDEX
            && visible.semantic_region == surface_animation.z
            && visible.stable_identity_high == surface_animation.w;
        if (targeted)
        {
            color.rgb = surface_animation.y == 1u
                ? saturate(color.rgb * 0.45 + float3(1.0, 0.62, 0.08) * 0.55)
                : saturate(color.rgb * 0.30 + float3(0.12, 1.0, 0.32) * 0.70);
            InterlockedAdd(group_targeted, 1);
        }
        InterlockedAdd(group_visible, 1);
        if (material_index < 32)
        {
            InterlockedOr(group_material_low, 1u << material_index);
        }
        else
        {
            InterlockedOr(group_material_high, 1u << (material_index - 32u));
        }
    }
    else
    {
        InterlockedAdd(group_background, 1);
    }
    if ((payload.x & 0x7fffu) != 0)
    {
        resolved_color[pixel] = color;
    }

    int selected_sample = sample_index(pixel);
    if (selected_sample >= 0)
    {
        uint offset = uint(selected_sample) * 52u;
        surface_samples.Store(offset + 0, payload.x);
        surface_samples.Store(offset + 4, payload.y);
        surface_samples.Store(offset + 8, visible_index);
        surface_samples.Store2(offset + 12, stable_identity);
        surface_samples.Store(offset + 20, material_index);
        surface_samples.Store(offset + 24, surface_shape.y);
        surface_samples.Store(offset + 28, pack_rgba8(color));
        surface_samples.Store(offset + 32, packed_texel);
        surface_samples.Store(offset + 36, shadowed);
        surface_samples.Store(offset + 40, packed_shadow_texel);
        surface_samples.Store(offset + 44, asuint(receiver_shadow_depth));
        surface_samples.Store(offset + 48, asuint(stored_shadow_depth));
    }
    GroupMemoryBarrierWithGroupSync();
    if (group_thread == 0)
    {
        uint ignored;
        surface_stats.InterlockedAdd(4, group_visible, ignored);
        surface_stats.InterlockedAdd(8, group_background, ignored);
        surface_stats.InterlockedOr(12, group_material_low, ignored);
        surface_stats.InterlockedOr(16, group_material_high, ignored);
        surface_stats.InterlockedAdd(20, group_targeted, ignored);
    }
    if (all(pixel == uint2(0, 0)))
    {
        surface_stats.Store(0, surface_shape.z * surface_shape.w);
    }
}
