
#version 450

out vec4 color;

void main()
{
    color = vec4(0.6f, 0.5f, 0.2f, 1.0f);
}
/*
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    //@location(0) color: vec3<f32>,
};


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(in.color, 1.0);
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
*/


