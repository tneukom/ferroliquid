{{version}}

layout(location = 0) in vec2 in_simulation_quad_offset;
layout(location = 1) in vec2 in_simulation_particle_position;
layout(location = 2) in vec2 in_simulation_particle_previous_position;

uniform mat3 device_from_simulation;
uniform float simulation_radius;

out vec2 pass_simulation_particle_delta_position;
out vec2 pass_point_uv;

void main() {
    pass_simulation_particle_delta_position = in_simulation_particle_position - in_simulation_particle_previous_position;

    pass_point_uv = in_simulation_quad_offset;

    vec2 simulation_position = in_simulation_particle_position + simulation_radius * in_simulation_quad_offset;
    vec2 device_position = (device_from_simulation * vec3(simulation_position, 1.0)).xy;
    gl_Position = vec4(device_position, 0.0, 1.0);
}