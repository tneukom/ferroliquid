{{version}}

in vec2 in_position;
in vec2 in_bitmap_position;
in vec4 in_brush_color;
in vec4 in_pen_color;

uniform mat3 device_from;
uniform mat3 uv_from_bitmap;

out vec2 pass_uv;
out vec4 pass_brush_color;
out vec4 pass_pen_color;

void main() {
    pass_uv = (uv_from_bitmap * vec3(in_bitmap_position, 1.0)).xy;
    pass_brush_color = in_brush_color;
    pass_pen_color = in_pen_color;
    vec2 device_position = (device_from * vec3(in_position, 1.0)).xy;
    gl_Position = vec4(device_position, 0.0, 1.0);
}