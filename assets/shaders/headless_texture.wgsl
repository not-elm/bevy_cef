#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var webview_texture: texture_2d<f32>;
@group(1) @binding(1) var webview_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // The target is Bgra8UnormSrgb: a plain sample yields linear RGBA (the
    // format handles channel order + sRGB decode) — do not decode manually.
    let color = textureSample(webview_texture, webview_sampler, in.uv);
    // Green-ish tint proves a third-party (non-bevy_cef) material is drawing.
    return vec4(color.rgb * vec3(0.8, 1.0, 0.8), color.a);
}
