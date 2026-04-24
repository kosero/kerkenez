#version 410 core

in vec2 v_tex_coords;
in vec4 v_color;
in vec3 v_world_normal;
in vec3 v_world_pos;

// G-Buffer MRT outputs
layout(location = 0) out vec4 g_albedo;
layout(location = 1) out vec4 g_normal;

uniform sampler2D u_Texture;
uniform bool u_HasTexture;

void main() {
    // RT0: Albedo (color/texture)
    if (u_HasTexture) {
        g_albedo = texture(u_Texture, v_tex_coords) * v_color;
    } else {
        g_albedo = v_color;
    }

    // RT1: World-space normal (packed into RGB, w=1.0)
    g_normal = vec4(normalize(v_world_normal), 1.0);
}
