RWStructuredBuffer<uint> output : register(u0);

cbuffer Parameters : register(b0)
{
    uint element_count;
    uint seed;
    uint row_stride;
};

uint hash_u32(uint value)
{
    value ^= value >> 16;
    value *= 0x7feb352d;
    value ^= value >> 15;
    value *= 0x846ca68b;
    value ^= value >> 16;
    return value;
}

[numthreads(256, 1, 1)]
void main(uint3 dispatch_id : SV_DispatchThreadID)
{
    uint index = dispatch_id.x + dispatch_id.y * row_stride;
    if (index < element_count)
    {
        output[index] = hash_u32(index ^ seed);
    }
}
