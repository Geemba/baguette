struct CameraUniform
{
    view_proj: mat4x4<f32>
}

// how many slices does this image have
struct SpriteSlice
{
    vertices: mat4x2<f32>,
    rows: u32,
    columns: u32,
    _padding: vec2<f32>
}

struct Uv
{
    u: f32,
    v: f32,
    _padding: vec2<f32>
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var diff_texs: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var diff_samplers: binding_array<sampler>;
@group(1) @binding(2) var<storage> sprite_data: array<SpriteSlice>;
@group(1) @binding(3) var<uniform> uvs: array<Uv, 4>;

struct InstanceInput
{
    //transform
    @location(1) model_matrix_0: vec4<f32>,
    @location(2) model_matrix_1: vec4<f32>,
    @location(3) model_matrix_2: vec4<f32>,
    @location(4) model_matrix_3: vec4<f32>,

    @location(5) uv_idx: u32,
    @location(6) bind_index: u32
}

struct VertexOutput
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) bind_index: u32
}

@vertex fn vertex(@location(0) vert_idx: u32, instance: InstanceInput) -> VertexOutput
{
    let bind_index = instance.bind_index;

    let model_matrix = mat4x4<f32>
    (
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );

    let slice = sprite_data[bind_index];

    let u_pos = f32(instance.uv_idx % slice.columns);
    let v_pos = f32(instance.uv_idx / slice.columns);

    let uv = uvs[vert_idx];
    
    let f32_columns = f32(slice.columns);
    let f32_rows = f32(slice.rows);

    let tex_coords = vec2<f32>
    (
        (uv.u / f32_columns) + u_pos * (1. / f32_columns),
        (uv.v / f32_rows) + v_pos * (1. / f32_rows)
    );

    let vertex = sprite_data[bind_index].vertices[vert_idx];

    return VertexOutput
    (
        camera.view_proj * model_matrix * vec4<f32>(vertex.x, vertex.y, 0., 1.),
        tex_coords,
        bind_index
    );
}

@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32>
{
    return textureSample(diff_texs[in.bind_index], diff_samplers[in.bind_index], in.tex_coords);
}