#version 410

layout(location = 2) in mat4 model;
layout(location = 0) in vec3 position;
layout(location = 1) in vec2 a_tex_coords;

out vec2 v_tex_coords;

uniform mat4 u_ViewProjection;

void main() {
    v_tex_coords = a_tex_coords;
    gl_Position = u_ViewProjection * model * vec4(position, 1.0);
}
