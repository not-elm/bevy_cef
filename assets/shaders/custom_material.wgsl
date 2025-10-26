#import bevy_pbr::{
    forward_io::VertexOutput,
}
#import webview::util::{
    surface_color,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var mask_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var mask_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // You can obtain the surface color.
    var color = surface_color(in.uv);
    // Blend the color with the mask texture.
    color *= (textureSample(mask_texture, mask_sampler, in.uv) * vec4(vec3(1.0), 0.3));
    return color;
}
