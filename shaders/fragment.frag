#version 410

in vec2 v_tex_coords;
in vec3 v_color;
out vec4 frag_color;

uniform sampler2D u_Texture;
uniform bool u_HasTexture;

void main() {
    if (u_HasTexture) {
        frag_color = texture(u_Texture, v_tex_coords) * vec4(v_color, 1.0);
    } else {
        frag_color = vec4(v_color, 1.0);
    }
}
