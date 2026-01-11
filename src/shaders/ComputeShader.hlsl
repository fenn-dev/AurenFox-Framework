#include "GlobalHeader.hlsli"

// Input buffer of all potential vertices
StructuredBuffer<VertexData> AllVertices : register(t0);
// Output buffer of vertices that are actually "active"
RWStructuredBuffer<VertexData> ActiveVertices : register(u1);

[numthreads(64, 1, 1)]
void main(uint3 id : SV_DispatchThreadID)
{
    VertexData v = AllVertices[id.x];
    
    // MATH LOGIC: Instead of an 'if', we use math to "remove" 
    // vertices by moving them to infinity or scaling to zero
    // Example: Only keep vertices with a positive Y value
    float visible = step(0.0, v.pos.y);
    
    v.pos *= visible; // If visible is 0, the vertex collapses to (0,0,0)
    
    ActiveVertices[id.x] = v;
}