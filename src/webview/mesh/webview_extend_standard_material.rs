use crate::prelude::WebviewMaterial;
#[cfg(not(target_os = "macos"))]
use crate::prelude::{WebviewSurface, update_webview_image};
use bevy::asset::*;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::shader::ShaderRef;
// `RenderTextureMessage` from the core prelude is only used by the CPU `OnPaint`
// consumer (Linux/Windows). macOS injects the texture via the GPU path.
#[cfg(not(target_os = "macos"))]
use bevy_cef_core::prelude::*;

const FRAGMENT_SHADER_HANDLE: Handle<Shader> = uuid_handle!("b231681f-9c17-4df6-89c9-9dc353e85a08");

pub(super) struct WebviewExtendStandardMaterialPlugin;

impl Plugin for WebviewExtendStandardMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WebviewExtendStandardMaterial>::default());

        // CPU `OnPaint` consumer: Linux/Windows only. macOS injects the live
        // webview texture into `RenderAssets<GpuImage>` via the GPU path.
        #[cfg(not(target_os = "macos"))]
        app.add_systems(PostUpdate, render_standard_materials);

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

#[cfg(not(target_os = "macos"))]
fn render_standard_materials(
    mut commands: Commands,
    mut er: MessageReader<RenderTextureMessage>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
    webviews: Query<&MeshMaterial3d<WebviewExtendStandardMaterial>>,
) {
    for texture in er.read() {
        if let Ok(handle) = webviews.get(texture.webview)
            && let Some(mut material) = materials.get_mut(handle.id())
            && let Some(image) = {
                let handle = material
                    .extension
                    .surface
                    .get_or_insert_with(|| images.add(Image::default()));
                commands
                    .entity(texture.webview)
                    .insert(WebviewSurface(handle.clone()));
                images.get_mut(handle.id())
            }
        {
            update_webview_image(texture, image.into_inner());
        }
    }
}

#[cfg(target_os = "macos")]
impl crate::webview::gpu_surface::WebviewSurfaceSlot for WebviewExtendStandardMaterial {
    fn webview_surface_slot(&self) -> &Option<Handle<Image>> {
        &self.extension.surface
    }

    fn webview_surface_slot_mut(&mut self) -> &mut Option<Handle<Image>> {
        &mut self.extension.surface
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::prelude::WebviewMaterial;
    use crate::webview::gpu_surface::WebviewSurfaceSlot;

    #[test]
    fn slot_points_at_extension_surface() {
        let mut m = WebviewExtendStandardMaterial {
            base: StandardMaterial::default(),
            extension: WebviewMaterial::default(),
        };
        // Default has no surface yet.
        assert!(m.webview_surface_slot().is_none());
        // Writing through the slot must land in `extension.surface`.
        *m.webview_surface_slot_mut() = Some(Handle::<Image>::default());
        assert!(m.extension.surface.is_some());
    }
}
