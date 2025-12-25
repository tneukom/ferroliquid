#version 300 es
precision highp float;

in vec2 pass_simulation_position;

uniform sampler2D noise_texture;
uniform vec4 color_a;
uniform vec4 color_b;

uniform mat3 uv_from_simulation;

out vec4 out_color;

void main() {
    vec2 uv = (uv_from_simulation * vec3(pass_simulation_position, 1.0)).xy;
    float pattern = texture(noise_texture, uv).r;
    out_color = mix(color_a, color_b, pattern);
}