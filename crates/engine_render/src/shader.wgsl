struct Camera {
    view_projection: mat4x4<f32>,
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(1)
var base_color_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_position = camera.model * vec4<f32>(input.position, 1.0);
    output.position = camera.view_projection * world_position;
    output.normal = normalize((camera.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.tex_coord = input.tex_coord;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(base_color_texture, base_color_sampler, input.tex_coord);
    let normal = normalize(input.normal);
    let light_direction = normalize(vec3<f32>(-0.4, -0.8, -0.6));
    let diffuse = max(dot(normal, -light_direction), 0.0);
    let illumination = 0.25 + 0.75 * diffuse;
    return vec4<f32>(base_color.rgb * illumination, base_color.a);
}
