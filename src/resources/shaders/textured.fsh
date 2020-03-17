#version 120

uniform sampler2D tex;

varying vec2 f_texture_uv;
varying vec3 f_normal;
varying vec4 f_color;

void main() {
    if (f_color.a == 0.0) discard;
    gl_FragColor = texture2D(tex, f_texture_uv) * f_color;
}