#version 450 core

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inTangent;
layout (location = 2) in vec3 inNormal;
layout (location = 3) in vec2 inUv;

layout (location = 0) out vec3 outWorldPos;
layout (location = 1) out vec3 outNormal;
layout (location = 2) out vec2 o_uv;

layout (binding = 0) uniform UBO
{
	mat4 projection;
	mat4 view;
	mat4 model;
} ubo;

void main() {
    vec4 WorldPos = ubo.model * vec4(inPosition, 1.0);
    outWorldPos = WorldPos.xyz;
    gl_Position = ubo.projection * ubo.view * WorldPos;

    o_uv = inUv;
    o_uv.t = 1.0 - o_uv.t;

    // GL to Vulkan coord space
    outWorldPos.y = -outWorldPos.y;

    // Normal in world space
    mat3 mNormal = transpose(inverse(mat3(ubo.model)));
    outNormal = mNormal * normalize(inNormal);
}