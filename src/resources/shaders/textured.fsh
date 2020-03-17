#version 120

uniform sampler2D tex;

attribute vec2 f_texture_uv;
attribute vec3 f_normal;
attribute vec4 f_color;

varying vec4 out_color;

void main() {
    if (f_color.a == 0.0) discard;
    out_color = texture2D(tex, f_texture_uv) * f_color;
}