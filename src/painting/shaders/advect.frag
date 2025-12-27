#version 300 es
precision highp float;

in vec2 pass_uv;

uniform sampler2D advect_texture;
uniform sampler2D color_texture;

uniform mat2 uv_from_simulation;

out vec4 out_color;

void main() {
    vec4 advection = texture(advect_texture, pass_uv);

    if (advection.b > 0.0) {
        vec2 simulation_delta = advection.rg / advection.b;
        vec2 uv_delta = uv_from_simulation * simulation_delta;
        vec2 previous_uv = pass_uv - uv_delta;
        out_color = texture(color_texture, previous_uv);
    } else {
        out_color = vec4(0.0, 0.0, 0.0, 1.0);
    }
}