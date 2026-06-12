use crate::prelude::WebviewMaterial;
#[cfg(not(target_os = "macos"))]
use crate::prelude::{WebviewSurface, update_webview_image};
use bevy::app::Plugin;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
// `RenderTextureMessage` from the core prelude is only used by the CPU `OnPaint`
// consumer (Linux/Windows). macOS injects the texture via the GPU path.
#[cfg(not(target_os = "macos"))]
use bevy_cef_core::prelude::*;
use std::hash::Hash;
use std::marker::PhantomData;

pub type WebviewExtendedMaterial<E> = ExtendedMaterial<WebviewMaterial, E>;

#[cfg(target_os = "macos")]
impl<E> crate::webview::gpu_surface::WebviewSurfaceSlot for WebviewExtendedMaterial<E>
where
    E: MaterialExtension + AsBindGroup<Data: PartialEq + Eq + Hash + Clone>,
{
    fn webview_surface_slot(&self) -> &Option<Handle<Image>> {
        &self.base.surface
    }

    fn webview_surface_slot_mut(&mut self) -> &mut Option<Handle<Image>> {
        &mut self.base.surface
    }
}

/// A plugin that extends the [`WebviewMaterial`] with a custom material extension.
///
/// This plugin allows you to create custom materials that can be used with webviews.
pub struct WebviewExtendMaterialPlugin<E>(PhantomData<E>);

impl<E> Default for WebviewExtendMaterialPlugin<E>
where
    E: MaterialExtension + Default,
    <E as AsBindGroup>::Data: PartialEq + Eq + Hash + Clone + Copy,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E> Plugin for WebviewExtendMaterialPlugin<E>
where
    E: MaterialExtension + AsBindGroup<Data: PartialEq + Eq + Hash + Clone + Copy> + Default,
{
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WebviewExtendedMaterial<E>>::default());

        // CPU `OnPaint` consumer: Linux/Windows only. macOS uses the GPU path.
        #[cfg(not(target_os = "macos"))]
        app.add_systems(PostUpdate, render::<E>);

        // macOS GPU path: register the generic surface allocate/mark systems for
        // this custom material type so it flows through the same IOSurface
        // injection pipeline as the standard material. Ordered via the shared
        // `WebviewSurfaceSet` phases (configured by `WebviewGpuInjectPlugin`,
        // which `CefPlugin` always adds on macOS).
        #[cfg(target_os = "macos")]
        {
            use crate::webview::gpu_surface::{
                WebviewSurfaceSet, allocate_webview_surfaces_for,
                mark_webview_materials_changed_for,
            };
            app.add_systems(
                Update,
                allocate_webview_surfaces_for::<WebviewExtendedMaterial<E>>
                    .in_set(WebviewSurfaceSet::Allocate),
            )
            .add_systems(
                Update,
                mark_webview_materials_changed_for::<WebviewExtendedMaterial<E>>
                    .in_set(WebviewSurfaceSet::MarkChanged),
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn render<E: MaterialExtension>(
    mut commands: Commands,
    mut er: MessageReader<RenderTextureMessage>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WebviewExtendedMaterial<E>>>,
    webviews: Query<&MeshMaterial3d<WebviewExtendedMaterial<E>>>,
) {
    for texture in er.read() {
        if let Ok(handle) = webviews.get(texture.webview)
            && let Some(material) = materials.get_mut(handle.id())
            && let Some(image) = {
                let handle = material
                    .base
                    .surface
                    .get_or_insert_with(|| images.add(Image::default()));
                commands
                    .entity(texture.webview)
                    .insert(WebviewSurface(handle.clone()));
                images.get_mut(handle.id())
            }
        {
            update_webview_image(texture, image);
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::webview::gpu_surface::WebviewSurfaceSlot;

    #[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
    struct NoopExtension {}
    impl bevy::pbr::MaterialExtension for NoopExtension {}

    #[test]
    fn slot_points_at_base_surface() {
        let mut m = WebviewExtendedMaterial::<NoopExtension> {
            base: WebviewMaterial::default(),
            extension: NoopExtension::default(),
        };
        assert!(m.webview_surface_slot().is_none());
        *m.webview_surface_slot_mut() = Some(Handle::<Image>::default());
        assert!(m.base.surface.is_some());
    }
}
