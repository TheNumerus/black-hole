#version 450

layout (binding = 0) uniform sampler2D tex;

in vec2 uv;

out vec4 FragColor;

void main() {
    FragColor = texture(tex, uv);
}
