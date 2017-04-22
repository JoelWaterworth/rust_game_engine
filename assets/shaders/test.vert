#version 330 core
layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

out vec3 Normal;
out vec2 Tex_coord;

void main() {
    Normal = normal;
    Tex_coord = tex_coord;
	gl_Position = vec4(vec3(position), 1.0);
}
