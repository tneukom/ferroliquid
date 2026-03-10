#version 300 es
precision mediump float;

in highp vec2 pass_uv;

uniform sampler2D signed_distance_sampler;

out vec4 out_color;

void main() {
    highp float signed_distance = texture(signed_distance_sampler, pass_uv).r;
    highp float alpha = 1.0 - smoothstep(-0.25, 0.25, signed_distance);
    out_color = vec4(0.3, 0.3, 0.3, alpha);
}