struct VertexOutput
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) bind_index: u32,
}

struct CameraUniform
{
    view_proj: mat4x4<f32>
}

struct Tile
{
	@location(1) pos: vec2<f32>,
	@location(2) uv: vec2<f32>
}

struct TileMap
{
    rows: u32,
    columns: u32
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var diff_texs: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var diff_samplers: binding_array<sampler>;
@group(1) @binding(2) var<uniform> matrix: mat4x4<f32>;

@group(2) @binding(0) var layer_depth: texture_storage_2d<r32uint, write>;

@vertex fn vertex(@location(0) vertex: vec2<f32>, tile: Tile) -> VertexOutput
{
    return VertexOutput
    (
        camera.view_proj * matrix *
        vec4<f32>
        (
            vertex.x,
            vertex.y,
            0.,
            1.
        ),
        tile.uv
    )
}

@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32>
{
    let tex_size: vec2<u32> = textureDimensions(layer_depth);

    let screen_coords = vec2<u32>
    (
        u32((in.clip_position.x * 0.5 + 0.5) * tex_size.x),
        u32((in.clip_position.y * 0.5 + 0.5) * tex_size.y)
    );

    let layer = textureLoad(layer_depth, screen_coords, 0);
    textureLoad(layer_depth, screen_coords, layer + 1u);

    return textureSample(diff_texs[in.bind_index], diff_samplers[in.bind_index], in.tex_coords);
}