//! [macOS GPU OSR — Approach 2] Import each webview's retained IOSurface and
//! blit it into an owned GPU texture via the `webview_blit` system (`RenderGraph`
//! schedule), then inject that owned texture into `RenderAssets<GpuImage>` so the
//! webview mesh material samples the live page.
//!
//! Why deferred to the `webview_blit` system (not the `on_accelerated_paint`
//! callback): Bevy owns ordered command submission — its render graph collects all
//! GPU commands into one `RenderContext` and submits them once per frame, then
//! presents. Doing an out-of-band `queue.submit` from the CEF callback (which runs
//! in the `Main` schedule) races that ordered submit/present and corrupts rendering
//! (the mesh goes black, no validation error). So the callback only *retains* the
//! latest IOSurface; the import + blit is recorded into the frame's command encoder
//! by the `webview_blit` system (`RenderGraph` schedule, Begin set), and Bevy
//! submits it in order.
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
//!    `PendingWebviewIoSurfaces`. It also schedules bind-group rebuilds by
//!    tagging the entity with the `WebviewSurfaceRebind` marker on first-frame /
//!    resize / re-key events, which the `mark_*` systems consume via a
//!    `With<WebviewSurfaceRebind>` filter — webview materials are NOT dirtied
//!    every frame.
//! 3. `ExtractSchedule`: `extract_webview_iosurfaces` copies the pending list into
//!    the render world (`ExtractedWebviewIoSurfaces`), and
//!    `extract_live_webview_surface_ids` records the live webviews' surface ids
//!    (so step 4 can prune entries for despawned webviews).
//! 4. Render `PrepareAssets` (after `prepare_assets::<GpuImage>`, before the
//!    material bind-group build): `inject_webview_gpu_images` prunes despawned
//!    surfaces, get-or-creates the owned `WebviewGpuSurface` for each id (it must
//!    exist before the material bind group is built), wraps each owned surface in a
//!    `GpuImage`, and inserts it into `RenderAssets<GpuImage>` for the surface id.
//! 5. `RenderGraph` schedule (`webview_blit` system, `RenderGraphSystems::Begin`):
//!    import each retained IOSurface into a transient wgpu texture and record a blit
//!    into the frame's command encoder, filling the owned surface created in step 4.
//!    The transient texture is dropped immediately — wgpu keeps recorded resources
//!    (and, via the MTLTexture's own IOSurface reference, the surface) alive until
//!    the submitted command buffer completes on the GPU.
//!
//! The owned texture is a single stable buffer (MVP, no double-buffering): the
//! `webview_blit` system blits into the same texture each frame, and the injected
//! `GpuImage` reuses the same `texture_view`, so the material bind group stays
//! valid between rebind events.

use crate::common::{WebviewIoSurface, WebviewSize, WebviewSource, WebviewTextureTarget};
use crate::prelude::{WebviewExtendStandardMaterial, WebviewSurface};
use crate::webview::texture_target::{WebviewGpuImageInjectSet, WebviewTextureSlot};
use crate::webview::ui::WebviewUiMaterial;
use bevy::asset::{Asset, AssetId, RenderAssetUsages};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::render::{
    Extract, Render, RenderApp, RenderSystems,
    erased_render_asset::prepare_erased_assets,
    render_asset::{RenderAssets, prepare_assets},
    render_resource::{Extent3d, TextureDescriptor, TextureFormat},
    renderer::{RenderContext, RenderDevice, RenderGraph, RenderGraphSystems},
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

/// [macOS GPU OSR] plugin: import each webview's retained IOSurface via the
/// `webview_blit` system (`RenderGraph` schedule) and inject the owned GPU
/// texture into the render world so the webview mesh renders the real page.
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
                    allocate_target_webview_surfaces,
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
                    mark_target_webview_images_changed,
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
            // injected view. Custom MESH materials (`WebviewExtendedMaterial<E>`)
            // intentionally have NO `.before()` edge — their rebind path touches
            // only the material (never the image), so on the echo frame the
            // injected entry persists in `RenderAssets<GpuImage>` and the rebuild
            // is order-independent (at-most-1-frame warmup flash, spec §6).
            // Headless targets are DIFFERENT: their rebind path touches the image,
            // which re-uploads the CPU placeholder in the same frames the
            // consumer's bind group rebuilds — so consumers MUST order after
            // `WebviewGpuImageInjectSet` (the turnkey plugin does it per material
            // type) or they can capture the placeholder permanently.
            .add_systems(
                Render,
                inject_webview_gpu_images
                    .in_set(RenderSystems::PrepareAssets)
                    .in_set(WebviewGpuImageInjectSet)
                    .after(prepare_assets::<GpuImage>)
                    .before(prepare_erased_assets::<MeshMaterial3d<WebviewExtendStandardMaterial>>)
                    .before(prepare_assets::<PreparedUiMaterial<WebviewUiMaterial>>),
            );

        // `RenderGraphSystems::Begin` is chained before `Render` (where
        // `camera_driver` runs), and encoder recording order == submission
        // order, so the blit lands before every camera pass samples the
        // texture. Ordering against the concrete `camera_driver` fn would work
        // too but couples us to bevy_core_pipeline internals (spec §3).
        render_app.add_systems(RenderGraph, webview_blit.in_set(RenderGraphSystems::Begin));
    }
}

/// One webview's latest retained IOSurface paired with its material surface id.
///
/// The `surface` carries an owned +1 IOSurface use-count (a `RetainedIoSurface`,
/// which is `Send`/`Sync`), so it stays valid across the main→render world
/// handoff under pipelined rendering. Ownership flows: `Browsers` (drained) →
/// main world → render world → `webview_blit` (import), then released on the
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

/// Marker: this webview's bind group must be rebuilt for the next
/// [`REBIND_FRAMES`] frames.
///
/// Attached by `collect_webview_iosurfaces` when the webview receives its first
/// IOSurface, changes size, or is re-keyed to a new surface id; removed by the
/// same system once `frames_left` runs out. The `mark_*` systems filter on
/// `With<Self>`, so steady-state frames iterate zero archetypes. This replaces
/// dirtying every webview material/image every frame, which forced a
/// per-webview bind-group rebuild each frame and — for sprites — a ~2.5 MB
/// placeholder re-upload each frame.
///
/// Known degenerate gap: when two webviews share one material handle and the
/// triggering entity despawns within the echo window, the surviving entity's
/// echo mark is lost (the marker dies with the entity).
#[derive(Component)]
pub(crate) struct WebviewSurfaceRebind {
    frames_left: u8,
}

/// The surface id last pushed for this webview, used to detect re-keying
/// (material swap → new surface handle) so the sticky IOSurface can be
/// re-pushed for the new id even when CEF delivers no new frame (a static page
/// never repaints under external begin-frames).
#[derive(Component)]
struct CollectedSurfaceId(AssetId<Image>);

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
struct LiveWebviewSurfaceIds(HashSet<AssetId<Image>>);

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
    webviews: Query<
        (Entity, &MeshMaterial3d<M>, Option<&WebviewSurface>),
        (With<WebviewSource>, Without<WebviewTextureTarget>),
    >,
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
                if let Some(mut material) = materials.get_mut(material_handle.id()) {
                    *material.webview_surface_slot_mut() = Some(surface.0.clone());
                }
            }
            (None, None) => {
                let handle = images.add(placeholder_surface_image(UVec2::ONE));
                if let Some(mut material) = materials.get_mut(material_handle.id()) {
                    *material.webview_surface_slot_mut() = Some(handle.clone());
                }
                commands.entity(entity).try_insert(WebviewSurface(handle));
            }
        }
    }
}

/// Main-world system: keep every headless webview (one carrying
/// [`WebviewTextureTarget`]) wired to a `WebviewSurface` keyed by the
/// user-supplied handle. Writes the canonical placeholder INTO the user's
/// asset (preserving the handle, like the sprite path) so the image has the
/// pipeline's required format/usages before the first injected frame.
///
/// Per-frame reconciliation like the other allocate paths: a handle swap on
/// the pub field shows up as an id mismatch and re-keys the surface (the
/// existing `CollectedSurfaceId` machinery re-pushes the sticky IOSurface for
/// the new id without waiting for a CEF repaint).
fn allocate_target_webview_surfaces(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    webviews: Query<(Entity, &WebviewTextureTarget, Option<&WebviewSurface>), With<WebviewSource>>,
    changed: Query<(), Changed<WebviewTextureTarget>>,
    // `warned` is shared by the stale-handle warning and the shared-handle
    // warning below. An id that already fired as stale will not re-fire as
    // shared (and vice versa) — one diagnostic signal per misconfigured id is
    // intentional; re-warning would be noisy with no new information.
    mut warned: Local<HashSet<AssetId<Image>>>,
) {
    for (entity, target, existing) in webviews.iter() {
        if target.0 == Handle::default() {
            bevy::log::warn_once!(
                "[bevy_cef] WebviewTextureTarget holds Handle::default(); create a \
                 dedicated image with `images.add(Image::default())` instead"
            );
            continue;
        }
        let id = target.0.id();
        if existing.is_none_or(|surface| surface.0.id() != id) {
            if let Err(err) = images.insert(id, placeholder_surface_image(UVec2::ONE)) {
                if warned.insert(id) {
                    warn!(
                        "[bevy_cef] WebviewTextureTarget handle is stale; surface not \
                         allocated for {entity}: {err}"
                    );
                }
                continue;
            }
            commands
                .entity(entity)
                .try_insert(WebviewSurface(target.0.clone()));
        }
    }

    // Shared-handle detection: two webviews blitting one asset id is
    // last-blit-wins. Scan only on frames where a target was added/changed
    // (`Changed` includes `Added`), and warn once per distinct id via the
    // `Local` set — each conflicting id is recorded once, ever (a conflict
    // resolved and later re-introduced on the same id will not re-warn).
    if !changed.is_empty() {
        let mut seen: HashMap<AssetId<Image>, Entity> = HashMap::default();
        for (entity, target, _) in webviews.iter() {
            if target.0 == Handle::default() {
                continue;
            }
            if let Some(first) = seen.insert(target.0.id(), entity)
                && warned.insert(target.0.id())
            {
                warn!(
                    "[bevy_cef] WebviewTextureTarget handle shared by {first} and \
                     {entity}; only one webview's frames will be visible (last blit wins)"
                );
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
/// IOSurface size change, surface re-key — and tags the entity with
/// [`WebviewSurfaceRebind`] for the `mark_*` systems.
fn collect_webview_iosurfaces(
    mut commands: Commands,
    mut rebinds: Query<(Entity, &mut WebviewSurfaceRebind)>,
    mut webviews: Query<(
        Entity,
        &WebviewSurface,
        Option<&mut WebviewIoSurface>,
        Option<&mut CollectedSurfaceId>,
    )>,
    browsers: NonSend<Browsers>,
    pending: ResMut<PendingWebviewIoSurfaces>,
) {
    // Decrement BEFORE any trigger inserts below: the same command queue applies
    // FIFO, so a `try_remove` queued here followed by a re-trigger's `try_insert`
    // leaves the marker present with a fresh count.
    for (entity, mut rebind) in rebinds.iter_mut() {
        rebind.frames_left = rebind.frames_left.saturating_sub(1);
        if rebind.frames_left == 0 {
            commands.entity(entity).try_remove::<WebviewSurfaceRebind>();
        }
    }

    let Ok(mut pending) = pending.0.lock() else {
        return;
    };
    // Clearing releases (CFRelease) any retains a skipped extract never consumed.
    pending.clear();

    let mut new_frames: HashMap<Entity, RetainedIoSurface> = browsers
        .take_latest_webview_iosurfaces(|entity| webviews.contains(entity))
        .into_iter()
        .collect();

    for (entity, surface, io_surface, collected_id) in webviews.iter_mut() {
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
            let rekeyed = if let Some(mut collected_id) = collected_id {
                let rekeyed = collected_id.0 != id;
                collected_id.0 = id;
                rekeyed
            } else {
                commands.entity(entity).try_insert(CollectedSurfaceId(id));
                true
            };
            if needs_rebind || rekeyed {
                commands.entity(entity).try_insert(WebviewSurfaceRebind {
                    frames_left: REBIND_FRAMES,
                });
            }
            pending.push(PendingIoSurface {
                id,
                surface: retained,
            });
        } else if let (Some(io_surface), Some(mut collected_id)) = (io_surface, collected_id) {
            // No new frame, but the surface id was re-keyed (e.g. material swap):
            // re-push the sticky surface so the new id gets pixels — CEF never
            // repaints a static page on its own.
            if collected_id.0 != id {
                collected_id.0 = id;
                commands.entity(entity).try_insert(WebviewSurfaceRebind {
                    frames_left: REBIND_FRAMES,
                });
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

/// Render-graph-schedule system (`RenderGraphSystems::Begin`): import each
/// retained IOSurface into a transient wgpu texture and record a blit into the
/// frame's command encoder, targeting the owned destination surface. Records
/// only — Bevy submits the recorded commands in `RenderGraphSystems::Submit`.
fn webview_blit(
    mut render_context: RenderContext,
    extracted: Res<ExtractedWebviewIoSurfaces>,
    surfaces: Res<WebviewGpuSurfaces>,
) {
    if extracted.0.is_empty() {
        return;
    }

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
}

/// Render-world system (`PrepareAssets`, after `prepare_assets::<GpuImage>` and
/// before the material's `prepare_erased_assets`): get-or-create the owned GPU
/// destination texture for each extracted webview surface id, wrap it in a
/// `GpuImage`, and overwrite the `RenderAssets<GpuImage>` entry for that id.
///
/// The owned texture is created here (not later in the frame) because it must
/// exist before the material bind group is built. The `webview_blit`
/// render-graph-schedule system fills this same texture's contents from the
/// IOSurface each frame.
fn inject_webview_gpu_images(
    mut surfaces: ResMut<WebviewGpuSurfaces>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    extracted: Res<ExtractedWebviewIoSurfaces>,
    render_device: Res<RenderDevice>,
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
            sampler: sampler.clone(),
            // Derived from the live texture so this can never drift from the
            // descriptor `WebviewGpuSurface::new` (bevy_cef_core) actually used.
            // `label`/`view_formats` have no accessors and stay literal — the
            // surface is created with no extra view formats.
            texture_descriptor: TextureDescriptor {
                label: None,
                size: surface.texture.size(),
                mip_level_count: surface.texture.mip_level_count(),
                sample_count: surface.texture.sample_count(),
                dimension: surface.texture.dimension(),
                format: surface.texture.format(),
                usage: surface.texture.usage(),
                view_formats: &[],
            },
            texture_view_descriptor: None,
            had_data: true,
        };

        gpu_images.insert(*id, gpu_image);
    }
}

/// Force an `AssetEvent::Modified` for `id` without changing the asset.
///
/// Bevy 0.19's `Assets::get_mut` returns an `AssetMut` guard that only queues
/// `Modified` if actually deref-mutated; a bare `let _ = assets.get_mut(id);`
/// silently emits nothing. Every rebind-trick site below MUST go through this
/// helper so the intent stays greppable and cannot silently regress.
fn touch_asset<A: Asset>(assets: &mut Assets<A>, id: impl Into<AssetId<A>>) {
    if let Some(asset) = assets.get_mut(id) {
        asset.into_inner();
    }
}

/// Main-world system: touch a mesh webview material of type `M` on rebind frames
/// (see [`WebviewSurfaceRebind`]) so Bevy re-extracts and rebuilds its bind
/// group, capturing the freshly injected owned-texture view rather than the
/// black placeholder. On steady-state frames no entity carries the marker, so
/// the query matches zero archetypes.
pub(crate) fn mark_webview_materials_changed_for<M: WebviewSurfaceSlot>(
    mut materials: ResMut<Assets<M>>,
    webviews: Query<&MeshMaterial3d<M>, (With<WebviewSource>, With<WebviewSurfaceRebind>)>,
) {
    for handle in webviews.iter() {
        touch_asset(&mut materials, handle.id());
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
        (With<WebviewSource>, Without<WebviewTextureTarget>),
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
                if let Some(mut material) = materials.get_mut(material_handle.id()) {
                    material.surface = Some(surface.0.clone());
                }
            }
            (None, None) => {
                let handle = images.add(placeholder_surface_image(UVec2::ONE));
                if let Some(mut material) = materials.get_mut(material_handle.id()) {
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
    webviews: Query<
        &MaterialNode<WebviewUiMaterial>,
        (With<WebviewSource>, With<WebviewSurfaceRebind>),
    >,
    mut materials: ResMut<Assets<WebviewUiMaterial>>,
) {
    for handle in webviews.iter() {
        touch_asset(&mut materials, handle.id());
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
        (
            With<WebviewSource>,
            Without<WebviewSurface>,
            Without<WebviewTextureTarget>,
        ),
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
    webviews: Query<
        &WebviewSurface,
        (
            With<WebviewSource>,
            With<Sprite>,
            With<WebviewSurfaceRebind>,
        ),
    >,
    mut images: ResMut<Assets<Image>>,
) {
    for surface in webviews.iter() {
        touch_asset(&mut images, surface.0.id());
    }
}

/// Main-world system: touch a headless webview's target `Image` on rebind
/// frames (see [`WebviewSurfaceRebind`]) so `AssetEvent::Modified` fires for
/// its id. This is the PUBLIC rebind signal for third-party consumers: Bevy
/// only rebuilds a material's bind group on the *material's own* asset events,
/// never on a referenced image's, so a consumer must `get_mut` its material
/// when this event fires (or use `WebviewTargetUiMaterialPlugin`). Mirrors
/// `mark_sprite_webview_images_changed`; the CPU placeholder re-upload this
/// triggers is harmless — injection overwrites it the same frame.
fn mark_target_webview_images_changed(
    webviews: Query<
        &WebviewSurface,
        (
            With<WebviewSource>,
            With<WebviewTextureTarget>,
            With<WebviewSurfaceRebind>,
        ),
    >,
    mut images: ResMut<Assets<Image>>,
) {
    for surface in webviews.iter() {
        touch_asset(&mut images, surface.0.id());
    }
}

/// Main-world system: touch every third-party asset of type `M` that
/// references a rebinding headless webview target (see
/// `crate::webview::texture_target::WebviewTextureSlot`), so Bevy rebuilds its
/// bind group against the freshly injected texture. Registered per material
/// type by `WebviewTargetUiMaterialPlugin`.
pub(crate) fn mark_target_materials_changed_for<M: WebviewTextureSlot>(
    rebinding: Query<&WebviewSurface, (With<WebviewTextureTarget>, With<WebviewSurfaceRebind>)>,
    mut materials: ResMut<Assets<M>>,
) {
    let rebind_ids: HashSet<AssetId<Image>> =
        rebinding.iter().map(|surface| surface.0.id()).collect();
    if rebind_ids.is_empty() {
        return;
    }
    // Two-phase on purpose: `iter_mut` would flag every `M` asset Modified; a
    // read-only scan plus targeted `get_mut` touches only the matches. The
    // linear scan is fine — rebind frames are rare and material asset counts
    // are small (a reverse index was considered and rejected in the spec).
    let to_touch: Vec<AssetId<M>> = materials
        .iter()
        .filter(|(_, material)| {
            material
                .webview_targets()
                .any(|target| rebind_ids.contains(&target))
        })
        .map(|(id, _)| id)
        .collect();
    for id in to_touch {
        touch_asset(&mut materials, id);
    }
}
