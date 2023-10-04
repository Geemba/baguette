
// learn how to import other modules

    struct CornerInput
    {
        @location(0) position: vec3<f32>,
        @location(1) tex_coords: vec2<f32>,
    }

    struct CornerOutput
    {
        @builtin(position) position: vec4<f32>,
        @location(0) tex_coords: vec2<f32>,
    }

//

@group(0) @binding(0) var<uniform> seed : f32;
@group(0) @binding(1) var<uniform> opacity : f32;
@group(0) @binding(2) var<uniform> brightness : f32;
@group(0) @binding(3) var texture_color : texture_2d<f32>;
@group(0) @binding(4) var texture_sampler : sampler;

@vertex fn vertex(vertex : CornerInput) -> CornerOutput
{
    return CornerOutput
    (
        vec4<f32>(vertex.position, 1.,),
        vertex.tex_coords,
    );
}

@fragment fn noise_grayscale(in: CornerOutput) -> @location(0) vec4<f32>
{
    let noise = (rand(in.tex_coords) - 1. + brightness) * opacity;

    let color = textureSample(texture_color, texture_sampler, in.tex_coords);
    
    return vec4<f32>(color.x + noise, color.y + noise, color.z +  noise, 1.);
}

fn rand(co : vec2<f32>) -> f32
{
    return fract(sin(dot(co, vec2(12.9898 + seed, 78.233))) * 43758.5453);
}