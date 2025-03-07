
#version 450

layout (location = 0) in vec3 pos;

layout (set = 0, binding = 0)
uniform mat4 transform;

void main() {
    gl_Position = transform * vec4(pos.x, pos.y, pos.z, 1.0);
}


/*
// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
 //   @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    //@location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    //out.color = model.color;
    //out.clip_position = vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

*/
