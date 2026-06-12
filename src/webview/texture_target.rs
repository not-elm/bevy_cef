//! Turnkey rebind propagation for third-party materials that sample a headless
//! webview's texture (see `WebviewTextureTarget`).
//!
//! Base contract (no trait needed): when the injected GPU texture for a
//! `WebviewTextureTarget` is (re)created — first frame, resize, handle swap —
//! bevy_cef touches the target `Image` asset, firing
//! `AssetEvent::Modified { id }`. A consumer that manages its own material can
//! listen for that event and `get_mut` its material asset to rebuild the bind
//! group:
//!
//! ```ignore
//! fn rebuild_on_webview_rebind(
//!     mut events: MessageReader<AssetEvent<Image>>,
//!     mut materials: ResMut<Assets<MyMaterial>>,
//!     my: Res<MyHandles>, // your own bookkeeping: target id + material handle
//! ) {
//!     for event in events.read() {
//!         if let AssetEvent::Modified { id } = event
//!             && *id == my.target.id()
//!         {
//!             let _ = materials.get_mut(&my.material);
//!         }
//!     }
//! }
//! ```
//!
//! This module is compiled on every platform so downstream crates never need
//! `#[cfg]`; on non-macOS the plugin registers nothing (the headless texture
//! path is macOS-only).

use bevy::asset::AssetId;
use bevy::prelude::*;
use std::marker::PhantomData;

/// Tells bevy_cef which webview texture targets an asset references, so
/// [`WebviewTargetUiMaterialPlugin`] can rebuild its bind group on rebind
/// frames.
///
/// Bounded on [`Asset`] (not a material trait): the mechanism — "touch the
/// asset so its own `Modified` event re-prepares it" — is identical for every
/// material kind, so future `Material` / `Material2d` plugin variants reuse
/// this same trait.
pub trait WebviewTextureSlot: Asset {
    /// The webview target asset ids this asset currently references.
    fn webview_targets(&self) -> impl Iterator<Item = AssetId<Image>>;
}

/// Registers rebind propagation for a third-party [`UiMaterial`] sampling a
/// headless webview texture: on rebind frames, every `M` asset whose
/// [`WebviewTextureSlot::webview_targets`] contains a rebinding target id is
/// touched, so Bevy rebuilds its bind group against the freshly injected
/// texture (instead of sampling the stale placeholder forever).
///
/// Add your own `UiMaterialPlugin::<M>` as usual — this plugin only adds the
/// rebind system and may be combined with any number of material types.
pub struct WebviewTargetUiMaterialPlugin<M>(PhantomData<M>);

impl<M> Default for WebviewTargetUiMaterialPlugin<M>
where
    M: WebviewTextureSlot + UiMaterial,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<M> Plugin for WebviewTargetUiMaterialPlugin<M>
where
    M: WebviewTextureSlot + UiMaterial,
{
    fn build(&self, _app: &mut App) {
        // macOS GPU path only: the rebind marker and surface pipeline live in
        // the macOS-gated `gpu_surface` module. Same cfg idiom as
        // `WebviewExtendMaterialPlugin` (webview_extend_material.rs).
        #[cfg(target_os = "macos")]
        {
            use crate::webview::gpu_surface::{
                WebviewSurfaceSet, mark_target_materials_changed_for,
            };
            _app.add_systems(
                Update,
                mark_target_materials_changed_for::<M>.in_set(WebviewSurfaceSet::MarkChanged),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::AsBindGroup;
    use bevy::shader::ShaderRef;

    #[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
    struct SlotMaterial {
        #[texture(0)]
        #[sampler(1)]
        webview: Option<Handle<Image>>,
    }

    impl UiMaterial for SlotMaterial {
        fn fragment_shader() -> ShaderRef {
            ShaderRef::Default
        }
    }

    impl WebviewTextureSlot for SlotMaterial {
        fn webview_targets(&self) -> impl Iterator<Item = AssetId<Image>> {
            self.webview.iter().map(Handle::id)
        }
    }

    #[test]
    fn webview_targets_yields_slot_id() {
        let mut material = SlotMaterial::default();
        assert_eq!(material.webview_targets().count(), 0);

        let handle = Handle::<Image>::default();
        material.webview = Some(handle.clone());
        assert_eq!(material.webview_targets().next(), Some(handle.id()));
    }
}
