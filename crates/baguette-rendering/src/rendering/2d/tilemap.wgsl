struct VertexOutput
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) bind_index: u32,
    @location(2) layer: f32
}

struct TileMap
{
    rows: f32,
    columns: f32,
    vertices: mat4x2<f32>,
    // the size of one tile
    size: vec2<f32>
}

struct VertexInput
{
    @builtin(vertex_index) index: u32,
    @location(0) u: f32,
    @location(1) v: f32
}

struct Tile
{
	@location(2) pos: vec2<f32>,
	@location(3) row: u32,
	@location(4) column: u32,
	@location(5) bind_idx: u32, // the index of the texture in the array
    @location(6) layer: f32,
}

@group(0) @binding(0) var<uniform> camera_proj: mat4x4<f32>;
@group(1) @binding(0) var<uniform> matrix: mat4x4<f32>;

@group(2) @binding(0) var diff_texs: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var diff_sampler: sampler;
@group(2) @binding(2) var layer_depth: texture_storage_2d<r32float, read_write>;
@group(2) @binding(3) var<storage> tilemap_data: array<TileMap>;

@vertex fn vertex
(
    vertex: VertexInput,
    tile: Tile

) -> VertexOutput
{
    var tilemap = tilemap_data[tile.bind_idx];
    let pos = tilemap.size[vertex.index];

    let tex_coords = vec2<f32>
    (
        1. / tilemap.rows * f32(tile.row) + (vertex.u / tilemap.rows),
        1. / tilemap.columns * f32(tile.column) + (vertex.v / tilemap.columns)
    );

    return VertexOutput
    (
        camera_proj * matrix *
        vec4<f32>(pos.xy + tile.pos.xy, 0., 1.),
        tex_coords,
        tile.bind_idx,
        tile.layer
    );
}

@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32>
{
    let tex_size: vec2<u32> = textureDimensions(layer_depth);
    
    let screen_coords = vec2<u32>
    (
        u32(in.clip_position.x * 0.5 + 0.5 * f32(tex_size.x)),
        u32(in.clip_position.y * 0.5 + 0.5 * f32(tex_size.y))
    );

    var layer: vec4<f32> = textureLoad(layer_depth, screen_coords);

    var diffuse = textureSample(diff_texs[in.bind_index], diff_sampler, in.tex_coords);

    //textureStore(layer_depth, screen_coords, vec4<f32>(0.1, 0., 0., 0.));

    return diffuse;
}