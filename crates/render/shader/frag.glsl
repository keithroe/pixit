
#version 450

layout(location = 0) in vec3 P;
#ifdef HAS_NORMALS
layout(location = 1) in vec3 N;
#endif
out vec4 color;

void main()
{
#ifdef HAS_NORMALS
    vec3 norm = N;
#else
    vec3 dx = dFdx(P);
    vec3 dy = dFdy(P);
    vec3 norm = normalize(cross(dx, dy));
#endif
    //color = vec4(0.6f, 0.5f, 0.2f, 1.0f);
    color = vec4((norm+vec3(1.0))*0.5, 1.0f);
    //color = vec4(N_scaled, 1.0f);
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


