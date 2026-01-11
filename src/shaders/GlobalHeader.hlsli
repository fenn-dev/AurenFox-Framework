// GlobalHeader.hlsli

struct VertexData
{
    float3 pos; // 12 bytes
    float padding; // 4 bytes (Alignment filler)
    float4 color; // 16 bytes
};

// This represents the data for a whole object
struct SceneData
{
    float4x4 modelMatrix;
    float4 baseColor;
    float time; // For math-based animations
    float3 padding;
};