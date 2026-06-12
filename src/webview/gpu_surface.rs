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
//!    placeholder `Handle<Image>` + `WebviewSurface` tag, and keep the material
//!    slot and the tag reconciled afterwards (material swaps re-wire instead of
//!    going permanently black). Mesh/UI store the handle on their material;
//!    sprites set it as `Sprite.image`. The collect → node → inject pipeline is
//!    keyed by `WebviewSurface`'s `AssetId<Image>`, so it is
//!    material/sprite-agnostic.
//! 2. Main world (`Update`, after allocation): `collect_webview_iosurfaces` pulls
//!    the latest retained IOSurface ptr per webview out of `Browsers` (`NonSend`)
//!    and pairs each with its surface `AssetId<Image>` into
//!    `PendingWebviewIoSurfaces`. It also schedules bind-group rebuilds
//!    (`WebviewSurfaceRebinds`) on first-frame / resize / re-key events, which
//!    the `mark_*` systems consume — webview materials are NOT dirtied every
//!    frame.
//! 3. `ExtractSchedule`: `extract_webview_iosurfaces` copies the pending list into
//!    the render world (`ExtractedWebviewIoSurfaces`), and
//!    `extract_live_webview_surface_ids` records the live webviews' surface ids
//!    (so step 4 can prune entries for despawned webviews).
//! 4. Render `PrepareAssets` (after `prepare_assets::<GpuImage>`, before the
//!    material bind-group build): `inject_webview_gpu_images` prunes despawned
//!    surfaces, get-or-creates the owned `WebviewGpuSurface` for each id (it must
//!    exist before the material bind group is built), wraps each owned surface in a
//!    `GpuImage`, and inserts it into `RenderAssets<GpuImage>` for the surface id.
//! 5. Render graph (`WebviewBlitNode`, before `CameraDriverLabel`): import each
//!    retained IOSurface into a transient wgpu texture and record a blit into the
//!    frame's command encoder, filling the owned surface created in step 4. The
//!    transient texture is dropped immediately — wgpu keeps recorded resources
//!    (and, via the MTLTexture's own IOSurface reference, the surface) alive
//!    until the submitted command buffer completes on the GPU.
//!
//! The owned texture is a single stable buffer (MVP, no double-buffering): the
//! node blits into the same texture each frame, and the injected `GpuImage`
//! reuses the same `texture_view`, so the material bind group stays valid
//! between rebind events.

use crate::common::{WebviewIoSurface, WebviewSize, WebviewSource};
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
    render_resource::{Extent3d, TextureFormat},
    renderer::{RenderContext, RenderDevice},
    texture::{DefaultImageSampler, GpuImage},
};
use bevy::ui_render::PreparedUiMaterial;
use bevy_cef_core::prelude::{Browsers, RetainedIoSurface, WebviewGpuSurface};

/// Number of consecutive main-world frames a rebind request stays active.
///
/// 2 = the frame the trigger was detected plus one echo frame. The echo is
/// required for custom mesh materials (`WebviewExtendedMaterial<E>`): their
/// bind-group build is deliberately unordered relative to
/// `inject_webview_gpu_images`, so on the trigger frame the rebuild may still
/// capture the placeholder view; on the echo frame the injected entry already
/// persists in `RenderAssets<GpuImage>`, making the rebuild order-independent.
const REBIND_FRAMES: u8 = 2;

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
    fn webview_surface_slot(&self) -> &Option<Handle<Image>>;
    fn webview_surface_slot_mut(&mut self) -> &mut Option<Handle<Image>>;
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
            .init_resource::<WebviewSurfaceRebinds>()
            .init_resource::<LastCollectedSurfaceIds>()
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
            .add_systems(
                ExtractSchedule,
                (extract_webview_iosurfaces, extract_live_webview_surface_ids),
            )
            // Ordering: AFTER `prepare_assets::<GpuImage>` so the insert overwrites
            // the black placeholder GpuImage for the same AssetId; BEFORE each
            // material's bind-group build so the rebuilt bind group captures the
            // injected view. Custom materials (`WebviewExtendedMaterial<E>`)
            // intentionally have NO `.before()` edge — the at-most-1-frame warmup
            // flash is the accepted trade-off (spec §6, Approach A); the
            // REBIND_FRAMES echo makes their rebuild order-independent.
            .add_systems(
                Render,
                inject_webview_gpu_images
                    .in_set(RenderSystems::PrepareAssets)
                    .after(prepare_assets::<GpuImage>)
                    .before(prepare_erased_assets::<MeshMaterial3d<WebviewExtendStandardMaterial>>)
                    .before(prepare_assets::<PreparedUiMaterial<WebviewUiMaterial>>),
            );

        // Before the camera driver, so the main passes sample the updated texture.
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
/// main world → render world → `WebviewBlitNode` (import), then released on the
/// next frame's extract (the Metal texture keeps its own IOSurface reference
/// until the submitted commands complete).
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

/// Main-world map of surface ids whose bind groups must be rebuilt, with the
/// number of remaining frames to keep marking (see [`REBIND_FRAMES`]).
///
/// Filled by `collect_webview_iosurfaces` when a webview receives its first
/// IOSurface, changes size, or is re-keyed to a new surface id; consumed by the
/// `mark_*` systems. This replaces dirtying every webview material/image every
/// frame, which forced a per-webview bind-group rebuild each frame and — for
/// sprites — a ~2.5 MB placeholder re-upload each frame.
#[derive(Resource, Default)]
pub(crate) struct WebviewSurfaceRebinds(HashMap<AssetId<Image>, u8>);

/// Main-world map of the surface id last pushed per webview entity, used to
/// detect re-keying (material swap → new surface handle) so the sticky
/// IOSurface can be re-pushed for the new id even when CEF delivers no new
/// frame (a static page never repaints under external begin-frames).
#[derive(Resource, Default)]
struct LastCollectedSurfaceIds(HashMap<Entity, AssetId<Image>>);

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

/// Black, fully-opaque BGRA placeholder for a webview surface `Image`. The real
/// pixels are injected directly into `RenderAssets<GpuImage>` in the render
/// world.
///
/// Mesh/UI surfaces use a 1×1 placeholder: its pixels are never sampled after
/// the first injection, and the pre-first-frame alpha fallback treats any black
/// pixel as opaque regardless of dimensions — so there is no reason to pin a
/// multi-megabyte CPU copy per webview. Sprites pass their `WebviewSize`: the
/// sprite picking backend maps cursor positions against the main-world image
/// dimensions, so the placeholder must agree with the rendered quad.
fn placeholder_surface_image(size: UVec2) -> Image {
    Image::new_fill(
        Extent3d {
            width: size.x.max(1),
            height: size.y.max(1),
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::all(),
    )
}

/// Main-world system: keep every mesh webview material of type `M` wired to a
/// surface `Handle<Image>` AND a matching `WebviewSurface` tag, since the macOS
/// accelerated-paint path produces no CPU frames and never allocates one.
///
/// Runs as a per-frame reconciliation (mirroring the CPU path's per-frame
/// `get_or_insert_with`), so a swapped or in-place-replaced material — whose
/// fresh `surface` slot is `None` — is re-wired instead of permanently binding
/// the `AsBindGroup` fallback texture. Reads are non-mutating; `Assets::get_mut`
/// (which flags the asset Modified) runs only when a write is needed.
pub(crate) fn allocate_webview_surfaces_for<M: WebviewSurfaceSlot>(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<M>>,
    webviews: Query<(Entity, &MeshMaterial3d<M>, Option<&WebviewSurface>), With<WebviewSource>>,
) {
    for (entity, material_handle, existing) in webviews.iter() {
        let Some(material) = materials.get(material_handle.id()) else {
            continue;
        };
        match (material.webview_surface_slot().clone(), existing) {
            (Some(handle), Some(surface)) if surface.0.id() == handle.id() => {}
            (Some(handle), _) => {
                commands.entity(entity).try_insert(WebviewSurface(handle));
            }
            (None, Some(surface)) => {
                if let Some(material) = materials.get_mut(material_handle.id()) {
                    *material.webview_surface_slot_mut() = Some(surface.0.clone());
                }
            }
            (None, None) => {
                let handle = images.add(placeholder_surface_image(UVec2::ONE));
                if let Some(material) = materials.get_mut(material_handle.id()) {
                    *material.webview_surface_slot_mut() = Some(handle.clone());
                }
                commands.entity(entity).try_insert(WebviewSurface(handle));
            }
        }
    }
}

/// Main-world system: drain each webview's latest retained IOSurface out of
/// `Browsers` (transferring the retain) and pair it with the material's surface
/// `AssetId<Image>`. Webviews without an allocated surface are NOT drained —
/// their frame stays in the browser-side latest-wins slot until the surface
/// exists (CEF never repaints an undamaged page, so dropping the only frame
/// would leave a static page black forever).
///
/// Also detects the events that require a bind-group rebuild — first frame,
/// IOSurface size change, surface re-key — and schedules them in
/// [`WebviewSurfaceRebinds`] for the `mark_*` systems.
fn collect_webview_iosurfaces(
    mut commands: Commands,
    mut rebinds: ResMut<WebviewSurfaceRebinds>,
    mut last_ids: ResMut<LastCollectedSurfaceIds>,
    mut webviews: Query<(Entity, &WebviewSurface, Option<&mut WebviewIoSurface>)>,
    browsers: NonSend<Browsers>,
    pending: ResMut<PendingWebviewIoSurfaces>,
) {
    rebinds.0.retain(|_, frames| {
        *frames -= 1;
        *frames > 0
    });
    last_ids.0.retain(|entity, _| webviews.contains(*entity));

    let Ok(mut pending) = pending.0.lock() else {
        return;
    };
    // Clearing releases (CFRelease) any retains a skipped extract never consumed.
    pending.clear();

    let mut new_frames: HashMap<Entity, RetainedIoSurface> = browsers
        .take_latest_webview_iosurfaces(|entity| webviews.contains(entity))
        .into_iter()
        .collect();

    for (entity, surface, io_surface) in webviews.iter_mut() {
        let id = surface.0.id();
        if let Some(retained) = new_frames.remove(&entity) {
            // The sticky component keeps an independent retain (`clone()` =
            // CFRetain) for alpha hit-testing; the original moves to the render
            // path. `try_insert`: a despawn may already be queued at this sync point.
            let needs_rebind = if let Some(mut io_surface) = io_surface {
                let resized =
                    io_surface.0.width != retained.width || io_surface.0.height != retained.height;
                io_surface.0 = retained.clone();
                resized
            } else {
                commands
                    .entity(entity)
                    .try_insert(WebviewIoSurface(retained.clone()));
                true
            };
            let rekeyed = last_ids.0.insert(entity, id) != Some(id);
            if needs_rebind || rekeyed {
                rebinds.0.insert(id, REBIND_FRAMES);
            }
            pending.push(PendingIoSurface {
                id,
                surface: retained,
            });
        } else if let Some(io_surface) = io_surface {
            // No new frame, but the surface id was re-keyed (e.g. material swap):
            // re-push the sticky surface so the new id gets pixels — CEF never
            // repaints a static page on its own.
            if last_ids.0.get(&entity) != Some(&id) {
                last_ids.0.insert(entity, id);
                rebinds.0.insert(id, REBIND_FRAMES);
                pending.push(PendingIoSurface {
                    id,
                    surface: io_surface.0.clone(),
                });
            }
        }
    }
}

/// Extract the pending retained IOSurfaces into the render world, moving the
/// retains across the world boundary (they are `Send`). `Extract` only grants
/// read-only main-world access, so we move ownership out through the `Mutex`.
fn extract_webview_iosurfaces(
    mut extracted: ResMut<ExtractedWebviewIoSurfaces>,
    pending: Extract<Res<PendingWebviewIoSurfaces>>,
) {
    // Releasing the previous frame's retains here is safe even though the GPU
    // may still be executing that frame's blit: the imported Metal texture holds
    // its own IOSurface reference, and wgpu keeps the recorded texture alive
    // until the submission completes.
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

        let render_device = render_context.render_device().clone();
        let encoder = render_context.command_encoder();

        for entry in &extracted.0 {
            let Some(surface) = surfaces.0.get(&entry.id) else {
                continue;
            };
            if !surface.import_and_blit(&render_device, encoder, &entry.surface) {
                bevy::log::error_once!(
                    "[macos-gpu-osr] IOSurface import failed ({}x{}); webview textures will \
                     not update (the macOS GPU OSR path requires the Metal wgpu backend)",
                    entry.surface.width,
                    entry.surface.height
                );
            }
        }

        Ok(())
    }
}

/// Render-world system (`PrepareAssets`, after `prepare_assets::<GpuImage>` and
/// before the material's `prepare_erased_assets`): get-or-create the owned GPU
/// destination texture for each extracted webview surface id, wrap it in a
/// `GpuImage`, and overwrite the `RenderAssets<GpuImage>` entry for that id.
///
/// The owned texture is created here (not later in the frame) because it must
/// exist before the material bind group is built. The `WebviewBlitNode` (render
/// graph) fills this same texture's contents from the IOSurface each frame.
fn inject_webview_gpu_images(
    extracted: Res<ExtractedWebviewIoSurfaces>,
    render_device: Res<RenderDevice>,
    mut surfaces: ResMut<WebviewGpuSurfaces>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    default_sampler: Res<DefaultImageSampler>,
    live: Res<LiveWebviewSurfaceIds>,
) {
    // Dropping a pruned entry releases its owned GPU texture; the matching
    // `RenderAssets` entry is removed by Bevy itself on `AssetEvent::Unused`.
    surfaces.0.retain(|id, _| live.0.contains(id));

    for entry in &extracted.0 {
        // A webview can despawn after collect ran this frame; creating a texture
        // and blitting for a dead id would be wasted work.
        if !live.0.contains(&entry.id) {
            continue;
        }
        surfaces
            .0
            .entry(entry.id)
            .or_insert_with(|| {
                WebviewGpuSurface::new(&render_device, entry.surface.width, entry.surface.height)
            })
            .ensure_size(&render_device, entry.surface.width, entry.surface.height);
    }

    if surfaces.0.is_empty() {
        return;
    }

    // The re-insert below must stay every-frame (not changed-only):
    // `prepare_assets::<GpuImage>` can re-prepare the CPU placeholder on
    // event-less frames (e.g. an upload deferred by `RenderAssetBytesPerFrame`),
    // and this insert is what guarantees the owned texture wins.
    //
    // Bevy's default image sampler is linear; building a fresh
    // `SamplerDescriptor::default()` here would be NEAREST and look aliased.
    let sampler = (**default_sampler).clone();

    for (id, surface) in surfaces.0.iter() {
        let gpu_image = GpuImage {
            texture: surface.texture.clone(),
            texture_view: surface.view.clone(),
            texture_format: TextureFormat::Bgra8UnormSrgb,
            texture_view_format: None,
            sampler: sampler.clone(),
            size: surface.texture.size(),
            mip_level_count: 1,
            had_data: true,
        };

        gpu_images.insert(*id, gpu_image);
    }
}

/// Main-world system: touch a mesh webview material of type `M` on rebind frames
/// (see [`WebviewSurfaceRebinds`]) so Bevy re-extracts and rebuilds its bind
/// group, capturing the freshly injected owned-texture view rather than the
/// black placeholder. No-op on steady-state frames.
pub(crate) fn mark_webview_materials_changed_for<M: WebviewSurfaceSlot>(
    mut materials: ResMut<Assets<M>>,
    rebinds: Res<WebviewSurfaceRebinds>,
    webviews: Query<(&MeshMaterial3d<M>, &WebviewSurface), With<WebviewSource>>,
) {
    if rebinds.0.is_empty() {
        return;
    }
    for (handle, surface) in webviews.iter() {
        if rebinds.0.contains_key(&surface.0.id()) {
            let _ = materials.get_mut(handle.id());
        }
    }
}

/// Main-world system: keep every UI webview's `WebviewUiMaterial.surface` wired
/// to a placeholder `Handle<Image>` and a matching `WebviewSurface` tag. The
/// macOS accelerated-paint path never fires `RenderTextureMessage`, so the
/// CPU-path `render_ui_surface` system never runs on macOS; we do its job here.
///
/// Mirrors `allocate_webview_surfaces_for`'s per-frame reconciliation (material
/// swaps re-wire instead of going permanently black); reads are non-mutating and
/// `get_mut` runs only when a write is needed.
fn allocate_ui_webview_surfaces(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
    webviews: Query<
        (
            Entity,
            &MaterialNode<WebviewUiMaterial>,
            Option<&WebviewSurface>,
        ),
        With<WebviewSource>,
    >,
) {
    for (entity, material_handle, existing) in webviews.iter() {
        let Some(material) = materials.get(material_handle.id()) else {
            continue;
        };
        match (material.surface.clone(), existing) {
            (Some(handle), Some(surface)) if surface.0.id() == handle.id() => {}
            (Some(handle), _) => {
                commands.entity(entity).try_insert(WebviewSurface(handle));
            }
            (None, Some(surface)) => {
                if let Some(material) = materials.get_mut(material_handle.id()) {
                    material.surface = Some(surface.0.clone());
                }
            }
            (None, None) => {
                let handle = images.add(placeholder_surface_image(UVec2::ONE));
                if let Some(material) = materials.get_mut(material_handle.id()) {
                    material.surface = Some(handle.clone());
                }
                commands.entity(entity).try_insert(WebviewSurface(handle));
            }
        }
    }
}

/// Main-world system: touch a UI webview material on rebind frames so Bevy
/// rebuilds the `PreparedUiMaterial` bind group (capturing the injected
/// owned-texture view rather than the black placeholder).
fn mark_webview_ui_materials_changed(
    webviews: Query<(&MaterialNode<WebviewUiMaterial>, &WebviewSurface), With<WebviewSource>>,
    rebinds: Res<WebviewSurfaceRebinds>,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
) {
    if rebinds.0.is_empty() {
        return;
    }
    for (handle, surface) in webviews.iter() {
        if rebinds.0.contains_key(&surface.0.id()) {
            let _ = materials.get_mut(handle.id());
        }
    }
}

/// Main-world system: give every sprite webview a placeholder surface `Image`,
/// point `Sprite.image` at it, and insert `WebviewSurface`.
///
/// Sprites have no material asset, so (unlike mesh/UI) there is nothing to write
/// a surface handle into — the sprite samples `Sprite.image` directly. The
/// placeholder is sized from the entity's `WebviewSize` so the pre-first-frame
/// quad and the picking backend's pixel-space mapping agree with the injected
/// texture.
///
/// When the user supplied their own `Sprite.image` asset, the placeholder is
/// written INTO that asset (preserving the handle) — on the CPU path the webview
/// pixels were written into that very asset, so other handles to it stayed live;
/// orphaning it with a fresh handle would silently break that. `Handle::default()`
/// is shared by every defaulted `Sprite` and must never be overwritten, so a
/// dedicated image is allocated in that case.
fn allocate_sprite_webview_surfaces(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut webviews: Query<
        (Entity, &mut Sprite, &WebviewSize),
        (With<WebviewSource>, Without<WebviewSurface>),
    >,
) {
    for (entity, mut sprite, size) in webviews.iter_mut() {
        let placeholder = placeholder_surface_image(size.0.as_uvec2());
        let handle = if sprite.image != Handle::default() && images.get(sprite.image.id()).is_some()
        {
            let _ = images.insert(sprite.image.id(), placeholder);
            sprite.image.clone()
        } else {
            let handle = images.add(placeholder);
            sprite.image = handle.clone();
            handle
        };
        commands.entity(entity).try_insert(WebviewSurface(handle));
    }
}

/// Main-world system: touch a sprite webview's `Image` on rebind frames to fire
/// `AssetEvent::Modified` for its id.
///
/// `bevy_sprite_render`'s `prepare_sprite_image_bind_groups` caches per-image
/// bind groups in a private `ImageBindGroups` map and only evicts an entry when
/// it sees an `AssetEvent::Modified { id }` for that image. Firing that event is
/// the only public lever to force a rebuild, so the rebuilt bind group samples
/// our freshly injected owned texture instead of the stale black placeholder.
/// (The Modified event also makes `prepare_assets::<GpuImage>` re-upload the CPU
/// placeholder that frame — harmless, since injection overwrites it — which is
/// exactly why this only fires on rebind frames instead of every frame.)
fn mark_sprite_webview_images_changed(
    webviews: Query<&WebviewSurface, (With<WebviewSource>, With<Sprite>)>,
    rebinds: Res<WebviewSurfaceRebinds>,
    mut images: ResMut<Assets<Image>>,
) {
    if rebinds.0.is_empty() {
        return;
    }
    for surface in webviews.iter() {
        if rebinds.0.contains_key(&surface.0.id()) {
            let _ = images.get_mut(surface.0.id());
        }
    }
}
