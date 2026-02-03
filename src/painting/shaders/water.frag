#version 300 es
precision mediump float;

in highp vec2 pass_uv;

uniform sampler2D color_sampler;
uniform sampler2D density_sampler;

uniform float edge_low;
uniform float edge_high;

uniform float darken_edge_low;
uniform float darken_edge_high;

out vec4 out_color;

vec3 linear_to_srgb(vec3 linear) {
    bvec3 cutoff = lessThanEqual(linear, vec3(0.0031308));
    vec3 higher = vec3(1.055) * pow(linear, vec3(1.0 / 2.4)) - vec3(0.055);
    vec3 lower = linear * vec3(12.92);

    return mix(higher, lower, vec3(cutoff));
}

void main() {
    vec4 color = texture(color_sampler, pass_uv);
    vec4 density = texture(density_sampler, pass_uv);

    float alpha = smoothstep(edge_low, edge_high, density.r);
    float darken = 0.5 + 0.5 * smoothstep(darken_edge_low, darken_edge_high, density.r);
    vec3 linear_rgb = darken * color.rgb;

    // output sRGB
    out_color = vec4(linear_to_srgb(linear_rgb), 0.8 * alpha);

    // output linear RGB
    // out_color = vec4(linear_rgb, alpha);
}