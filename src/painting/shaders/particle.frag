#version 300 es
precision highp float;

layout (location = 0) out float out_density;
layout (location = 1) out vec4 out_advection;

in vec2 pass_simulation_delta_position;

void main() {
    vec2 u = gl_PointCoord - vec2(0.5, 0.5);

    // poly6 kernel for advection
    // float h_squared = 0.5 * 0.5;
    // float r_squared = min(h_squared, dot(u, u));
    // float poly6 = pow(h_squared - r_squared, 3.0);
    // out_advection = vec4(poly6 * pass_simulation_delta_position, poly6, 0.0);

    // linear kernel for advection
    float linear = max(0.0, 0.5 - length(u));
    out_advection = vec4(linear * pass_simulation_delta_position, linear, 0.0);

    // linear kernel for density
    out_density = max(0.0, 0.5 - length(u));
}