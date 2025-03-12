
#version 450

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

void main()
{

#ifndef HAS_NORMALS
    vec3 dx = dFdx(object_pos);
    vec3 dy = dFdy(object_pos);
    vec3 object_normal = normalize(cross(dx, dy));
#endif
    vec3 world_normal = normalize(normal_transform*vec4(object_normal, 1.0)).xyz;

    //color = vec4(0.6f, 0.5f, 0.2f, 1.0f);
    color = vec4((object_normal+vec3(1.0))*0.5, 1.0f);
    //color = vec4(N_scaled, 1.0f);
}
