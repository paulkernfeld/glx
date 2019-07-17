#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 vertexColor;
layout(location = 0) out vec4 fragmentColor;

void main() {
    // Hack: divide x by hard-coded screen aspect ratio
    gl_Position = vec4(pos[0] / 1.6, pos[1], 0.0, 1.0);
    fragmentColor = vertexColor;
}
