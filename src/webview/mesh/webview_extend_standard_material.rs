use crate::prelude::{WebviewMaterial, update_webview_image};
use bevy::asset::*;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::shader::ShaderRef;
use bevy_cef_core::prelude::*;

const FRAGMENT_SHADER_HANDLE: Handle<Shader> = uuid_handle!("b231681f-9c17-4df6-89c9-9dc353e85a08");

pub(super) struct WebviewExtendStandardMaterialPlugin;

impl Plugin for WebviewExtendStandardMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WebviewExtendStandardMaterial>::default())
            .add_systems(PostUpdate, render_standard_materials);
        load_internal_asset!(
            app,
            FRAGMENT_SHADER_HANDLE,
            "./webview_extend_standard_material.wgsl",
            Shader::from_wgsl
        );
    }
}

impl MaterialExtension for WebviewMaterial {
    fn fragment_shader() -> ShaderRef {
        FRAGMENT_SHADER_HANDLE.into()
    }
}

pub type WebviewExtendStandardMaterial = ExtendedMaterial<StandardMaterial, WebviewMaterial>;

fn render_standard_materials(
    mut er: MessageReader<RenderTextureMessage>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
    webviews: Query<&MeshMaterial3d<WebviewExtendStandardMaterial>>,
) {
    for texture in er.read() {
        if let Ok(handle) = webviews.get(texture.webview)
            && let Some(material) = materials.get_mut(handle.id())
            && let Some(image) = {
                let handle = material
                    .extension
                    .surface
                    .get_or_insert_with(|| images.add(Image::default()));
                images.get_mut(handle.id())
            }
        {
            //OPTIMIZE: Avoid cloning the texture.
            update_webview_image(texture.clone(), image);
        }
    }
}
