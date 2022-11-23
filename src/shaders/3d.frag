#version 330 core

in vec3 v_normal;
in vec4 v_color;
in vec3 fragment_position;

struct Light {
    vec3 position;
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform vec3 view_position;
uniform Light light;

// const vec3 LIGHT = vec3(1.0, 1.0, 1.0);

void main() {

    vec3 norm = normalize(v_normal);
    vec3 light_direction = normalize(light.position - fragment_position);
    vec3 view_direction = normalize(view_position - fragment_position);
    vec3 reflection_direction = reflect(-light_direction, norm);

    vec3 ambient = light.ambient; 
    vec3 diffuse = light.diffuse * max(dot(norm, light_direction), 0.0);
    vec3 specular = light.specular * pow(max(dot(view_direction, reflection_direction), 0.0), 32);

    gl_FragColor = vec4((ambient + diffuse + specular), 1.0) * v_color;
}
