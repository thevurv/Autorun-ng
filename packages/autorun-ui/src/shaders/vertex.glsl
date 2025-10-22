#version 100

// In
attribute vec2 inPos;
attribute vec2 inUV;
attribute vec4 inColor;

// Out
varying lowp vec2 uv;
varying lowp vec4 color;

// Uniforms
uniform lowp int hasTexture;
uniform lowp mat4 projMatrix;
uniform lowp sampler2D textureSampler;

void main() {
    gl_Position = projMatrix * vec4(inPos, 0, 1);

    uv = inUV;
    color = inColor;
}
