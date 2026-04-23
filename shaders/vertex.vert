#version 410
layout (location = 0) in vec2 position;
layout (location = 1) in vec3 color;

out vec3 v_color;

uniform mat4 u_ViewProjection;

void main() {
    v_color = color;
    gl_Position = u_ViewProjection * vec4(position, 0.0, 1.0);
}
