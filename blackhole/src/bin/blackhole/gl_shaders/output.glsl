#version 450

layout (binding = 0) uniform sampler2D tex;

in vec2 uv;

out vec4 FragColor;

void main() {
    vec4 t = texture(tex, uv);

    float luminance = dot(t.rgb, vec3(0.2126, 0.7152, 0.0722));

    float new_luminance = luminance / (luminance + 1.0);

    vec3 tonemapped = t.rgb * (new_luminance / luminance);

    float gamma = 1.0 / 2.2;
    vec3 srgb = pow(tonemapped, vec3(gamma));

    FragColor = vec4(srgb, 1.0);
}
