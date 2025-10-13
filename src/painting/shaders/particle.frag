#version 300 es
precision highp float;

layout(location = 0) out float out_density;
layout(location = 1) out vec4 out_advection;

void main() {
    vec2 u = gl_PointCoord - vec2(0.5, 0.5);

    //poly6 kernel
    float h_squared = 0.5 * 0.5;
    float r_squared = min(h_squared, dot(u, u));
    float poly6 = pow(h_squared - r_squared, 3.0);

    //linear kernel
    float linear = (1.0 / 16.0) * max(0.0, 0.5 - length(u));

    vec2 delta = vec2(1.0, 1.0);
    out_density = linear;
    out_advection = vec4(poly6 * delta, poly6, 0.0);
}