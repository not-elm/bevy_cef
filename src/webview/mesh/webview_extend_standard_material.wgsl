#define_import_path webview::standard_material;

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    pbr_bindings::material,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_view_bindings::view,
}
#import webview::util::{
    surface_color,
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color *= surface_color(in.uv);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    var out: FragmentOutput;
    if (material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
        out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    }else{
        out.color = pbr_input.material.base_color;
    }
    return out;
}
