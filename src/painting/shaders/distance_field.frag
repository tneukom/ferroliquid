{{version}}
precision mediump float;

in highp vec2 pass_uv;

uniform sampler2D signed_distance_sampler;

out vec4 out_color;

void main() {
    highp float signed_distance = texture(signed_distance_sampler, pass_uv).r;

    // Outline width in distance field coordinates
    const float OUTLINE_WIDTH = 0.8;
    float outline_alpha = 1.0 - smoothstep(OUTLINE_WIDTH - 0.1, OUTLINE_WIDTH + 0.1, abs(signed_distance));
    vec3 outline_color = vec3(0.0);
    // 1 where signed_distance < 0 and 0 otherwise
    float interior_alpha = step(0.0, -signed_distance);
    vec3 interior_color = vec3(0.8);

    // Draw outline over interior
    float alpha = 1.0 - (1.0 - outline_alpha) * (1.0 - interior_alpha);
    vec3 color = outline_alpha * outline_color + (1.0 - outline_alpha) * interior_alpha * interior_color;

    // Premultiplied alpha blending
    out_color = vec4(color, alpha);
}