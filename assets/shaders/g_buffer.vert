#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 texCoords;

out vec3 FragPos;
out vec2 TexCoords;
out vec3 Normal;

uniform mat4 mvp;
uniform mat4 model;
uniform mat4 normal_mat;

void main() {
    vec4 worldPos = model * vec4(position, 1.0f);
    FragPos = worldPos.xyz;
    gl_Position = mvp * vec4(position, 1.0f);
    TexCoords = texCoords;

    Normal = normalize(mat3(normal_mat) * normal);
}