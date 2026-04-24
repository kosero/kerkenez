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
uniform vec3 u_CameraPos;

// Lighting Structures
struct DirectionalLight {
    vec3 direction;
    vec3 color;
    float intensity;
    bool enabled;
};

struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
    float radius;
};

uniform vec3 u_AmbientColor;
uniform float u_AmbientIntensity;
uniform DirectionalLight u_DirLight;

#define MAX_POINT_LIGHTS 32
uniform int u_PointLightsCount;
uniform PointLight u_PointLights[MAX_POINT_LIGHTS];

// Debug
#ifndef DEBUG_MODE
#define DEBUG_MODE 0
#endif

// SSAO
uniform float u_SSAORadius;
uniform float u_SSAOIntensity;
uniform float u_SSAOBias;

// Fog
uniform float u_FogDensity;
uniform vec3 u_FogColor;

// Tone Mapping & Color Grading
uniform float u_Exposure;
uniform float u_Contrast;
uniform float u_Brightness;
uniform float u_Saturation;

// Vignette
uniform float u_VignetteIntensity;
uniform float u_VignetteRadius;
uniform float u_VignetteSoftness;

// Temporal variation
uniform float u_Time;

//  Utility Functions



//  G-Buffer Normal Fetch

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
    float dist = distance(uv, vec2(0.5));
    float vignette = smoothstep(u_VignetteRadius, u_VignetteRadius - u_VignetteSoftness, dist);
    return color * mix(1.0, vignette, u_VignetteIntensity);
}

//  Dithering (Banding / Şeritlenme Önleyici)
vec3 ApplyDithering(vec3 color, vec2 uv) {
    // Temporal variation: include u_Time in the hash to vary the pattern every frame
    float dither = fract(sin(dot(gl_FragCoord.xy + u_Time * 0.1, vec2(12.9898, 78.233))) * 43758.5453) / 255.0;
    return color + dither;
}

//  Main
void main() {
    float rawDepth = texture(u_DepthTexture, v_TexCoords).r;
    float linearDepth = LinearizeDepth(rawDepth, u_Near, u_Far);

    #if DEBUG_MODE == 1
    float d = linearDepth / u_Far;
    FragColor = vec4(vec3(d), 1.0);
    #elif DEBUG_MODE == 2
    vec3 n = GetNormal(v_TexCoords) * 0.5 + 0.5;
    FragColor = vec4(n, 1.0);
    #elif DEBUG_MODE == 3
    float ao = texture(u_SSAOTexture, v_TexCoords).r;
    FragColor = vec4(vec3(ao), 1.0);
    #else

    vec3 albedo = texture(u_ScreenTexture, v_TexCoords).rgb;
    vec3 normal = GetNormal(v_TexCoords);
    vec3 fragPos = WorldPosFromDepth(v_TexCoords, rawDepth, u_InverseVP);
    vec3 viewDir = normalize(u_CameraPos - fragPos);

    // Hardcoded for now (could come from G-Buffer RT2 later)
    float roughness = 0.5;
    float shininess = 32.0;
    float specularStrength = 0.2;

    // SSAO
    float ao_final = 1.0;
    #ifdef ENABLE_SSAO
        float ao = texture(u_SSAOTexture, v_TexCoords).r;
        ao = clamp(ao, 0.0, 1.0);
        float mask = GetScreenEdgeFade(v_TexCoords);
        ao_final = 1.0 - ((1.0 - ao) * mask);
    #endif

    // 1. Ambient Lighting
    vec3 ambient = u_AmbientColor * u_AmbientIntensity * albedo;

    vec3 lighting = ambient;

    // Background mask (don't light the sky/background)
    if (rawDepth < 1.0) {
        // 2. Directional Light
        if (u_DirLight.enabled) {
            vec3 lightDir = normalize(-u_DirLight.direction);
            float diff = max(dot(normal, lightDir), 0.0);
            vec3 diffuse = u_DirLight.color * u_DirLight.intensity * diff * albedo;
            
            vec3 halfwayDir = normalize(lightDir + viewDir);  
            float spec = pow(max(dot(normal, halfwayDir), 0.0), shininess);
            vec3 specular = u_DirLight.color * u_DirLight.intensity * spec * specularStrength;
            
            lighting += diffuse + specular;
        }

        // 3. Point Lights
        for (int i = 0; i < u_PointLightsCount; i++) {
            PointLight light = u_PointLights[i];
            
            vec3 lightDir = normalize(light.position - fragPos);
            float distance = length(light.position - fragPos);
            
            // Attenuation (smooth step falloff)
            float attenuation = clamp(1.0 - (distance * distance) / (light.radius * light.radius), 0.0, 1.0);
            attenuation *= attenuation;
            
            if (attenuation > 0.0) {
                float diff = max(dot(normal, lightDir), 0.0);
                vec3 diffuse = light.color * light.intensity * diff * albedo * attenuation;
                
                vec3 halfwayDir = normalize(lightDir + viewDir);  
                float spec = pow(max(dot(normal, halfwayDir), 0.0), shininess);
                vec3 specular = light.color * light.intensity * spec * specularStrength * attenuation;
                
                lighting += diffuse + specular;
            }
        }
    } else {
        // Just show albedo for background
        lighting = albedo;
    }

    // Apply SSAO to the final lighting
    lighting *= ao_final;

    // 1. Exposure (on Linear HDR)
    vec3 color = lighting * u_Exposure;

    // 2. Fog (on Linear HDR)
    #ifdef ENABLE_FOG
        color = ApplyFog(color, linearDepth);
    #endif

    // 3. Tone Mapping (Linear to LDR)
    #ifdef ENABLE_TONEMAP
        color = ACESFilm(color);
    #endif

    // 4. Gamma Correction (LDR to sRGB)
    color = pow(max(color, vec3(0.0)), vec3(1.0 / 2.2));

    // 5. Color Grading (in sRGB space)
    color = ApplyColorGrading(color);

    // 6. Vignette (in sRGB space)
    #ifdef ENABLE_VIGNETTE
        color = ApplyVignette(color, v_TexCoords);
    #endif

    // 7. Dithering (Final step)
    color = ApplyDithering(color, v_TexCoords);

    FragColor = vec4(color, 1.0);
    #endif
}
