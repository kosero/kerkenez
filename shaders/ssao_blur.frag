#version 410 core

in vec2 v_TexCoords;
out float FragColor;

uniform sampler2D u_SSAOTexture;
uniform sampler2D u_DepthTexture;

uniform float u_Near;
uniform float u_Far;
uniform vec2 u_Resolution;



uniform vec2 u_Direction;

// Depth-aware Bilateral Blur to preserve edges
const float BLUR_DEPTH_FALLOFF = 50.0;

void main() {
    vec2 texelSize = 1.0 / u_Resolution;
    float result = 0.0;
    float weightSum = 0.0;
    
    // Normalized linear depth for consistent falloff
    float centerDepth = LinearizeDepth(texture(u_DepthTexture, v_TexCoords).r, u_Near, u_Far) / u_Far;
    
    for (int i = -3; i <= 3; ++i) {
        vec2 offset = u_Direction * float(i) * texelSize;
        vec2 sampleUV = clamp(v_TexCoords + offset, 0.0, 1.0);
        float sampleDepth = LinearizeDepth(texture(u_DepthTexture, sampleUV).r, u_Near, u_Far) / u_Far;
        float sampleSSAO = texture(u_SSAOTexture, sampleUV).r;
        
        // Spatial weight (Gaussian-like)
        float spatialWeight = exp(-(float(i * i)) / 18.0);
        
        // Bilateral weight based on normalized depth difference
        float depthDiff = abs(centerDepth - sampleDepth);
        float depthWeight = exp(-depthDiff * 200.0);
        
        float weight = spatialWeight * depthWeight;
        
        result += sampleSSAO * weight;
        weightSum += weight;
    }
    
    FragColor = result / weightSum;
}
