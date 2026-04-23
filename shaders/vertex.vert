#version 410
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 a_vertex_color;
layout (location = 2) in mat4 model;
layout (location = 6) in vec3 a_instance_color;

out vec3 v_color;

uniform mat4 u_ViewProjection;

void main() {
    v_color = a_instance_color;
    gl_Position = u_ViewProjection * model * vec4(position, 1.0);
}
