#version 300 es
precision highp float;

in vec2 pass_uv;

uniform sampler2D texture;

out vec4 out_color;

void main() {
    out_color = texture2D(texture, pass_uv);
}