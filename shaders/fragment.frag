#version 410 core

in vec2 v_tex_coords;
in vec4 v_tint;
in vec3 v_world_normal;
in vec3 v_world_pos;

// G-Buffer MRT outputs
layout(location = 0) out vec4 g_albedo;
layout(location = 1) out vec4 g_normal;

uniform sampler2D u_Texture;
uniform bool u_HasTexture;
uniform vec4 u_AlbedoColor;

void main() {
    if (u_HasTexture) {
        g_albedo = texture(u_Texture, v_tex_coords) * v_tint * u_AlbedoColor;
    } else {
        g_albedo = v_tint * u_AlbedoColor;
    }

    // RT1: World-space normal (packed into RGB, w=1.0)
    g_normal = vec4(normalize(v_world_normal), 1.0);
}
