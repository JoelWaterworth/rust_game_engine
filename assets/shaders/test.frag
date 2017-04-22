#version 330 core

out vec4 frag_color;
in vec3 Normal;
in vec2 Tex_coord;

void main() {
	frag_color = vec4(((Normal / 2) + 0.5) + vec3(Tex_coord, 0), 1.0);
}
