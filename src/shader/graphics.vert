#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 vertexColor;
layout(location = 0) out vec4 fragmentColor;

void main() {
    // This works correctly b/c we're rendering to a square
    gl_Position = vec4(pos, 0.0, 1.0);
    fragmentColor = vertexColor;
}
