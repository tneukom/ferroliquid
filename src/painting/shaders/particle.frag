{{version}}
precision mediump float;

out vec4 out_advection;

in highp vec2 pass_simulation_particle_delta_position;
in highp vec2 pass_point_uv;

void main() {
    float r = length(pass_point_uv);
    float linear = max(0.0, 1.0 - r);
    float k = sqrt(linear);
    out_advection = vec4(k * pass_simulation_particle_delta_position, k, 0.0);
}