
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

@group(0) @binding(0) var color: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

@group(0) @binding(3) var<uniform> opacity : f32;

@vertex fn vertex(vertex : CornerInput) -> CornerOutput
{
    return CornerOutput
    (
        vec4<f32>(vertex.position, 1.),
        vertex.tex_coords,
    );
}

const pi2 = 6.28318531;

@fragment fn fragment(in: CornerOutput) -> @location(0) vec4<f32>
{  
    // GAUSSIAN BLUR SETTINGS {{{
    let directions = 64.; // BLUR DIRECTIONS (Default 16.0 - More is better but slower)
    let quality = 12.; // BLUR QUALITY (Default 4.0 - More is better but slower)
    let size = .04; // BLUR SIZE (Radius)
    // GAUSSIAN BLUR SETTINGS }}}
    
    // pixel color
    var pixel = textureSample(color, texture_sampler, in.tex_coords);

    // Blur calculations
    for(var d = 0.; d < pi2; d += pi2 / directions)
    {
		for(var i = 1. / quality; i <= 1.; i += 1./ quality)
        {
			pixel += textureSample(color, texture_sampler, in.tex_coords + vec2<f32>(cos(d),sin(d)) * size * i);		
        }
    }
    
    pixel /= quality * directions - directions - 1.;
    return pixel;
}

