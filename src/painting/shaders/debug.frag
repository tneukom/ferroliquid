{{version}}
precision mediump float;

in highp vec2 pass_uv;

uniform sampler2D sampler;
uniform int style;

out vec4 out_color;

const int STYLE_DEFAULT = 0;
const int STYLE_ADVECTION = 1;

void main() {
    vec4 color = texture(sampler, pass_uv);
    if (style == STYLE_DEFAULT) {
        out_color = color;
    } else if (style == STYLE_ADVECTION) {
        out_color = vec4(0.5 * color.rg / color.b + 0.5, 0.0, 1.0);
    } else {
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
}