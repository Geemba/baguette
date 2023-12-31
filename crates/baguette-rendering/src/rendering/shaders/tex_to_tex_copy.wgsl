
struct VertexInput
{
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
}

struct VertexOutput
{
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

// this is the texture we will be coping from
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var sampletex: sampler;

@vertex fn vertex(vertex: VertexInput) -> VertexOutput
{
    return VertexOutput
    (
        vec4<f32>(vertex.position, 1.0),
        vertex.tex_coords
    );
}

@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32>
{
    return textureSample(tex, sampletex, in.tex_coords,);
}