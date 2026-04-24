#version 410 core

in vec2 v_tex_coords;
in vec4 v_color;
out vec4 frag_color;

uniform sampler2D u_Texture;
uniform bool u_HasTexture;

void main() {
    if (u_HasTexture) {
        frag_color = texture(u_Texture, v_tex_coords) * v_color;
    } else {
        frag_color = v_color;
    }
}
