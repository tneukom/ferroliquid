#version 300 es
precision highp float;

out vec4 out_color;

void main() {
    vec2 u = gl_PointCoord - vec2(0.5, 0.5);

    //poly6 kernel
    float h_squared = 0.5 * 0.5;
    float r_squared = min(h_squared, dot(u, u));
    float poly6 = pow(h_squared - r_squared, 3.0);

    //linear kernel
    float linear = 1.0 / 8.0 * (0.5 - length(u));

    out_color = vec4(poly6, linear, 0.0, 1.0);
}