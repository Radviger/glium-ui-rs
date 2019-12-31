#version 140

in vec2 f_texture_uv;
out vec4 out_color;

uniform vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
uniform sampler2D tex;

void main() {
    vec4 c = vec4(color.rgb, color.a * texture(tex, f_texture_uv).r);
    if (c.a <= 0.01) {
        discard;
    } else {
        out_color = c;
    }
}