#version 410 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 a_tex_coords;
layout(location = 7) in vec3 a_normal;
layout(location = 2) in mat4 model;
layout(location = 6) in vec4 a_tint;

out vec2 v_tex_coords;
out vec4 v_tint;
out vec3 v_world_normal;
out vec3 v_world_pos;

uniform mat4 u_ViewProjection;

void main() {
    v_tex_coords = a_tex_coords;
    v_tint = a_tint;

    vec4 worldPos = model * vec4(position, 1.0);
    v_world_pos = worldPos.xyz;

    // Normal matrix: mat3(model) works for uniform scale.
    // For non-uniform scale, use transpose(inverse(mat3(model))).
    mat3 normalMatrix = mat3(model);
    v_world_normal = normalize(normalMatrix * a_normal);

    gl_Position = u_ViewProjection * worldPos;
}
