#version 450 core

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (binding = 1) uniform sampler2D dTexture;
layout (binding = 2) uniform sampler2D noramlmap;

layout (location = 0) in vec3 outWorldPos;
layout (location = 1) in vec3 outNormal;
layout (location = 2) in vec3 outTangent;
layout (location = 3) in vec2 o_uv;

layout (location = 0) out vec4 gPosition;
layout (location = 1) out vec4 gNormal;
layout (location = 2) out vec4 gcolor;

void main(){
    gPosition   = vec4(outWorldPos,1.0);

// Calculate normal in tangent space
	vec3 N = normalize(outNormal);
	N.y = -N.y;
	vec3 T = normalize(outTangent);
	vec3 B = cross(N, T);
	mat3 TBN = mat3(T, B, N);
	vec3 tnorm = TBN * normalize(texture(noramlmap, o_uv).xyz * 2.0 - vec3(1.0));
	gNormal     = vec4(tnorm, 1.0);

    gcolor      = texture(dTexture, o_uv);
}