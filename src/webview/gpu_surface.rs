//! [macOS GPU OSR — Approach 2] Import each webview's retained IOSurface and
//! blit it into an owned GPU texture inside a Bevy render-graph node, then inject
//! that owned texture into `RenderAssets<GpuImage>` so the webview mesh material
//! samples the live page.
//!
//! Why a render-graph node (not the `on_accelerated_paint` callback): Bevy owns
//! ordered command submission — its render graph collects all GPU commands into
//! one `RenderContext` and submits them once per frame, then presents. Doing an
//! out-of-band `queue.submit` from the CEF callback (which runs in the `Main`
//! schedule) races that ordered submit/present and corrupts rendering (the mesh
//! goes black, no validation error). So the callback only *retains* the latest
//! IOSurface; the import + blit is recorded into the frame's command encoder by
//! `WebviewBlitNode`, and Bevy submits it in order.
//!
//! Frame flow (macOS):
//! 1. Main world (`Update`): `allocate_webview_surfaces_for::<M>` (mesh, per
//!    registered material type),
//!    `allocate_ui_webview_surfaces` (bevy_ui), and
//!    `allocate_sprite_webview_surfaces` (2D `Sprite`) give every webview a
//!    placeholder `Handle<Image>` + `WebviewSurface` tag. Mesh/UI store the
//!    handle on their material; sprites set it as `Sprite.image`. The collect →
//!    node → inject pipeline is keyed by `WebviewSurface`'s `AssetId<Image>`, so
//!    it is material/sprite-agnostic.
//! 2. Main world (`Update`, after allocation): `collect_webview_iosurfaces` pulls
//!    the latest retained IOSurface ptr per webview out of `Browsers` (`NonSend`)
//!    and pairs each with its surface `AssetId<Image>` into
//!    `PendingWebviewIoSurfaces`.
//! 3. `ExtractSchedule`: `extract_webview_iosurfaces` copies the pending list into
//!    the render world (`ExtractedWebviewIoSurfaces`), and
//!    `extract_live_webview_surface_ids` records the live webviews' surface ids
//!    (so step 5 can prune entries for despawned webviews).
//!
//! The remaining steps run in render-schedule order — note `PrepareAssets` runs
//! BEFORE the render graph, so inject (5) precedes the blit (6):
//!
//! 4. Render `PrepareResources`: `prepare_webview_gpu_surfaces` clears the previous
//!    frame's imported transient textures (now safely submitted).
//! 5. Render `PrepareAssets` (after `prepare_assets::<GpuImage>`, before the
//!    material bind-group build): `inject_webview_gpu_images` prunes despawned
//!    surfaces, get-or-creates the owned `WebviewGpuSurface` for each id (it must
//!    exist before the material bind group is built), wraps each owned surface in a
//!    `GpuImage`, and inserts it into `RenderAssets<GpuImage>` for the surface id.
//! 6. Render graph (`WebviewBlitNode`, before `CameraDriverLabel`): import each
//!    retained IOSurface into a transient wgpu texture and record a blit into the
//!    frame's command encoder, filling the owned surface created in step 5. The
//!    transient textures are stashed so they outlive encoder submission.
//!
//! The owned texture is a single stable buffer (MVP, no double-buffering): the
//! node blits into the same texture each frame, and the injected `GpuImage`
//! reuses the same `texture_view`, so the material bind group stays valid.

use crate::common::{WebviewIoSurface, WebviewSource};
use crate::prelude::{WebviewExtendStandardMaterial, WebviewSurface};
use crate::webview::ui::WebviewUiMaterial;
use bevy::asset::{AssetId, RenderAssetUsages};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::{
    Extract, Render, RenderApp, RenderSystems,
    erased_render_asset::prepare_erased_assets,
    render_asset::{RenderAssets, prepare_assets},
    render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
    render_resource::{Extent3d, Texture, TextureFormat},
    renderer::{RenderContext, RenderDevice},
    texture::{DefaultImageSampler, GpuImage},
};
use bevy::ui_render::PreparedUiMaterial;
use bevy_cef_core::prelude::{Browsers, RetainedIoSurface, WebviewGpuSurface};

const SURFACE_WIDTH: u32 = 800;
const SURFACE_HEIGHT: u32 = 800;

/// Abstracts where a mesh webview material stores its surface `Handle<Image>`.
///
/// - `WebviewExtendStandardMaterial = ExtendedMaterial<StandardMaterial, WebviewMaterial>`
///   keeps it in `extension.surface`.
/// - `WebviewExtendedMaterial<E>     = ExtendedMaterial<WebviewMaterial, E>`
///   keeps it in `base.surface`.
///
/// `allocate_webview_surfaces_for` / `mark_webview_materials_changed_for` are
/// generic over any mesh material implementing this trait, so the standard and
/// custom materials share one GPU-injection code path.
pub(crate) trait WebviewSurfaceSlot: bevy::pbr::Material {
    fn webview_surface_slot(&mut self) -> &mut Option<Handle<Image>>;
}

/// Update-schedule phases for the macOS webview surface pipeline. Exposed so the
/// custom-material plugin can register its generic allocate/mark systems in the
/// right phase relative to `collect_webview_iosurfaces`.
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum WebviewSurfaceSet {
    Allocate,
    Collect,
    MarkChanged,
}

/// Render-graph label for the webview IOSurface import + blit node.
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct WebviewBlitLabel;

/// [macOS GPU OSR] plugin: import each webview's retained IOSurface in a custom
/// render-graph node and inject the owned GPU texture into the render world so
/// the webview mesh renders the real page.
pub struct WebviewGpuInjectPlugin;

impl Plugin for WebviewGpuInjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingWebviewIoSurfaces>()
            .configure_sets(
                Update,
                (
                    WebviewSurfaceSet::Allocate,
                    WebviewSurfaceSet::Collect,
                    WebviewSurfaceSet::MarkChanged,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    allocate_webview_surfaces_for::<WebviewExtendStandardMaterial>,
                    allocate_ui_webview_surfaces,
                    allocate_sprite_webview_surfaces,
                )
                    .in_set(WebviewSurfaceSet::Allocate),
            )
            .add_systems(
                Update,
                collect_webview_iosurfaces.in_set(WebviewSurfaceSet::Collect),
            )
            .add_systems(
                Update,
                (
                    // Touch every webview material each frame so Bevy rebuilds its
                    // bind group against the injected owned-texture view (the cached
                    // bind group otherwise stays bound to the black placeholder).
                    // We must NOT touch the placeholder Image asset for mesh/UI
                    // (that would make `prepare_assets::<GpuImage>` re-upload the
                    // black placeholder). Sprites have no material asset, so
                    // `mark_sprite_webview_images_changed` instead fires
                    // `AssetEvent::Modified` on the sprite's `Image`.
                    mark_webview_materials_changed_for::<WebviewExtendStandardMaterial>,
                    mark_webview_ui_materials_changed,
                    mark_sprite_webview_images_changed,
                )
                    .in_set(WebviewSurfaceSet::MarkChanged),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("[macos-gpu-osr] RenderApp sub-app missing; GPU texture injection disabled");
            return;
        };

        render_app
            .init_resource::<ExtractedWebviewIoSurfaces>()
            .init_resource::<WebviewGpuSurfaces>()
            .init_resource::<LiveWebviewSurfaceIds>()
            .init_resource::<ImportedWebviewTextures>()
            .add_systems(
                ExtractSchedule,
                (extract_webview_iosurfaces, extract_live_webview_surface_ids),
            )
            // Clear last frame's imported transient textures before the node runs.
            .add_systems(
                Render,
                prepare_webview_gpu_surfaces.in_set(RenderSystems::PrepareResources),
            )
            // Inject the owned GPU texture into RenderAssets<GpuImage> in
            // `PrepareAssets`, ordered:
            //   - AFTER `prepare_assets::<GpuImage>` so our insert overwrites the
            //     black CPU placeholder GpuImage for the same AssetId;
            //   - BEFORE the material's bind-group build
            //     (`prepare_erased_assets::<MeshMaterial3d<…>>`, also in
            //     PrepareAssets) so the rebuilt bind group captures OUR texture's
            //     view instead of the placeholder's.
            // `inject` also get-or-creates the owned `WebviewGpuSurface` (from the
            // extracted IOSurface size) since the texture must exist here, before
            // the material prepare runs. The blit node fills that same texture.
            .add_systems(
                Render,
                inject_webview_gpu_images
                    .in_set(RenderSystems::PrepareAssets)
                    .after(prepare_assets::<GpuImage>)
                    // Standard mesh material: inject before its bind-group build so
                    // it captures our texture view. Custom materials
                    // (WebviewExtendedMaterial<E>) intentionally do NOT add a
                    // symmetric .before() — the at-most-1-frame warmup flash is the
                    // accepted trade-off (spec §6, Approach A).
                    .before(prepare_erased_assets::<MeshMaterial3d<WebviewExtendStandardMaterial>>)
                    // UI: bind group for PreparedUiMaterial is built during
                    // prepare_assets, so our injection must precede it.
                    .before(prepare_assets::<PreparedUiMaterial<WebviewUiMaterial>>),
            );

        // Add the blit node to the TOP-LEVEL render graph, ordered before the
        // camera driver (and therefore before the main passes that sample the
        // webview texture).
        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(WebviewBlitLabel, WebviewBlitNode);
        render_graph.add_node_edge(WebviewBlitLabel, bevy::render::graph::CameraDriverLabel);
    }
}

/// One webview's latest retained IOSurface paired with its material surface id.
///
/// The `surface` carries an owned +1 IOSurface use-count (a `RetainedIoSurface`,
/// which is `Send`/`Sync`), so it stays valid across the main→render world
/// handoff under pipelined rendering. Ownership flows: `Browsers` (drained) →
/// main world → render world → `WebviewBlitNode` (import) → held alive until the
/// frame is submitted, then dropped (released).
struct PendingIoSurface {
    id: AssetId<Image>,
    // Width/height are read from `surface.width`/`surface.height` (both `pub` on
    // `RetainedIoSurface`) at use sites, so we don't duplicate them here.
    surface: RetainedIoSurface,
}

/// Main-world store of the latest retained IOSurfaces drained this frame.
///
/// Wrapped in a `Mutex` so the `ExtractSchedule` system — which only gets
/// read-only access to the main world (`Extract<Res<_>>`, as `Extract` requires
/// `ReadOnlySystemParam`) — can still *move* the owned retains out into the
/// render world via `&Mutex` interior mutability.
#[derive(Resource, Default)]
struct PendingWebviewIoSurfaces(std::sync::Mutex<Vec<PendingIoSurface>>);

/// Render-world copy of the latest retained IOSurfaces to import + blit this frame.
#[derive(Resource, Default)]
struct ExtractedWebviewIoSurfaces(Vec<PendingIoSurface>);

/// Render-world store of the per-webview owned destination textures, keyed by
/// the material's surface `AssetId<Image>`.
#[derive(Resource, Default)]
struct WebviewGpuSurfaces(HashMap<AssetId<Image>, WebviewGpuSurface>);

/// Render-world set of the surface ids of all webviews that are currently live in
/// the main world. Filled each frame from the authoritative `WebviewSurface`
/// query (in `ExtractSchedule`) and used by `inject_webview_gpu_images` to prune
/// `WebviewGpuSurfaces` entries whose webview has despawned — otherwise the owned
/// GPU texture (~2.5 MB each) leaks and a dead `GpuImage` is re-injected forever.
#[derive(Resource, Default)]
struct LiveWebviewSurfaceIds(bevy::platform::collections::HashSet<AssetId<Image>>);

/// Render-world holder keeping each transient imported IOSurface texture alive
/// until the frame's command encoder is submitted (the imported MTLTexture
/// aliases the IOSurface; the blit copies *out* of it, so it must outlive the
/// submit). Wrapped in a `Mutex` so the render-graph node (which only gets
/// `&World`) can stash into it during `run`. Cleared at the start of the next
/// frame, by which point the previous frame's encoder has been submitted.
///
/// The `RetainedIoSurface` backing each import is held one frame longer in
/// `ExtractedWebviewIoSurfaces` (released when `extract` clears it next frame),
/// which is strictly after this holder is cleared — so the surface always
/// outlives the texture that aliases it.
#[derive(Resource, Default)]
struct ImportedWebviewTextures(std::sync::Mutex<Vec<Texture>>);

/// Main-world system: ensure every mesh webview material of type `M` has a
/// surface `Handle<Image>` bound AND a `WebviewSurface` tag, since the macOS
/// accelerated-paint path produces no CPU frames and never allocates one.
///
/// Idempotent: if the material's surface slot is already `Some` (e.g. a custom
/// material the user pre-populated), reuse that handle; otherwise allocate a
/// black BGRA placeholder and set it. Either way `WebviewSurface` is always
/// attached so the entity is collected (mirrors the CPU path's
/// `get_or_insert_with` + insert behavior).
pub(crate) fn allocate_webview_surfaces_for<M: WebviewSurfaceSlot>(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<M>>,
    webviews: Query<(Entity, &MeshMaterial3d<M>), Without<WebviewSurface>>,
) {
    for (entity, material_handle) in webviews.iter() {
        let Some(material) = materials.get_mut(material_handle.id()) else {
            continue;
        };
        let handle = match material.webview_surface_slot().clone() {
            Some(handle) => handle,
            None => {
                // Allocate a black BGRA placeholder image. The real pixels are
                // injected directly into RenderAssets<GpuImage> in the render world.
                let image = Image::new_fill(
                    Extent3d {
                        width: SURFACE_WIDTH,
                        height: SURFACE_HEIGHT,
                        depth_or_array_layers: 1,
                    },
                    bevy::render::render_resource::TextureDimension::D2,
                    &[0, 0, 0, 255],
                    TextureFormat::Bgra8UnormSrgb,
                    RenderAssetUsages::all(),
                );
                let handle = images.add(image);
                *material.webview_surface_slot() = Some(handle.clone());
                handle
            }
        };
        commands.entity(entity).insert(WebviewSurface(handle));
    }
}

/// Main-world system: drain each webview's latest retained IOSurface out of
/// `Browsers` (transferring the retain) and pair it with the material's surface
/// `AssetId<Image>`. Any retain whose webview has no allocated surface yet is
/// dropped (released) here.
fn collect_webview_iosurfaces(
    mut commands: Commands,
    browsers: NonSend<Browsers>,
    surfaces: Query<&WebviewSurface>,
    pending: ResMut<PendingWebviewIoSurfaces>,
) {
    let Ok(mut pending) = pending.0.lock() else {
        return;
    };
    // Drop any retains left over from a frame extract never consumed (e.g. before
    // the surface id was known); releasing them keeps use-counts balanced.
    pending.clear();
    for (entity, retained) in browsers.take_latest_webview_iosurfaces() {
        let Ok(surface) = surfaces.get(entity) else {
            // No surface id yet; dropping `retained` releases the IOSurface.
            continue;
        };
        // Keep the latest retained surface on the entity for on-demand alpha
        // hit-testing. `clone()` is a second, independent `CFRetain`; the original
        // retain still moves to the render path below. Overwriting the component
        // releases the previous frame's retain (latest-wins). On no-paint frames
        // `take_latest_webview_iosurfaces` yields nothing, so the component keeps
        // its prior (sticky) value.
        commands
            .entity(entity)
            .insert(WebviewIoSurface(retained.clone()));
        pending.push(PendingIoSurface {
            id: surface.0.id(),
            surface: retained,
        });
    }
}

/// Extract the pending retained IOSurfaces into the render world, moving the
/// retains across the world boundary (they are `Send`). `Extract` only grants
/// read-only main-world access, so we move ownership out through the `Mutex`.
fn extract_webview_iosurfaces(
    mut extracted: ResMut<ExtractedWebviewIoSurfaces>,
    pending: Extract<Res<PendingWebviewIoSurfaces>>,
) {
    // Any surfaces still in `extracted` from a frame the node didn't consume are
    // released here. Move this frame's drained retains into the render world.
    extracted.0.clear();
    if let Ok(mut pending) = pending.0.lock() {
        extracted.0.append(&mut pending);
    }
}

/// Extract the surface ids of all live webviews from the main world so the render
/// world can prune `WebviewGpuSurfaces` entries belonging to despawned webviews.
///
/// This is the authoritative live set: a live webview always has its
/// `WebviewSurface` in this query, so its id is always present, and a webview that
/// has despawned drops out — which is exactly the signal `inject_webview_gpu_images`
/// uses to release the leaked owned texture.
fn extract_live_webview_surface_ids(
    mut live: ResMut<LiveWebviewSurfaceIds>,
    surfaces: Extract<Query<&WebviewSurface>>,
) {
    live.0.clear();
    live.0.extend(surfaces.iter().map(|s| s.0.id()));
}

/// Render-world system (`PrepareResources`): release the previous frame's
/// transient imported textures (now safely submitted). The owned destination
/// textures are created/resized in `inject_webview_gpu_images` instead, because
/// they must exist in `PrepareAssets` (before the material bind group is built),
/// which runs before `PrepareResources`.
fn prepare_webview_gpu_surfaces(imported: Res<ImportedWebviewTextures>) {
    if let Ok(mut held) = imported.0.lock() {
        held.clear();
    }
}

/// Render-graph node: import each retained IOSurface into a transient wgpu texture
/// and record a blit into the frame's command encoder, targeting the owned
/// destination surface. Records only — Bevy submits the encoder once at frame end.
struct WebviewBlitNode;

impl Node for WebviewBlitNode {
    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let extracted = world.resource::<ExtractedWebviewIoSurfaces>();
        if extracted.0.is_empty() {
            return Ok(());
        }
        let surfaces = world.resource::<WebviewGpuSurfaces>();
        let imported_holder = world.resource::<ImportedWebviewTextures>();

        // The owned destination texture and the imported transient are both wgpu
        // resources; split the borrow so we can use the render device and encoder.
        let render_device = render_context.render_device().clone();
        let encoder = render_context.command_encoder();

        let mut imported_this_frame = Vec::new();
        for entry in &extracted.0 {
            let Some(surface) = surfaces.0.get(&entry.id) else {
                continue;
            };
            // Safety: `entry.surface` is a `RetainedIoSurface` holding a +1
            // use-count, so the IOSurface is alive for this import. The retain is
            // released only next frame (when `extract` clears
            // `ExtractedWebviewIoSurfaces`), strictly after this frame's blit is
            // submitted and after the imported texture below is dropped.
            let imported = unsafe {
                surface.import_and_blit(
                    &render_device,
                    encoder,
                    entry.surface.ptr(),
                    entry.surface.width,
                    entry.surface.height,
                )
            };
            match imported {
                Some(texture) => imported_this_frame.push(texture),
                None => {
                    bevy::log::error!(
                        "[macos-gpu-osr] IOSurface import failed ({}x{})",
                        entry.surface.width,
                        entry.surface.height
                    );
                }
            }
        }

        // Keep the imported transient textures alive until the encoder is
        // submitted (cleared next frame in `prepare_webview_gpu_surfaces`).
        if let Ok(mut held) = imported_holder.0.lock() {
            held.extend(imported_this_frame);
        }

        Ok(())
    }
}

/// Render-world system (`PrepareAssets`, after `prepare_assets::<GpuImage>` and
/// before the material's `prepare_erased_assets`): get-or-create the owned GPU
/// destination texture for each extracted webview surface id, wrap it in a
/// `GpuImage`, and overwrite the `RenderAssets<GpuImage>` entry for that id.
///
/// The owned texture is created here (not in `PrepareResources`) because it must
/// exist before the material bind group is built. The `WebviewBlitNode` (Render
/// phase) fills this same texture's contents from the IOSurface each frame.
fn inject_webview_gpu_images(
    extracted: Res<ExtractedWebviewIoSurfaces>,
    render_device: Res<RenderDevice>,
    mut surfaces: ResMut<WebviewGpuSurfaces>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    default_sampler: Res<DefaultImageSampler>,
    live: Res<LiveWebviewSurfaceIds>,
) {
    // Prune owned surfaces whose webview has despawned. We only drop ids that are
    // ABSENT from the authoritative live set, so a live webview (whose
    // `WebviewSurface` is always in the extract query) is never pruned. Dropping
    // the entry releases its ~2.5 MB owned GPU texture and stops re-injecting a
    // dead `GpuImage` every frame. (Bevy's `prepare_assets` already removes the
    // dead Image's `GpuImage` from `RenderAssets` on `AssetEvent::Removed`, so we
    // do not need to touch `gpu_images` for pruned ids.)
    surfaces.0.retain(|id, _| live.0.contains(id));

    // Ensure an owned destination texture exists at the right size for each id
    // that produced an IOSurface this frame.
    for entry in &extracted.0 {
        match surfaces.0.get_mut(&entry.id) {
            Some(surface) => {
                surface.ensure_size(&render_device, entry.surface.width, entry.surface.height);
            }
            None => {
                surfaces.0.insert(
                    entry.id,
                    WebviewGpuSurface::new(
                        &render_device,
                        entry.surface.width,
                        entry.surface.height,
                    ),
                );
            }
        }
    }

    // Re-inject ALL known owned surfaces every frame, even when no new IOSurface
    // arrived this frame. The owned texture is a stable buffer that retains the
    // last blitted page, so re-binding it keeps the webview sampling the live
    // page. This matters for sprites in particular: `mark_sprite_webview_images_changed`
    // fires `AssetEvent::Modified` every frame, so `prepare_assets::<GpuImage>`
    // re-uploads the black CPU placeholder every frame — if we skipped injection
    // on a no-paint frame, the placeholder would win and the sprite would go
    // black. (Mesh/UI never touch the Image, so they never re-upload the
    // placeholder, but re-injecting for them is harmless and idempotent.)
    if surfaces.0.is_empty() {
        return;
    }

    // Use Bevy's configured default image sampler (linear filtering via
    // `ImagePlugin::default()`), matching how normal `Image`s are sampled.
    // Creating our own `SamplerDescriptor::default()` would use NEAREST filtering,
    // which makes the minified webview texture look rough/aliased.
    let sampler = (**default_sampler).clone();

    for (id, surface) in surfaces.0.iter() {
        let gpu_image = GpuImage {
            texture: surface.texture.clone(),
            texture_view: surface.view.clone(),
            texture_format: TextureFormat::Bgra8UnormSrgb,
            texture_view_format: None,
            sampler: sampler.clone(),
            size: Extent3d {
                width: surface.width,
                height: surface.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            had_data: true,
        };

        gpu_images.insert(*id, gpu_image);
    }
}

/// Main-world system: touch every mesh webview material of type `M` each frame so
/// Bevy re-extracts and rebuilds its (otherwise cached) bind group, capturing the
/// freshly injected owned-texture view rather than the black placeholder.
pub(crate) fn mark_webview_materials_changed_for<M: WebviewSurfaceSlot>(
    webviews: Query<&MeshMaterial3d<M>>,
    mut materials: ResMut<Assets<M>>,
) {
    for handle in webviews.iter() {
        // `get_mut` flags the asset as modified → re-extract → bind group rebuild.
        let _ = materials.get_mut(handle.id());
    }
}

/// Main-world system: give every UI webview a placeholder `Handle<Image>`,
/// wire it into `WebviewUiMaterial.surface`, and insert `WebviewSurface`. The
/// macOS accelerated-paint path never fires `RenderTextureMessage`, so the
/// CPU-path `render_ui_surface` system never runs on macOS; we do its job here.
fn allocate_ui_webview_surfaces(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
    webviews: Query<(Entity, &MaterialNode<WebviewUiMaterial>), Without<WebviewSurface>>,
) {
    for (entity, material_handle) in webviews.iter() {
        let Some(material) = materials.get_mut(material_handle.id()) else {
            continue;
        };
        // Idempotent (matches the generic mesh path `allocate_webview_surfaces_for`):
        // reuse a pre-populated surface handle, otherwise allocate a black BGRA
        // placeholder. Either way always attach `WebviewSurface` so the entity is
        // collected — skipping when `surface` is already `Some` would leave a
        // pre-populated UI material uncollected and permanently black.
        let handle = match material.surface.clone() {
            Some(handle) => handle,
            None => {
                // The real pixels are injected directly into RenderAssets<GpuImage>
                // in the render world.
                let image = Image::new_fill(
                    Extent3d {
                        width: SURFACE_WIDTH,
                        height: SURFACE_HEIGHT,
                        depth_or_array_layers: 1,
                    },
                    bevy::render::render_resource::TextureDimension::D2,
                    &[0, 0, 0, 255],
                    TextureFormat::Bgra8UnormSrgb,
                    RenderAssetUsages::all(),
                );
                let handle = images.add(image);
                material.surface = Some(handle.clone());
                handle
            }
        };
        commands.entity(entity).insert(WebviewSurface(handle));
    }
}

/// Main-world system: touch every UI webview material each frame so Bevy
/// rebuilds the `PreparedUiMaterial` bind group (capturing the injected
/// owned-texture view rather than the black placeholder).
fn mark_webview_ui_materials_changed(
    webviews: Query<&MaterialNode<WebviewUiMaterial>>,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
) {
    for handle in webviews.iter() {
        let _ = materials.get_mut(handle.id());
    }
}

/// Main-world system: give every sprite webview a dedicated placeholder
/// `Handle<Image>`, point `Sprite.image` at it, and insert `WebviewSurface`.
///
/// Sprites have no material asset, so (unlike mesh/UI) there is nothing to write
/// a surface handle into — instead the sprite samples `Sprite.image` directly.
/// We allocate a fresh 800×800 BGRA placeholder (rather than reuse the example's
/// 1×1 `Image::default()`) so the injected owned texture and the placeholder
/// agree on size, avoiding any 1×1-vs-800×800 ambiguity in the sprite quad's UVs.
/// The real pixels are injected straight into `RenderAssets<GpuImage>` for this
/// id in the render world; the placeholder's CPU data is never sampled.
fn allocate_sprite_webview_surfaces(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut webviews: Query<(Entity, &mut Sprite), (With<WebviewSource>, Without<WebviewSurface>)>,
) {
    for (entity, mut sprite) in webviews.iter_mut() {
        let image = Image::new_fill(
            Extent3d {
                width: SURFACE_WIDTH,
                height: SURFACE_HEIGHT,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::all(),
        );
        let handle = images.add(image);

        sprite.image = handle.clone();
        commands
            .entity(entity)
            .insert(WebviewSurface(handle.clone()));
    }
}

/// Main-world system: touch every sprite webview's `Image` each frame to fire
/// `AssetEvent::Modified` for its id.
///
/// `bevy_sprite_render`'s `prepare_sprite_image_bind_groups` caches per-image
/// bind groups in a private `ImageBindGroups` map and only evicts an entry when
/// it sees an `AssetEvent::Modified { id }` for that image. Firing that event is
/// the only public lever to force a rebuild, so the rebuilt bind group samples
/// our freshly injected owned texture instead of the stale black placeholder.
///
/// The `Image` id used here is the sprite's image handle, which equals the
/// `WebviewSurface` id (allocated together above) — the same id injected into
/// `RenderAssets<GpuImage>` — so the eviction and our injection line up.
fn mark_sprite_webview_images_changed(
    // Restrict to sprite webviews: marking mesh/UI placeholder `Image`s modified
    // would make `prepare_assets::<GpuImage>` re-upload the black placeholder each
    // frame (a no-op thanks to our later injection, but pointless work the mesh/UI
    // paths deliberately avoid by using the material-changed lever instead).
    webviews: Query<&WebviewSurface, (With<WebviewSource>, With<Sprite>)>,
    mut images: ResMut<Assets<Image>>,
) {
    for surface in webviews.iter() {
        // `get_mut` flags the asset as modified → `AssetEvent::Modified` →
        // sprite bind group eviction + rebuild reading our injected GpuImage.
        let _ = images.get_mut(surface.0.id());
    }
}
