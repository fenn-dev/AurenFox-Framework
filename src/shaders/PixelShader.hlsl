#include "GlobalHeader.hlsli"

struct VSOutput
{
    float4 position : SV_POSITION;
    float4 color : TEXCOORD0;
};

float4 main(VSOutput input) : SV_Target
{
    // Math Logic: Add a subtle pulse effect based on Scene.time
    // No 'if' statements, just a sine wave oscillation
    // float pulse = sin(Scene.time) * 0.5 + 0.5;
    
    // Return the final color (interpolated by the hardware)
    return input.color;
}