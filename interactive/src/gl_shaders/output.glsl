#version 450

layout (binding = 0) uniform sampler2D tex;

in vec2 uv;

out vec4 FragColor;

void main() {
    vec2 uv_centered = vec2(uv.x, - uv.y + 1.0) - 0.5;

    vec2 uv_r = (uv_centered * 1.001) + 0.5;
    vec2 uv_g = (uv_centered) + 0.5;
    vec2 uv_b = (uv_centered * 0.999) + 0.5;

    vec3 t = vec3(texture(tex, uv_g).r, texture(tex, uv_g).g, texture(tex, uv_g).b);

    float luminance = dot(t.rgb, vec3(0.2126, 0.7152, 0.0722));

    float new_luminance = luminance / (luminance + 1.0);

    vec3 tonemapped = t.rgb * (new_luminance / luminance);

    float gamma = 1.0 / 2.2;
    vec3 srgb = pow(tonemapped, vec3(gamma));

    FragColor = vec4(srgb, 1.0);
}
