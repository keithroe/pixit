
#version 450

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 normal;

layout (set = 0, binding = 0)
uniform mat4 view_proj_transform;

layout (set = 1, binding = 0)
uniform mat4 model_transform;
layout (set = 1, binding = 1)
uniform mat4 normal_transform;

layout (location = 0) out vec3 world_pos;
layout (location = 1) out vec3 world_normal;

void main() {
    vec4 world_p = model_transform * vec4(pos, 1.0); 

    gl_Position = view_proj_transform * world_p; 
    world_pos = world_p.xyz;
    world_normal = normalize((normal_transform * vec4(normal, 0.0)).xyz);
}
