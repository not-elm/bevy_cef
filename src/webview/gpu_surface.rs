//! [macOS GPU OSR] Inject the owned CEF webview GPU texture into Bevy's
//! `RenderAssets<GpuImage>` so the webview mesh material samples the live page.
//!
//! On macOS the accelerated-paint path produces no CPU frames, so the standard
//! material never gets a `Handle<Image>` to bind. We:
//!
//! 1. Main world (`Update`): allocate a placeholder `Handle<Image>` surface for
//!    each webview mesh material that lacks one, point the material's
//!    `extension.surface` at it, and tag the entity with `WebviewSurface`
//!    (`allocate_webview_surfaces`).
//! 2. Main world (`Update`, after allocation): pull the owned GPU surface
//!    textures out of `Browsers` (`NonSend`) and pair each with its webview's
//!    surface `AssetId<Image>` into `PendingWebviewGpuTextures`
//!    (`collect_webview_gpu_textures`).
//! 3. `ExtractSchedule`: copy the pending payloads into the render world
//!    (`extract_webview_gpu_textures`).
//! 4. `Render` (after `prepare_assets::<GpuImage>`): build a `GpuImage` wrapping
//!    the owned texture/view and `insert` it into `RenderAssets<GpuImage>` for
//!    that id, overwriting whatever `prepare_assets` produced from the
//!    placeholder CPU image (`inject_webview_gpu_images`).
//!
//! The owned texture is a single stable buffer that `on_accelerated_paint`
//! blits each CEF frame into, so no per-frame texture re-creation is needed —
//! re-registering the same texture keeps the bound contents fresh.

use crate::prelude::{WebviewExtendStandardMaterial, WebviewMaterial, WebviewSurface};
use bevy::asset::{AssetId, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::{
    Extract, Render, RenderApp,
    render_asset::{RenderAssets, prepare_assets},
    render_resource::{
        Extent3d, Sampler, SamplerDescriptor, Texture, TextureDimension, TextureFormat,
        TextureView,
    },
    renderer::RenderDevice,
    texture::GpuImage,
};
use bevy_cef_core::prelude::Browsers;

const SURFACE_WIDTH: u32 = 800;
const SURFACE_HEIGHT: u32 = 800;

/// [macOS GPU OSR] plugin: inject the owned webview GPU texture into the render
/// world so the webview mesh renders the real page.
pub struct WebviewGpuInjectPlugin;

impl Plugin for WebviewGpuInjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingWebviewGpuTextures>().add_systems(
            Update,
            (allocate_webview_surfaces, collect_webview_gpu_textures).chain(),
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("[macos-gpu-osr] RenderApp sub-app missing; GPU texture injection disabled");
            return;
        };

        render_app
            .init_resource::<ExtractedWebviewGpuTextures>()
            .add_systems(ExtractSchedule, extract_webview_gpu_textures)
            // Run after the built-in GpuImage prepare so our insert overwrites
            // (rather than gets overwritten by) the placeholder CPU image.
            .add_systems(
                Render,
                inject_webview_gpu_images.after(prepare_assets::<GpuImage>),
            );
    }
}

/// One webview's owned GPU surface, paired with the material's surface id.
#[derive(Clone)]
struct WebviewGpuTexture {
    id: AssetId<Image>,
    texture: Texture,
    view: TextureView,
    width: u32,
    height: u32,
}

/// Main-world store of the owned GPU textures collected this frame.
#[derive(Resource, Default)]
struct PendingWebviewGpuTextures(Vec<WebviewGpuTexture>);

/// Render-world copy of the owned GPU textures to inject this frame.
#[derive(Resource, Default)]
struct ExtractedWebviewGpuTextures(Vec<WebviewGpuTexture>);

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
            TextureDimension::D2,
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

/// Main-world system: pull each webview's owned GPU surface texture out of
/// `Browsers` and pair it with the material's surface `AssetId<Image>`.
fn collect_webview_gpu_textures(
    browsers: NonSend<Browsers>,
    surfaces: Query<(Entity, &WebviewSurface)>,
    mut pending: ResMut<PendingWebviewGpuTextures>,
) {
    pending.0.clear();
    for (entity, texture, view, width, height) in browsers.webview_gpu_textures() {
        let Some((_, surface)) = surfaces.iter().find(|(e, _)| *e == entity) else {
            continue;
        };
        pending.0.push(WebviewGpuTexture {
            id: surface.0.id(),
            texture,
            view,
            width,
            height,
        });
    }
}

/// Extract the pending owned GPU textures into the render world.
fn extract_webview_gpu_textures(
    mut extracted: ResMut<ExtractedWebviewGpuTextures>,
    pending: Extract<Res<PendingWebviewGpuTextures>>,
) {
    extracted.0.clear();
    extracted.0.extend(pending.0.iter().cloned());
}

/// Render-world system: wrap each owned webview GPU texture in a `GpuImage` and
/// overwrite the `RenderAssets<GpuImage>` entry for the webview's surface id.
fn inject_webview_gpu_images(
    extracted: Res<ExtractedWebviewGpuTextures>,
    render_device: Res<RenderDevice>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    mut sampler: Local<Option<Sampler>>,
    mut logged: Local<bool>,
) {
    if extracted.0.is_empty() {
        return;
    }

    if !*logged {
        info!(
            "[macos-gpu-osr] render-world injection running for {} surface(s)",
            extracted.0.len()
        );
        *logged = true;
    }

    let sampler = sampler
        .get_or_insert_with(|| render_device.create_sampler(&SamplerDescriptor::default()))
        .clone();

    for entry in &extracted.0 {
        let gpu_image = GpuImage {
            texture: entry.texture.clone(),
            texture_view: entry.view.clone(),
            texture_format: TextureFormat::Bgra8UnormSrgb,
            texture_view_format: None,
            sampler: sampler.clone(),
            size: Extent3d {
                width: entry.width,
                height: entry.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            had_data: true,
        };

        gpu_images.insert(entry.id, gpu_image);
    }
}

// Reference WebviewMaterial so its type path stays linked even if the field
// access above is refactored; keeps the module self-documenting.
#[allow(dead_code)]
fn _assert_material_type(_m: &WebviewMaterial) {}
