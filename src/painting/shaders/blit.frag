{{version}}
precision mediump float;

in highp vec2 pass_uv;

uniform sampler2D sampler;

out vec4 out_color;

void main() {
    out_color = texture(sampler, pass_uv);
}