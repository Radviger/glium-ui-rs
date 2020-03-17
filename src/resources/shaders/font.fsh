#version 120

varying vec2 f_texture_uv;

uniform vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
uniform sampler2D tex;

void main() {
    vec4 c = vec4(color.rgb, color.a * texture2D(tex, f_texture_uv).r);
    if (c.a <= 0.01) {
        discard;
    } else {
        gl_FragColor = c;
    }
}