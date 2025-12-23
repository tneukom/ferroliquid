#version 300 es
precision highp float;

in vec2 in_simulation_position;
in vec4 in_color;

uniform mat3 device_from_simulation;

out vec4 pass_color;

void main() {
    pass_color = in_color;
    vec2 device_position = (device_from_simulation * vec3(in_simulation_position, 1.0)).xy;
    gl_Position = vec4(device_position, 0.0, 1.0);
}