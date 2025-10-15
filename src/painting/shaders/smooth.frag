#version 300 es
precision highp float;

in vec2 pass_uv;

#define KERNEL_MAX_SIZE 32

uniform sampler2D density_texture;
uniform float kernel[KERNEL_MAX_SIZE];
uniform vec2 uv_offsets[KERNEL_MAX_SIZE];
uniform int kernel_size;

out float out_density;

void main() {
    float sum = 0.0;
    for (int i = 0; i < kernel_size; ++i) {
        float density_sample = texture(density_texture, pass_uv + uv_offsets[i]);
        sum += kernel[i] * density_sample;
    }

    out_density = sum;
}