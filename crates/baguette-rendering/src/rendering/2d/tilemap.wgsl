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
	@location(2) pos: vec2<f32>,
	@location(3) idx: u32,
	@location(4) bind_idx: u32,
}

struct TileMap
{
    rows: u32,
    columns: u32
}

struct VertexInput
{
    @location(0) vert: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> matrix: mat4x4<f32>;

@group(2) @binding(0) var diff_texs: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var diff_sampler: sampler;
@group(2) @binding(2) var layer_depth: texture_storage_2d<r32uint, read_write>;

@vertex fn vertex(in: VertexInput, tile: Tile) -> VertexOutput
{
    return VertexOutput
    (
        camera.view_proj * matrix *
        vec4<f32>(in.vert.xy + tile.pos.xy, 0., 1.),
        in.uv,
        tile.bind_idx
    );
}

@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32>
{
    //todo check that this is working
    //let tex_size: vec2<u32> = textureDimensions(layer_depth);

    //let screen_coords = vec2<u32>
    //(
    //    u32((in.clip_position.x * 0.5 + 0.5)) * tex_size.x,
    //    u32((in.clip_position.y * 0.5 + 0.5)) * tex_size.y
    //);

    //let layer = textureLoad(layer_depth, screen_coords);
    //textureStore(layer_depth, screen_coords, layer + 1u);

    return textureSample(diff_texs[in.bind_index], diff_sampler, in.tex_coords);
}