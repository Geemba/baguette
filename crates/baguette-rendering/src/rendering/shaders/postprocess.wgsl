struct CornerInput
{
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

//these are in the same binding because they are used in different pipelines
@group(0) @binding(0) var color: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler : sampler;

struct CornerOutput
{
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vertex(vertex : CornerInput) -> CornerOutput
{
    return CornerOutput
    (
        vec4<f32>(vertex.position, 1.0),
        vertex.tex_coords,
    );
}

@fragment
fn fragment(in: CornerOutput) -> @location(0) vec4<f32>
{
    return textureSampleLevel(color, texture_sampler, in.tex_coords, 0.);
}

// create a blue triangle 

    @vertex
    fn vert_test(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32>
    {
        let i: i32 = i32(index % 3u);
        let x: f32 = f32(i - 1) * 0.75;
        let y: f32 = f32((i & 1) * 2 - 1) * 0.75 + x * 0.2 + 0.1;
        return vec4<f32>(x, y, 0.0, 1.0);
    }

    @fragment
    fn fragment_test() -> @location(0) vec4<f32>
    {
        return vec4<f32>(0.13, 0.31, 0.85, 1.0); // cornflower blue in linear space
    }

///