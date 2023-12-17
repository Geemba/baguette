struct CameraUniform
{
    view_proj: mat4x4<f32>
}

@group(1) @binding(0) var<uniform> camera: CameraUniform;

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@group(0) @binding(2) var<storage> uvs: array<vec2<f32>>;

struct VertexInput
{
    @builtin(vertex_index) vertex_index: u32,
    @location(0) x: f32,
    @location(1) y: f32,
}
struct InstanceInput
{
    //transform
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    // index to the instance's uvs
    @location(6) index: u32
}

struct VertexOutput
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

@vertex fn vertex(input: VertexInput, instance: InstanceInput) -> VertexOutput
{
    let model_matrix = mat4x4<f32>
    (
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3
    );

    let position = vec3<f32>(input.x, input.y, 0.);

    return VertexOutput
    (
        camera.view_proj * model_matrix * vec4<f32>(position, 1.),
        uvs[instance.index + input.vertex_index]
    );
}


@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32>
{
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}