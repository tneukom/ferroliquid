#version 300 es
precision mediump float;

out float out_distance;

void main() {
    // u is in range [-0.5, 0.5]^2
    vec2 u = gl_PointCoord - vec2(0.5, 0.5);
    float r = 2.0 * length(u);
    out_distance = max(0.0, 1.0 - r);
}