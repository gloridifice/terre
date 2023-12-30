struct InstanceInput{
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
}

struct GlobalUniform{
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
    ambient: vec4<f32>,
}

struct LightUniform{
    position: vec3<f32>,
    color: vec3<f32>,
    direction: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> global: GlobalUniform;

@group(0) @binding(1)
var<uniform> light: LightUniform;

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let world_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
        );

    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = global.view_proj * world_matrix * vec4<f32>(model.position, 1.0);
    var world_position: vec4<f32> = world_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.world_normal = normal_matrix * model.normal;
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let diffuse_strength = max(dot(in.world_normal, light.direction), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let ambient_color = global.ambient;

    let result = (ambient_color + vec4<f32>(diffuse_color, 1.0)) * tex_color;
    return result;
}