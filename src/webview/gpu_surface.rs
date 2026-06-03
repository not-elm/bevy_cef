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
    render_asset::RenderAssets,
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
            (allocate_webview_surfaces, collect_webview_iosurfaces).chain(),
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
            // Create/resize owned destination textures (and clear last frame's
            // imported transients) before the node runs, so the node only
            // reads them via `&World`.
            .add_systems(
                Render,
                prepare_webview_gpu_surfaces.in_set(RenderSystems::PrepareResources),
            )
            // Inject the owned textures into RenderAssets<GpuImage> before bind
            // groups are built (hardening: must run before PrepareBindGroups).
            .add_systems(
                Render,
                inject_webview_gpu_images.in_set(RenderSystems::PrepareAssets),
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

        info!(
            "[macos-gpu-osr] allocated surface {:?} for webview {:?}",
            handle.id(),
            entity
        );
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

/// Render-world system (`PrepareResources`): ensure an owned destination texture
/// exists at the right size for every extracted webview surface id, and clear the
/// previous frame's transient imported textures (now safely submitted).
fn prepare_webview_gpu_surfaces(
    extracted: Res<ExtractedWebviewIoSurfaces>,
    render_device: Res<RenderDevice>,
    mut surfaces: ResMut<WebviewGpuSurfaces>,
    imported: Res<ImportedWebviewTextures>,
) {
    // The previous frame's blit has been submitted by now; release the transient
    // imported textures.
    if let Ok(mut held) = imported.0.lock() {
        held.clear();
    }

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

/// Render-world system (`PrepareAssets`): wrap each owned webview GPU texture in a
/// `GpuImage` and overwrite the `RenderAssets<GpuImage>` entry for the webview's
/// surface id, so the material samples the live blitted contents.
fn inject_webview_gpu_images(
    surfaces: Res<WebviewGpuSurfaces>,
    render_device: Res<RenderDevice>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    mut sampler: Local<Option<Sampler>>,
    mut logged: Local<bool>,
) {
    if surfaces.0.is_empty() {
        return;
    }

    if !*logged {
        info!(
            "[macos-gpu-osr] render-world injection running for {} surface(s)",
            surfaces.0.len()
        );
        *logged = true;
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

// Reference WebviewMaterial so its type path stays linked even if the field
// access above is refactored; keeps the module self-documenting.
#[allow(dead_code)]
fn _assert_material_type(_m: &WebviewMaterial) {}
