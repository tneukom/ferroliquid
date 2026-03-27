{{version}}

in vec2 in_device_position;
in vec2 in_uv;

out vec2 pass_uv;

void main() {
    pass_uv = in_uv;
    gl_Position = vec4(in_device_position, 0.0, 1.0);
}