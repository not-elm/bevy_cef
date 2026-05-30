//! `bevy_ui` webview display path: renders a webview into a `MaterialNode<WebviewUiMaterial>`.

use crate::prelude::{
    WebviewDpr, WebviewSize, WebviewSource, WebviewSurface, update_webview_image,
};
use crate::webview::ui::material::WEBVIEW_UI_SHADER_HANDLE;
use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy_cef_core::prelude::RenderTextureMessage;

mod input;
mod material;

pub use material::WebviewUiMaterial;

/// Adds the `bevy_ui` webview display path. Sibling to `MeshWebviewPlugin`.
pub(in crate::webview) struct UiWebviewPlugin;

impl Plugin for UiWebviewPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            WEBVIEW_UI_SHADER_HANDLE,
            "ui/webview_ui.wgsl",
            Shader::from_wgsl
        );

        if !app.is_plugin_added::<bevy::ui::picking_backend::UiPickingPlugin>() {
            app.add_plugins(bevy::ui::picking_backend::UiPickingPlugin);
        }

        app.add_plugins(UiMaterialPlugin::<WebviewUiMaterial>::default())
            .add_systems(
                PostUpdate,
                (
                    render_ui_surface.run_if(on_message::<RenderTextureMessage>),
                    update_webview_ui_size.after(bevy::ui::UiSystems::Layout),
                ),
            );

        #[cfg(not(target_os = "windows"))]
        app.add_systems(Update, input::setup_ui_observers);

        #[cfg(target_os = "windows")]
        app.add_systems(Update, input::setup_ui_observers_win);
    }
}

/// Converts a node's physical-pixel `ComputedNode` size to the logical DIP size
/// `WebviewSize` expects. Returns `None` for a pre-layout / sub-pixel size so a
/// 0-area surface is never requested.
pub(crate) fn webview_size_from_computed(
    physical_size: Vec2,
    inverse_scale_factor: f32,
) -> Option<Vec2> {
    let logical = physical_size * inverse_scale_factor;
    if logical.x < 1.0 || logical.y < 1.0 {
        None
    } else {
        Some(logical)
    }
}

/// Copies each incoming CEF frame into the corresponding entity's
/// `WebviewUiMaterial` surface, allocating the `Image` on first frame. Mirrors
/// the mesh path's `render` system. Runs in `PostUpdate`.
fn render_ui_surface(
    mut commands: Commands,
    mut er: MessageReader<RenderTextureMessage>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
    webviews: Query<&MaterialNode<WebviewUiMaterial>>,
) {
    for texture in er.read() {
        if let Ok(handle) = webviews.get(texture.webview)
            && let Some(material) = materials.get_mut(handle.id())
            && let Some(image) = {
                let image_handle = material
                    .surface
                    .get_or_insert_with(|| images.add(Image::default()));
                commands
                    .entity(texture.webview)
                    .insert(WebviewSurface(image_handle.clone()));
                images.get_mut(image_handle.id())
            }
        {
            update_webview_image(texture, image);
        }
    }
}

/// Writes `WebviewSize` (logical DIP) from each UI webview node's `ComputedNode`
/// (physical px). Runs in `PostUpdate` after layout; `set_if_neq` avoids a
/// needless CEF resize when the size is unchanged. Re-runs on DPR change too.
fn update_webview_ui_size(
    mut webviews: Query<
        (&ComputedNode, &mut WebviewSize),
        (
            With<WebviewSource>,
            With<MaterialNode<WebviewUiMaterial>>,
            Or<(Changed<ComputedNode>, Changed<WebviewDpr>)>,
        ),
    >,
) {
    for (computed, mut size) in webviews.iter_mut() {
        if let Some(logical) =
            webview_size_from_computed(computed.size(), computed.inverse_scale_factor())
        {
            size.set_if_neq(WebviewSize(logical));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_physical_to_logical() {
        // 1600x1200 physical at 2x DPI (inverse 0.5) -> 800x600 logical.
        let out = webview_size_from_computed(Vec2::new(1600.0, 1200.0), 0.5);
        assert_eq!(out, Some(Vec2::new(800.0, 600.0)));
    }

    #[test]
    fn identity_at_1x() {
        let out = webview_size_from_computed(Vec2::new(640.0, 480.0), 1.0);
        assert_eq!(out, Some(Vec2::new(640.0, 480.0)));
    }

    #[test]
    fn zero_and_subpixel_return_none() {
        assert_eq!(webview_size_from_computed(Vec2::ZERO, 1.0), None);
        assert_eq!(webview_size_from_computed(Vec2::new(0.0, 600.0), 1.0), None);
        assert_eq!(webview_size_from_computed(Vec2::new(0.5, 0.5), 1.0), None);
    }
}
