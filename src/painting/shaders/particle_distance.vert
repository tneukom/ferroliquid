#version 300 es

in vec2 in_simulation_position;

uniform mat3 device_from_simulation;
uniform float point_size;

void main() {
    vec2 device_position = (device_from_simulation * vec3(in_simulation_position, 1.0)).xy;
    gl_PointSize = point_size;
    gl_Position = vec4(device_position, 0.0, 1.0);
}