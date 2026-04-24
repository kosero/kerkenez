#version 410 core

in vec2 v_TexCoords;
out vec4 FragColor;

// ── Textures ────────────────────────────────────────────────────────
uniform sampler2D u_ScreenTexture;
uniform sampler2D u_DepthTexture;

// ── Camera ──────────────────────────────────────────────────────────
uniform float u_Near;
uniform float u_Far;
uniform mat4  u_InverseVP;
uniform vec2  u_Resolution;

// ── Debug ───────────────────────────────────────────────────────────
#ifndef DEBUG_MODE
#define DEBUG_MODE 0
#endif

// ── SSAO ────────────────────────────────────────────────────────────
uniform bool  u_SSAOEnabled;
uniform float u_SSAORadius;
uniform float u_SSAOIntensity;
uniform float u_SSAOBias;

// ── Fog ─────────────────────────────────────────────────────────────
uniform bool  u_FogEnabled;
uniform float u_FogDensity;
uniform vec3  u_FogColor;

// ── Tone Mapping & Color Grading ────────────────────────────────────
uniform bool  u_ToneMappingEnabled;
uniform float u_Exposure;
uniform float u_Contrast;
uniform float u_Brightness;
uniform float u_Saturation;

// ── Vignette ────────────────────────────────────────────────────────
uniform bool  u_VignetteEnabled;
uniform float u_VignetteIntensity;

// ═══════════════════════════════════════════════════════════════════
//  Utility Functions
// ═══════════════════════════════════════════════════════════════════

float LinearizeDepth(float depth) {
    float z = depth * 2.0 - 1.0;
    return (2.0 * u_Near * u_Far) / (u_Far + u_Near - z * (u_Far - u_Near));
}

vec3 WorldPosFromDepth(vec2 uv, float depth) {
    vec4 ndc = vec4(uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
    vec4 wp  = u_InverseVP * ndc;
    return wp.xyz / wp.w;
}

// ═══════════════════════════════════════════════════════════════════
//  Normal Reconstruction (Accurate, discontinuity-aware)
// ═══════════════════════════════════════════════════════════════════

vec3 ReconstructNormal(vec2 uv) {
    vec2 texel = 1.0 / u_Resolution;

    float dc = texture(u_DepthTexture, uv).r;
    float dl = texture(u_DepthTexture, uv - vec2(texel.x, 0.0)).r;
    float dr = texture(u_DepthTexture, uv + vec2(texel.x, 0.0)).r;
    float db = texture(u_DepthTexture, uv - vec2(0.0, texel.y)).r;
    float dt = texture(u_DepthTexture, uv + vec2(0.0, texel.y)).r;

    vec3 center = WorldPosFromDepth(uv, dc);

    // Pick the neighbor pair with the smallest depth delta
    // to avoid artifacts at depth discontinuities (edges).
    vec3 ddx = (abs(dl - dc) < abs(dr - dc))
        ? center - WorldPosFromDepth(uv - vec2(texel.x, 0.0), dl)
        : WorldPosFromDepth(uv + vec2(texel.x, 0.0), dr) - center;

    vec3 ddy = (abs(db - dc) < abs(dt - dc))
        ? center - WorldPosFromDepth(uv - vec2(0.0, texel.y), db)
        : WorldPosFromDepth(uv + vec2(0.0, texel.y), dt) - center;

    return normalize(cross(ddx, ddy));
}

// ═══════════════════════════════════════════════════════════════════
//  Screen-Space Ambient Occlusion (Vogel Disk — noise-free)
// ═══════════════════════════════════════════════════════════════════

const int   SSAO_SAMPLES = 24;
const float GOLDEN_ANGLE  = 2.3998277; // π(3 − √5)

// Generate a Vogel disk sample: deterministic, well-distributed spiral
vec2 VogelDiskSample(int index, int count, float phi) {
    float r     = sqrt((float(index) + 0.5) / float(count));
    float theta = float(index) * GOLDEN_ANGLE + phi;
    return vec2(cos(theta), sin(theta)) * r;
}

float ComputeSSAO(vec2 uv, float linearDepth) {
    // Skip sky / far plane
    if (linearDepth >= u_Far * 0.99) return 1.0;

    // Scale sample radius by inverse depth so it stays
    // perceptually consistent at different distances.
    float radius = u_SSAORadius / linearDepth;

    // Fixed rotation per 2×2 pixel block for subtle variation
    // without the per-pixel noise that causes grain.
    ivec2 px  = ivec2(gl_FragCoord.xy) % 4;
    float phi = float(px.x + px.y * 4) * GOLDEN_ANGLE;

    float occlusion = 0.0;

    for (int i = 0; i < SSAO_SAMPLES; i++) {
        vec2  offset    = VogelDiskSample(i, SSAO_SAMPLES, phi) * radius;
        vec2  sampleUV  = clamp(uv + offset, vec2(0.0), vec2(1.0));
        float sampleLin = LinearizeDepth(texture(u_DepthTexture, sampleUV).r);

        // How much closer is the sample than the center?
        float delta = linearDepth - sampleLin;

        // Only occlude if sample is in front (closer) and within a reasonable range
        float rangeCheck = smoothstep(0.0, 1.0, u_SSAORadius / (abs(delta) + 0.0001));
        occlusion += smoothstep(0.0, u_SSAOBias * 4.0, delta) * rangeCheck;
    }

    return clamp(1.0 - (occlusion / float(SSAO_SAMPLES)) * u_SSAOIntensity, 0.0, 1.0);
}

// ═══════════════════════════════════════════════════════════════════
//  Exponential Fog
// ═══════════════════════════════════════════════════════════════════

vec3 ApplyFog(vec3 color, float linearDepth) {
    float fogFactor = 1.0 - exp(-u_FogDensity * linearDepth);
    return mix(color, u_FogColor, clamp(fogFactor, 0.0, 1.0));
}

// ═══════════════════════════════════════════════════════════════════
//  ACES Filmic Tone Mapping
// ═══════════════════════════════════════════════════════════════════

vec3 ACESFilm(vec3 x) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// ═══════════════════════════════════════════════════════════════════
//  Color Grading
// ═══════════════════════════════════════════════════════════════════

vec3 ApplyColorGrading(vec3 color) {
    // Exposure
    color *= u_Exposure;

    // Contrast (pivot around mid-gray 0.18)
    color = mix(vec3(0.18), color, u_Contrast);

    // Brightness (additive)
    color += vec3(u_Brightness);

    // Saturation
    float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
    color = mix(vec3(luma), color, u_Saturation);

    return color;
}

// ═══════════════════════════════════════════════════════════════════
//  Vignette
// ═══════════════════════════════════════════════════════════════════

vec3 ApplyVignette(vec3 color, vec2 uv) {
    vec2  center = uv - 0.5;
    float dist   = dot(center, center);
    float vignette = 1.0 - dist * u_VignetteIntensity * 2.0;
    return color * clamp(vignette, 0.0, 1.0);
}

// ═══════════════════════════════════════════════════════════════════
//  Main
// ═══════════════════════════════════════════════════════════════════

void main() {
    float rawDepth   = texture(u_DepthTexture, v_TexCoords).r;
    float linearDepth = LinearizeDepth(rawDepth);

    // ── Debug visualization modes ───────────────────────────────
#if DEBUG_MODE == 1
    // Depth: remap to [0, 1] for visualization
    float d = linearDepth / u_Far;
    FragColor = vec4(vec3(d), 1.0);
#elif DEBUG_MODE == 2
    // Normals: remap [-1,1] to [0,1] for visualization
    vec3 n = ReconstructNormal(v_TexCoords) * 0.5 + 0.5;
    FragColor = vec4(n, 1.0);
#elif DEBUG_MODE == 3
    // SSAO only
    float ao = ComputeSSAO(v_TexCoords, linearDepth);
    FragColor = vec4(vec3(ao), 1.0);
#else

    // ── Normal rendering pipeline ───────────────────────────────
    vec3 color = texture(u_ScreenTexture, v_TexCoords).rgb;

    // 1. SSAO
    if (u_SSAOEnabled) {
        float ao = ComputeSSAO(v_TexCoords, linearDepth);
        color *= ao;
    }

    // 2. Fog
    if (u_FogEnabled) {
        color = ApplyFog(color, linearDepth);
    }

    // 3. Color grading (exposure, contrast, brightness, saturation)
    color = ApplyColorGrading(color);

    // 4. Tone mapping (HDR → LDR)
    if (u_ToneMappingEnabled) {
        color = ACESFilm(color);
    }

    // 5. Gamma correction (linear → sRGB)
    color = pow(color, vec3(1.0 / 2.2));

    // 6. Vignette (applied after gamma for perceptual correctness)
    if (u_VignetteEnabled) {
        color = ApplyVignette(color, v_TexCoords);
    }

    FragColor = vec4(color, 1.0);
#endif
}
