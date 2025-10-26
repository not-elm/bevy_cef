
#define_import_path webview::util

#import bevy_pbr::{
    mesh_view_bindings::view,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(101) var surface_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var surface_sampler: sampler;

fn surface_color(uv: vec2<f32>) -> vec4<f32> {
    return textureSampleBias(surface_texture, surface_sampler, uv, view.mip_bias);
}