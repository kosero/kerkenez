#version 410 core

in vec2 v_TexCoords;
out float FragColor;

uniform sampler2D u_DepthTexture;

uniform float u_Near;
uniform float u_Far;
uniform vec2 u_Resolution;
uniform mat4 u_InverseVP;

uniform float u_SSAORadius;
uniform float u_SSAOIntensity;
uniform float u_SSAOBias;

float LinearizeDepth(float depth) {
    float z = depth * 2.0 - 1.0; // NDC
    return (2.0 * u_Near * u_Far) / (u_Far + u_Near - z * (u_Far - u_Near));
}

const int SSAO_SAMPLES = 16;
const float PI2 = 6.28318530718;

float Hash(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

void main() {
    float rawDepth = texture(u_DepthTexture, v_TexCoords).r;
    float linearDepth = LinearizeDepth(rawDepth);

    if (linearDepth >= u_Far * 0.99) {
        FragColor = 1.0;
        return;
    }

    vec2 texelSize = 1.0 / u_Resolution;

    // Screen-space slope correction to prevent self-shadowing on flat surfaces
    float depthRight = LinearizeDepth(texture(u_DepthTexture, v_TexCoords + vec2(texelSize.x, 0.0)).r);
    float depthUp = LinearizeDepth(texture(u_DepthTexture, v_TexCoords + vec2(0.0, texelSize.y)).r);
    float dDepthX = depthRight - linearDepth;
    float dDepthY = depthUp - linearDepth;

    if (abs(dDepthX) > 0.5) dDepthX = 0.0;
    if (abs(dDepthY) > 0.5) dDepthY = 0.0;

    float radius = clamp(u_SSAORadius / linearDepth, 0.005, 0.2);
    float noise = Hash(gl_FragCoord.xy * 12.34);

    float occlusion = 0.0;
    float sampleCount = float(SSAO_SAMPLES);
    float validSamples = 0.0;

    for (int i = 0; i < SSAO_SAMPLES; i++) {
        float t = (float(i) + noise) / sampleCount;
        float r = sqrt(t);
        float theta = t * PI2 * 8.5;

        vec2 offset = vec2(cos(theta), sin(theta)) * r * radius;
        vec2 sampleUV = v_TexCoords + offset;

        if (sampleUV.x < 0.0 || sampleUV.x > 1.0 || sampleUV.y < 0.0 || sampleUV.y > 1.0) {
            continue;
        }

        validSamples += 1.0;

        float sampleLin = LinearizeDepth(texture(u_DepthTexture, sampleUV).r);

        // Expected depth if surface was perfectly flat
        float expectedDepth = linearDepth + dDepthX * (offset.x / texelSize.x) + dDepthY * (offset.y / texelSize.y);

        float delta = expectedDepth - sampleLin;

        float rangeCheck = smoothstep(0.0, 1.0, u_SSAORadius / (abs(delta) + 0.01));
        occlusion += smoothstep(u_SSAOBias, u_SSAOBias * 2.0, delta) * rangeCheck;
    }

    float ao = 1.0;
    if (validSamples > 0.0) {
        ao = 1.0 - (occlusion / validSamples) * u_SSAOIntensity;
    }
    FragColor = clamp(ao, 0.0, 1.0);
}
