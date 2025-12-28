#version 300 es
precision highp float;

in vec2 pass_uv;

uniform sampler2D color_texture;
uniform sampler2D density_texture;
// uniform sampler2D texture_background;

uniform float edge_low;
uniform float edge_high;

uniform float darken_edge_low;
uniform float darken_edge_high;

vec3 linear_to_srgb(vec3 linear) {
    bvec3 cutoff = lessThanEqual(linear, vec3(0.0031308));
    vec3 higher = vec3(1.055) * pow(linear, vec3(1.0 / 2.4)) - vec3(0.055);
    vec3 lower = linear * vec3(12.92);

    return mix(higher, lower, vec3(cutoff));
}

void main() {
    vec4 color = texture2D(color_texture, pass_uv);
    vec4 density = texture2D(density_texture, pass_uv);
    // vec4 bg = texture2D(texture_background, pass_uv);

    //float darkness = -8.0 * density.r * density.r + 16.0 * density.r - 7.0;

    // vec4 color = vec4(1.0, 0.0, 0.0, 1.0);
    // vec4 bg = vec4(0.0, 0.0, 0.0, 1.0);

    float alpha = smoothstep(edge_low, edge_high, density.r);
    float darken = 0.5 + 0.5 * smoothstep(darken_edge_low, darken_edge_high, density.r);
    vec3 linear_rgb = darken * color.rgb;

    // output sRGB
    gl_FragColor = vec4(linear_to_srgb(linear_rgb), alpha);

    // output linear RGB
    // gl_FragColor = vec4(linear_rgb, alpha);
}