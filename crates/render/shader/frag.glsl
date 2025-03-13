
#version 450

struct Light {
    vec3 dir;
    uint enabled;

    vec3 color;
    float scale;

    vec3 ambient;
    float ambient_scale;

};

// inputs
layout(location = 0) in vec3 world_pos;
layout(location = 1) in vec3 object_pos;
#ifdef HAS_NORMALS
layout(location = 2) in vec3 object_normal;
#endif

// outputs
out vec4 color;

// uniforms
layout (set = 1, binding = 1)
uniform mat4 normal_transform;

layout (set = 2, binding = 0)
uniform Light light;

void main()
{

#ifndef HAS_NORMALS
    vec3 dx = dFdx(object_pos);
    vec3 dy = dFdy(object_pos);
    vec3 object_normal = normalize(cross(dx, dy));
#endif
    vec3 world_normal = normalize(normal_transform*vec4(object_normal, 0.0)).xyz;

    vec3 w_in = -normalize(light.dir);
    float n_dot_w = max(0.0, dot(world_normal, w_in));
    //float n_dot_w = max(0.0, dot(object_normal, w_in));
    vec3 l = n_dot_w * light.color * light.scale + light.ambient*light.ambient_scale;
    vec3 b = vec3(1.0); 
    color = vec4(b*l, 1.0);

    //color = vec4((object_normal+vec3(1.0))*0.5, 1.0f);
}
