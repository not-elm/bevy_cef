//! Turnkey rebind propagation for third-party materials that sample a headless
//! webview's texture (see `WebviewTextureTarget`).
//!
//! Base contract (no trait needed): when the injected GPU texture for a
//! `WebviewTextureTarget` is (re)created — first frame, resize, handle swap —
//! bevy_cef touches the target `Image` asset, firing
//! `AssetEvent::Modified { id }`. A consumer that manages its own material can
//! listen for that event and deref-mutate its material asset (via `get_mut`)
//! to rebuild the bind group. Bevy 0.19's `Assets::get_mut` returns an
//! `AssetMut` guard that only queues `Modified` if actually deref-mutated (or
//! `into_inner()` is called) before it drops — a bare `let _ = get_mut(..)`
//! silently does nothing:
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
//!             if let Some(material) = materials.get_mut(&my.material) {
//!                 let _ = material.into_inner(); // force Modified → bind-group rebuild
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! Ordering caveat for the manual path: the rebind image-touch ALSO makes
//! `prepare_assets::<GpuImage>` re-upload the CPU placeholder that same frame
//! (the `Modified` event is shared). A material bind group rebuilt that frame
//! must therefore be prepared AFTER bevy_cef's GPU injection, or it can
//! capture the placeholder instead of the webview texture — and stay black
//! forever, because nothing rebuilds it once the rebind window closes.
//! [`WebviewTargetUiMaterialPlugin`] configures that ordering automatically;
//! manual consumers must order [`WebviewGpuImageInjectSet`] before their
//! material's `prepare_assets` in the `Render` schedule (see the set's docs).
//!
//! This module is compiled on every platform so downstream crates never need
//! `#[cfg]`; on non-macOS the plugin registers nothing (the headless texture
//! path is macOS-only).

use bevy::asset::AssetId;
use bevy::prelude::*;
use std::marker::PhantomData;

/// Render-world system set containing bevy_cef's webview GPU texture injection
/// (`RenderSystems::PrepareAssets` phase; populated on macOS only).
///
/// A material that samples a `WebviewTextureTarget` image must build its bind
/// group AFTER this set: on rebind frames the image-touch re-uploads the CPU
/// placeholder, and an unordered rebuild can land between that re-upload and
/// the injection, capturing the placeholder permanently.
/// [`WebviewTargetUiMaterialPlugin`] adds the required edge automatically;
/// manual consumers do:
///
/// ```ignore
/// use bevy::render::{Render, RenderApp, render_asset::prepare_assets};
/// use bevy::ui_render::PreparedUiMaterial;
///
/// render_app.configure_sets(
///     Render,
///     WebviewGpuImageInjectSet.before(prepare_assets::<PreparedUiMaterial<MyMaterial>>),
/// );
/// ```
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub struct WebviewGpuImageInjectSet;

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
            use bevy::render::{Render, RenderApp, render_asset::prepare_assets};
            use bevy::ui_render::PreparedUiMaterial;

            _app.add_systems(
                Update,
                mark_target_materials_changed_for::<M>.in_set(WebviewSurfaceSet::MarkChanged),
            );

            // Deterministic rebind: the rebind image-touch re-uploads the CPU
            // placeholder in the very frames this plugin rebuilds M's bind
            // group. Without this edge the rebuild can run between the
            // placeholder re-upload and the GPU injection and capture the
            // placeholder — permanently, since nothing rebuilds the bind group
            // after the rebind window closes (observed as a forever-black
            // texture on machines where the scheduler picks that order).
            if let Some(render_app) = _app.get_sub_app_mut(RenderApp) {
                render_app.configure_sets(
                    Render,
                    WebviewGpuImageInjectSet.before(prepare_assets::<PreparedUiMaterial<M>>),
                );
            }
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
