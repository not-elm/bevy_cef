#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var surface_texture: texture_2d<f32>;
@group(1) @binding(1) var surface_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // The surface is Bgra8UnormSrgb; the format handles channel order + sRGB,
    // so a plain sample yields correct RGBA (mirrors the mesh material).
    return textureSample(surface_texture, surface_sampler, in.uv);
}
