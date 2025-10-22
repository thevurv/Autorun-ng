#version 100

// In
varying lowp vec2 uv;
varying lowp vec4 color;

// Uniforms
uniform lowp mat4 projMatrix;
// uniform lowp sampler2D textureSampler;

void main() {
    gl_FragColor = color;
}
