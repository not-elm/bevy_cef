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
//! 1. Main world (`Update`): `allocate_webview_surfaces` gives every webview mesh
//!    material a placeholder `Handle<Image>` + `WebviewSurface` tag.
//! 2. Main world (`Update`, after allocation): `collect_webview_iosurfaces` pulls
//!    the latest retained IOSurface ptr per webview out of `Browsers` (`NonSend`)
//!    and pairs each with its surface `AssetId<Image>` into
//!    `PendingWebviewIoSurfaces`.
//! 3. `ExtractSchedule`: `extract_webview_iosurfaces` copies the pending list into
//!    the render world (`ExtractedWebviewIoSurfaces`).
//! 4. Render `PrepareResources`: `prepare_webview_gpu_surfaces` ensures an owned
//!    destination `WebviewGpuSurface` exists (at the right size) for each id, and
//!    clears the previous frame's imported transient textures.
//! 5. Render graph (`WebviewBlitNode`, before `CameraDriverLabel`): import each
//!    retained IOSurface into a transient wgpu texture and record a blit into the
//!    frame's command encoder targeting the owned surface. The transient textures
//!    are stashed so they outlive encoder submission.
//! 6. Render `PrepareAssets` (before `PrepareBindGroups`): `inject_webview_gpu_images`
//!    wraps each owned surface in a `GpuImage` and inserts it into
//!    `RenderAssets<GpuImage>` for the webview's surface id.
//!
//! The owned texture is a single stable buffer (MVP, no double-buffering): the
//! node blits into the same texture each frame, and the injected `GpuImage`
//! reuses the same `texture_view`, so the material bind group stays valid.

use crate::prelude::{WebviewExtendStandardMaterial, WebviewMaterial, WebviewSurface};
use bevy::asset::{AssetId, RenderAssetUsages};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::{
    Extract, Render, RenderApp, RenderSystems,
    erased_render_asset::prepare_erased_assets,
    render_asset::{RenderAssets, prepare_assets},
    render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
    render_resource::{Extent3d, Sampler, SamplerDescriptor, Texture, TextureFormat},
    renderer::{RenderContext, RenderDevice},
    texture::GpuImage,
};
use bevy_cef_core::prelude::{Browsers, RetainedIoSurface, WebviewGpuSurface};

const SURFACE_WIDTH: u32 = 800;
const SURFACE_HEIGHT: u32 = 800;

/// Render-graph label for the webview IOSurface import + blit node.
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct WebviewBlitLabel;

/// [macOS GPU OSR] plugin: import each webview's retained IOSurface in a custom
/// render-graph node and inject the owned GPU texture into the render world so
/// the webview mesh renders the real page.
pub struct WebviewGpuInjectPlugin;

impl Plugin for WebviewGpuInjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingWebviewIoSurfaces>().add_systems(
            Update,
            (
                allocate_webview_surfaces,
                collect_webview_iosurfaces,
                // Touch every webview material each frame so Bevy re-extracts and
                // rebuilds its bind group. The cached bind group captures the
                // texture view at build time, so without this it stays bound to
                // the black placeholder GpuImage forever (design §9 risk). We must
                // NOT touch the placeholder Image asset (that would make
                // `prepare_assets::<GpuImage>` re-upload the black placeholder and
                // clobber our injected GpuImage).
                mark_webview_materials_changed,
            )
                .chain(),
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("[macos-gpu-osr] RenderApp sub-app missing; GPU texture injection disabled");
            return;
        };

        render_app
            .init_resource::<ExtractedWebviewIoSurfaces>()
            .init_resource::<WebviewGpuSurfaces>()
            .init_resource::<ImportedWebviewTextures>()
            .add_systems(ExtractSchedule, extract_webview_iosurfaces)
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
                    .before(prepare_erased_assets::<MeshMaterial3d<WebviewExtendStandardMaterial>>),
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
    surface: RetainedIoSurface,
    width: u32,
    height: u32,
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

/// Main-world system: give every webview mesh material a surface `Handle<Image>`
/// to bind, since the macOS accelerated-paint path produces no CPU frames and
/// therefore never allocates one.
fn allocate_webview_surfaces(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
    webviews: Query<
        (Entity, &MeshMaterial3d<WebviewExtendStandardMaterial>),
        Without<WebviewSurface>,
    >,
) {
    for (entity, material_handle) in webviews.iter() {
        let Some(material) = materials.get_mut(material_handle.id()) else {
            continue;
        };
        if material.extension.surface.is_some() {
            continue;
        }

        // Allocate a black BGRA placeholder image. The real pixels are injected
        // directly into RenderAssets<GpuImage> in the render world.
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

        material.extension.surface = Some(handle.clone());
        commands
            .entity(entity)
            .insert(WebviewSurface(handle.clone()));
    }
}

/// Main-world system: drain each webview's latest retained IOSurface out of
/// `Browsers` (transferring the retain) and pair it with the material's surface
/// `AssetId<Image>`. Any retain whose webview has no allocated surface yet is
/// dropped (released) here.
fn collect_webview_iosurfaces(
    browsers: NonSend<Browsers>,
    surfaces: Query<(Entity, &WebviewSurface)>,
    pending: ResMut<PendingWebviewIoSurfaces>,
) {
    let Ok(mut pending) = pending.0.lock() else {
        return;
    };
    // Drop any retains left over from a frame extract never consumed (e.g. before
    // the surface id was known); releasing them keeps use-counts balanced.
    pending.clear();
    for (entity, retained) in browsers.take_latest_webview_iosurfaces() {
        let Some((_, surface)) = surfaces.iter().find(|(e, _)| *e == entity) else {
            // No surface id yet; dropping `retained` releases the IOSurface.
            continue;
        };
        let width = retained.width;
        let height = retained.height;
        pending.push(PendingIoSurface {
            id: surface.0.id(),
            surface: retained,
            width,
            height,
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
                    entry.width,
                    entry.height,
                )
            };
            match imported {
                Some(texture) => imported_this_frame.push(texture),
                None => {
                    bevy::log::error!(
                        "[macos-gpu-osr] IOSurface import failed ({}x{})",
                        entry.width,
                        entry.height
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
    mut sampler: Local<Option<Sampler>>,
) {
    if extracted.0.is_empty() {
        return;
    }

    // Ensure an owned destination texture exists at the right size for each id.
    for entry in &extracted.0 {
        match surfaces.0.get_mut(&entry.id) {
            Some(surface) => {
                surface.ensure_size(&render_device, entry.width, entry.height);
            }
            None => {
                surfaces.0.insert(
                    entry.id,
                    WebviewGpuSurface::new(&render_device, entry.width, entry.height),
                );
            }
        }
    }

    let sampler = sampler
        .get_or_insert_with(|| render_device.create_sampler(&SamplerDescriptor::default()))
        .clone();

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

/// Main-world system: touch every webview material each frame so Bevy
/// re-extracts and rebuilds its (otherwise cached) bind group, capturing the
/// freshly injected owned-texture view rather than the black placeholder.
fn mark_webview_materials_changed(
    webviews: Query<&MeshMaterial3d<WebviewExtendStandardMaterial>>,
    mut materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
) {
    for handle in webviews.iter() {
        // `get_mut` flags the asset as modified → re-extract → bind group rebuild.
        let _ = materials.get_mut(handle.id());
    }
}

// Reference WebviewMaterial so its type path stays linked even if the field
// access above is refactored; keeps the module self-documenting.
#[allow(dead_code)]
fn _assert_material_type(_m: &WebviewMaterial) {}
