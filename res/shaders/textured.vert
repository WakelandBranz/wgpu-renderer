#version 450

layout(location=0) in vec2 aPosition;
layout(location=1) in vec4 aColor;

layout(location=0) out vec4 vColor;

void main() {
    // Convert from screen space (0-800, 0-600) to NDC (-1 to 1)
    float ndc_x = (aPosition.x / 800.0) * 2.0 - 1.0;
    float ndc_y = 1.0 - (aPosition.y / 600.0) * 2.0;
    gl_Position = vec4(ndc_x, ndc_y, 0.0, 1.0);
    vColor = aColor;
}