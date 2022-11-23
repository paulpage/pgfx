#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec4 color;

uniform mat4 world;
uniform mat4 view;
uniform mat4 proj;

out vec3 v_normal;
out vec4 v_color;
out vec3 fragment_position;

void main() {
    mat4 worldview = view * world;
    v_normal = transpose(inverse(mat3(worldview))) * normal;
    v_color = color;
    // TODO check if world is correct to use below, original said model
    fragment_position = vec3(world * vec4(position, 1.0));
    // fragment_position = position;
    gl_Position = proj * worldview * vec4(position, 1.0);
}
