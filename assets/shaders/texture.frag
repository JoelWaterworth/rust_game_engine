#version 450 core

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

//layout (binding = 0) uniform sampler2D samplerColor;

layout (location = 0) in vec3 outWorldPos;
layout (location = 1) in vec3 outNormal;
layout (location = 2) in vec2 o_uv;

layout (location = 0) out vec4 gPosition;
layout (location = 1) out vec4 gNormal;
layout (location = 2) out vec4 gcolor;

void main(){
    //vec4 color = texture(samplerColor, o_uv);
    gPosition = vec4(outWorldPos,1.0);
    gNormal = vec4(outNormal,1.0);
    gcolor = vec4(0.0,0.5,0.0,1.0);
}