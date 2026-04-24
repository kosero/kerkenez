#version 410 core

in vec2 v_TexCoords;
out vec4 FragColor;

// G-Buffer Textures
uniform sampler2D u_ScreenTexture;   // RT0: Albedo
uniform sampler2D u_DepthTexture;    // Depth
uniform sampler2D u_NormalTexture;   // RT1: World-space Normal
uniform sampler2D u_SSAOTexture;

// Camera
uniform float u_Near;
uniform float u_Far;
uniform mat4 u_InverseVP;
uniform vec2 u_Resolution;

// Debug
#ifndef DEBUG_MODE
#define DEBUG_MODE 0
#endif

// SSAO
uniform bool u_SSAOEnabled;
uniform float u_SSAORadius;
uniform float u_SSAOIntensity;
uniform float u_SSAOBias;

// Fog
uniform bool u_FogEnabled;
uniform float u_FogDensity;
uniform vec3 u_FogColor;

// Tone Mapping & Color Grading
uniform bool u_ToneMappingEnabled;
uniform float u_Exposure;
uniform float u_Contrast;
uniform float u_Brightness;
uniform float u_Saturation;

// Vignette
uniform bool u_VignetteEnabled;
uniform float u_VignetteIntensity;

//  Utility Functions

float LinearizeDepth(float depth) {
    float z = depth * 2.0 - 1.0; // NDC
    return (2.0 * u_Near * u_Far) / (u_Far + u_Near - z * (u_Far - u_Near));
}

vec3 WorldPosFromDepth(vec2 uv, float depth) {
    vec4 ndc = vec4(uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
    vec4 wp = u_InverseVP * ndc;
    return wp.xyz / wp.w;
}

//  G-Buffer Normal Fetch (replaces depth-based reconstruction)

vec3 GetNormal(vec2 uv) {
    return normalize(texture(u_NormalTexture, uv).rgb);
}

//  Screen Edge Fade (SSAO Halo Fix)
float GetScreenEdgeFade(vec2 uv) {
    vec2 edge = smoothstep(vec2(0.0), vec2(0.05), uv) * (1.0 - smoothstep(vec2(0.95), vec2(1.0), uv));
    return edge.x * edge.y;
}

//  Exponential Squared Fog
vec3 ApplyFog(vec3 color, float linearDepth) {
    float fogFactor = exp(-pow(u_FogDensity * linearDepth, 2.0));
    fogFactor = clamp(fogFactor, 0.0, 1.0);
    return mix(u_FogColor, color, fogFactor);
}

//  ACES Filmic Tone Mapping
vec3 ACESFilm(vec3 x) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

//  Color Grading (Post-Tonemap)
vec3 ApplyColorGrading(vec3 color) {
    color = mix(vec3(0.5), color, u_Contrast);
    color += vec3(u_Brightness);
    float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
    color = mix(vec3(luma), color, u_Saturation);
    return max(color, vec3(0.0));
}

//  Cinematic Vignette
vec3 ApplyVignette(vec3 color, vec2 uv) {
    uv = uv * 2.0 - 1.0;
    float dist = dot(uv, uv);
    float vignette = smoothstep(0.8, u_VignetteIntensity * 0.799, dist);
    return color * clamp(vignette, 0.0, 1.0);
}

//  Dithering (Banding / Şeritlenme Önleyici)
vec3 ApplyDithering(vec3 color, vec2 uv) {
    float dither = fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233))) * 43758.5453) / 255.0;
    return color + dither;
}

//  Main
void main() {
    float rawDepth = texture(u_DepthTexture, v_TexCoords).r;
    float linearDepth = LinearizeDepth(rawDepth);

    #if DEBUG_MODE == 1
    float d = linearDepth / u_Far;
    FragColor = vec4(vec3(d), 1.0);
    #elif DEBUG_MODE == 2
    // G-Buffer normal visualization (no more depth reconstruction needed)
    vec3 n = GetNormal(v_TexCoords) * 0.5 + 0.5;
    FragColor = vec4(n, 1.0);
    #elif DEBUG_MODE == 3
    float ao = texture(u_SSAOTexture, v_TexCoords).r;
    FragColor = vec4(vec3(ao), 1.0);
    #else

    vec3 color = texture(u_ScreenTexture, v_TexCoords).rgb;

    color *= u_Exposure;

    if (u_SSAOEnabled) {
        float ao = texture(u_SSAOTexture, v_TexCoords).r;

        ao = clamp(ao, 0.0, 1.0);

        float mask = GetScreenEdgeFade(v_TexCoords);

        float ao_final = 1.0 - ((1.0 - ao) * mask);

        color *= ao_final;
    }

    // 3. Fog
    if (u_FogEnabled) {
        color = ApplyFog(color, linearDepth);
    }

    // 4. Tone Mapping
    if (u_ToneMappingEnabled) {
        color = ACESFilm(color);
    }

    // 5. Color Grading
    color = ApplyColorGrading(color);

    // 6. Vignette
    if (u_VignetteEnabled) {
        color = ApplyVignette(color, v_TexCoords);
    }

    // 7. Gamma Correction
    color = pow(max(color, vec3(0.0)), vec3(1.0 / 2.2));

    // 8. Dithering
    color = ApplyDithering(color, v_TexCoords);

    FragColor = vec4(color, 1.0);
    #endif
}
