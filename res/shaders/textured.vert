#version 450

layout(binding = 0) uniform ScreenSize {
    vec2 screen_size;
};

layout(location = 0) in vec2 aPosition;
layout(location = 1) in vec4 aColor;

layout(location = 0) out vec4 vColor;

void main() {
    float ndc_x = (aPosition.x / screen_size.x) * 2.0 - 1.0;
    float ndc_y = 1.0 - (aPosition.y / screen_size.y) * 2.0;
    
    gl_Position = vec4(ndc_x, ndc_y, 0.0, 1.0);
    vColor = aColor;
}