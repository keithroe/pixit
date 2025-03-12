
#version 450

// inputs
layout (location = 0) in vec3 pos;
#ifdef HAS_NORMALS
layout (location = 1) in vec3 normal;
#endif // HAS_NORMALS

// outputs
layout (location = 0) out vec3 world_pos;
layout (location = 1) out vec3 object_pos;
#ifdef HAS_NORMALS
layout (location = 2) out vec3 object_normal;
#endif // HAS_NORMALS

// uniforms
layout (set = 0, binding = 0)
uniform mat4 view_proj_transform;

layout (set = 1, binding = 0)
uniform mat4 model_transform;

void main() {
    vec4 world_p = model_transform * vec4(pos, 1.0); 
    gl_Position = view_proj_transform * world_p; 

    world_pos = world_p.xyz;
    object_pos = pos;
#ifdef HAS_NORMALS
    object_normal = normal;
#endif // HAS_NORMALS
}
