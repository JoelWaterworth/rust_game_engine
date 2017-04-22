#version 450 core


#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec2 o_uv;

void main()
{
    gl_Position = vec4(position, 1.0f);
    o_uv = uv;
}