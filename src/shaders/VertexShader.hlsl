#include "GlobalHeader.hlsli"

// t0 matches the descriptor set slot you'll define in C++
StructuredBuffer<VertexData> VertexBuffer : register(t0);
ConstantBuffer<SceneData> Scene : register(b1);

struct VSOutput
{
    float4 position : SV_POSITION;
    float4 color : TEXCOORD0;
};

VSOutput main(uint vID : SV_VertexID)
{
    VSOutput output;

    // Pull data directly from memory
    VertexData v = VertexBuffer[vID];

    // Math-based transformation
    float4 worldPos = mul(Scene.modelMatrix, float4(v.pos, 1.0));
    
    output.position = worldPos;
    output.color = v.color * Scene.baseColor;

    return output;
}