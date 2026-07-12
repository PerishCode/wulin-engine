cbuffer FrameConstants : register(b0)
{
    column_major float4x4 view_projection;
};

cbuffer DrawConstants : register(b1)
{
    column_major float4x4 model;
    float4 object_color;
    uint material_kind;
};

struct VertexInput
{
    float3 position : POSITION;
    float3 normal : NORMAL;
};

struct PixelInput
{
    float4 position : SV_POSITION;
    float3 world_position : TEXCOORD0;
    float3 world_normal : NORMAL;
    float4 color : COLOR0;
    nointerpolation uint material : TEXCOORD1;
};

PixelInput vs_main(VertexInput input)
{
    PixelInput output;
    float4 world = mul(model, float4(input.position, 1.0));
    output.position = mul(view_projection, world);
    output.world_position = world.xyz;
    output.world_normal = normalize(input.normal);
    output.color = object_color;
    output.material = material_kind;
    return output;
}

float4 ps_main(PixelInput input) : SV_TARGET
{
    if (input.material == 1)
    {
        int checker = ((int)floor(input.world_position.x) + (int)floor(input.world_position.z)) & 1;
        float factor = checker == 0 ? 0.72 : 1.0;
        return float4(input.color.rgb * factor, 1.0);
    }

    float3 light_direction = normalize(float3(-0.45, 0.8, 0.3));
    float lighting = 0.28 + 0.72 * saturate(dot(normalize(input.world_normal), light_direction));
    return float4(input.color.rgb * lighting, 1.0);
}
