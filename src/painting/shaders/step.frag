#version 300 es
precision highp float;

in vec2 pass_uv;

uniform sampler2D sampler;
uniform float edge;

out float out_density;

void main() {
    float density = texture(sampler, pass_uv).r;
    out_density = step(edge, density);
}