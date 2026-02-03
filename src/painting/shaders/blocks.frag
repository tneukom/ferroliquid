#version 300 es
precision mediump float;

in highp vec2 pass_uv;
in highp vec4 pass_pen_color;
in highp vec4 pass_brush_color;

uniform sampler2D brush_texture;
uniform sampler2D pen_texture;

uniform int mode;

out vec4 out_color;

#define MODE_BACKGROUND_BRUSH 0
#define MODE_PEN 1
#define MODE_FOREGROUND_BRUSH 2

void main() {
    if(mode == MODE_BACKGROUND_BRUSH) {
        vec4 brush_texture_color = texture(brush_texture, pass_uv);
        vec4 brush_color = vec4(pass_brush_color.rgb, pass_brush_color.a * brush_texture_color.a);
        out_color = brush_color;
    } else if(mode == MODE_PEN) {
        vec4 background_color = vec4(1.0, 1.0, 1.0, 1.0);
        vec4 pen_texture_color = texture(pen_texture, pass_uv);
        vec4 pen_color = vec4(pen_texture_color.r * pass_pen_color.rgb + pen_texture_color.g * background_color.rgb, pen_texture_color.a);
        out_color = pen_color;
    } else {
        // MODE_FOREGROUND_BRUSH
        vec4 pen_texture_color = texture(pen_texture, pass_uv);
        vec4 brush_texture_color = texture(brush_texture, pass_uv);
        vec4 brush_color = vec4(pass_brush_color.rgb, pass_brush_color.a * brush_texture_color.a * pen_texture_color.a);
        out_color = brush_color;
    }
}