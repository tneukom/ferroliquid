#version 300 es
precision mediump float;

in highp vec2 pass_point_uv;

out float out_distance;

void main() {
    float r = length(pass_point_uv);
    out_distance = max(0.0, 1.0 - r);
}