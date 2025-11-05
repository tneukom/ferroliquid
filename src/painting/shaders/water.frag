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

void main() {
    vec4 color = texture2D(color_texture, pass_uv);
    vec4 density = texture2D(density_texture, pass_uv);
    // vec4 bg = texture2D(texture_background, pass_uv);

    //float darkness = -8.0 * density.r * density.r + 16.0 * density.r - 7.0;

    // vec4 color = vec4(1.0, 0.0, 0.0, 1.0);
    // vec4 bg = vec4(0.0, 0.0, 0.0, 1.0);

    float alpha = smoothstep(edge_low, edge_high, density.r);
    float darken = 0.5 + 0.5 * smoothstep(darken_edge_low, darken_edge_high, density.r);
    gl_FragColor = vec4(darken * color.rgb, alpha);
}