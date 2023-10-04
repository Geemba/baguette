
/// shader to mix to textures

// learn how to import other modules to not do stuff like this

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

@group(0) @binding(0) var color1: texture_2d<f32>;
@group(0) @binding(1) var color2: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

@fragment fn fragment(in: CornerOutput) -> @location(0) vec4<f32>
{
    let tex1 = textureSampleLevel(color1, texture_sampler, in.tex_coords, 0.);
    let tex2 = textureSampleLevel(color2, texture_sampler, in.tex_coords, 0.);
    
    return vec4<f32>
    (
        tex1.x + tex2.x,
        tex1.y + tex2.y,
        tex1.z + tex2.z,
        tex1.w
    );
}