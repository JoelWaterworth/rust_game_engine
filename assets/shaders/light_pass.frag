#version 330 core

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (binding = 1) uniform sampler2D gPosition;
layout (binding = 2) uniform sampler2D gNormal;
layout (binding = 3) uniform sampler2D gAlbedoSpec;

struct Light {
	vec3 position;
	vec3 color;
	float radius;
};

layout (binding = 4) uniform UBO {
    Light lights[6];
    vec3 viewPos;
} ubo;

layout (location = 0) in vec2 inUV;

layout (location = 0) out vec4 outFragcolor;

void main() {
//    // Retrieve data from gbuffer
    vec3 fragPos = texture(gPosition, inUV).rgb;
    vec3 normal  = texture(gNormal, inUV).rgb;
    vec3 Diffuse = texture(gAlbedoSpec, inUV).rgb;
    float Specular = texture(gAlbedoSpec, inUV).a;

    #define lightCount 6
    #define ambient 0.0
    #define linear 0.7
    #define quadratic 1.8

    // Then calculate lighting as usual
    vec3 lighting  = Diffuse * 0.1; // hard-coded ambient component
    vec3 viewDir  = normalize(ubo.viewPos - fragPos);
    for(int i = 0; i < lightCount; ++i) {
        // Diffuse
        vec3 lightDir = normalize(ubo.lights[i].position - fragPos);
        vec3 diffuse = max(dot(normal, lightDir), 0.0) * Diffuse * ubo.lights[i].color;
        // Specular
        vec3 halfwayDir = normalize(lightDir + viewDir);
        float spec = pow(max(dot(normal, halfwayDir), 0.0), 16.0);
        vec3 specular = ubo.lights[i].color * spec * Specular;
        // Attenuation
        float distance = length(ubo.lights[i].position - fragPos);
        float attenuation = 1.0 / (1.0 + linear * distance + quadratic * distance * distance);
        diffuse *= attenuation;
        specular *= attenuation;
        lighting += diffuse + specular;
    }
    outFragcolor = vec4(lighting, 1.0);
}