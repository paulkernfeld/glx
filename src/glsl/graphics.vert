#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 vertexColor;
layout(location = 2) in float z;
layout(location = 0) out vec4 fragmentColor;

void main() {
    // pos is (x, y) and from (-1, -1) to (1, 1)
    // This works correctly b/c we're rendering to a square
    // As is standard, lower z is rendered atop higher z
    gl_Position = vec4(pos, z, 1.0);
    fragmentColor = vertexColor;
}
