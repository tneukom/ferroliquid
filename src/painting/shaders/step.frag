#version 300 es
precision highp float;

in vec2 pass_uv;

uniform mediump sampler2D texture;
uniform mediump float edge;

out float out_density;

void main() {
    float density = texture2D(texture, pass_uv);
    out_density = step(edge, density);
}