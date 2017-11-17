#version 450 core

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (binding = 1) uniform sampler2D dTexture;
layout (binding = 2) uniform sampler2D sTexture;

layout (location = 0) in vec3 outWorldPos;
layout (location = 1) in vec3 outNormal;
layout (location = 2) in vec2 o_uv;

layout (location = 0) out vec4 gPosition;
layout (location = 1) out vec4 gNormal;
layout (location = 2) out vec4 gcolor;

void main(){
    vec4 diff        = texture(dTexture, o_uv);
    vec4 spec        = texture(sTexture, o_uv);
    gcolor      = vec4(diff.rgb, spec.r);
    gPosition   = vec4(outWorldPos,1.0);
    gNormal     = vec4(outNormal,1.0);
}