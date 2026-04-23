#version 410

in vec2 v_tex_coords;
out vec4 frag_color;

uniform sampler2D u_Texture;

void main() {
    frag_color = texture(u_Texture, v_tex_coords);
}
