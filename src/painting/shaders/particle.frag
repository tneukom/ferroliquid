#version 300 es
precision mediump float;

out vec4 out_advection;

in highp vec2 pass_simulation_delta_position;

void main() {
    vec2 u = gl_PointCoord - vec2(0.5, 0.5);

    // linear kernel
    float r = length(u);
    float linear = max(0.0, 0.5 - r);

    // float k = 1.0 - step(0.5, r);
    float k = sqrt(linear);
    out_advection = vec4(k * pass_simulation_delta_position, k, 0.0);
}