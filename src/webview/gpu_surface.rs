//! [poc-osr] SPIKE: prove that an externally-created wgpu texture can be
//! injected into Bevy's `RenderAssets<GpuImage>` and shown on a webview mesh
//! material.
//!
//! This is isolated from the real IOSurface/FFI work: instead of a real CEF
//! shared texture, it fills the injected texture with solid MAGENTA. If the
//! webview mesh renders magenta, the injection wiring is proven and later
//! tasks can swap the magenta fill for the real IOSurface-derived texture.
//!
//! Pipeline:
//! 1. Main world (`Update`): allocate a `Handle<Image>` surface for each
//!    webview mesh material that does not yet have one, point the material's
//!    `extension.surface` at it, and tag the entity with `WebviewSurface`.
//! 2. `ExtractSchedule`: copy the surface `AssetId<Image>`s into the render
//!    world (`ExtractedWebviewSurfaces`).
//! 3. `Render` (after `prepare_assets::<GpuImage>`): create + fill a magenta
//!    wgpu texture and `insert` it into `RenderAssets<GpuImage>` for that id,
//!    overwriting whatever `prepare_assets` produced from the (black) CPU
//!    image.

use crate::prelude::{WebviewExtendStandardMaterial, WebviewMaterial, WebviewSurface};
use bevy::asset::{AssetId, RenderAssetUsages};
use bevy::prelude::*;
use bevy::render::{
    Extract, Render, RenderApp,
    render_asset::{RenderAssets, prepare_assets},
    render_resource::{
        Extent3d, Origin3d, SamplerDescriptor, TexelCopyBufferLayout, TexelCopyTextureInfo,
        TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        TextureViewDescriptor,
    },
    renderer::{RenderDevice, RenderQueue},
    texture::GpuImage,
};

const SURFACE_WIDTH: u32 = 800;
const SURFACE_HEIGHT: u32 = 800;
/// BGRA8 magenta: B=255, G=0, R=255, A=255.
const MAGENTA_BGRA: [u8; 4] = [255, 0, 255, 255];

/// [poc-osr] SPIKE plugin: inject a dummy magenta `GpuImage` into the render
/// world for each webview mesh, validating the OSR material-binding path.
pub struct WebviewGpuInjectSpikePlugin;

impl Plugin for WebviewGpuInjectSpikePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, allocate_webview_surfaces);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("[poc-osr] RenderApp sub-app missing; magenta injection disabled");
            return;
        };

        render_app
            .init_resource::<ExtractedWebviewSurfaces>()
            .add_systems(ExtractSchedule, extract_webview_surfaces)
            // Run after the built-in GpuImage prepare so our insert overwrites
            // (rather than gets overwritten by) the CPU-image-derived GpuImage.
            .add_systems(
                Render,
                inject_magenta_gpu_images.after(prepare_assets::<GpuImage>),
            );
    }
}

/// Render-world store of the webview surface ids that should be overwritten
/// with the injected (magenta) texture this frame.
#[derive(Resource, Default)]
struct ExtractedWebviewSurfaces {
    ids: Vec<AssetId<Image>>,
}

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
            "[poc-osr] allocated magenta surface {:?} for webview {:?}",
            handle.id(),
            entity
        );
    }
}

/// Extract the surface `AssetId`s into the render world.
fn extract_webview_surfaces(
    mut extracted: ResMut<ExtractedWebviewSurfaces>,
    surfaces: Extract<Query<&WebviewSurface>>,
) {
    extracted.ids.clear();
    extracted
        .ids
        .extend(surfaces.iter().map(|surface| surface.0.id()));
}

/// Render-world system: build a magenta texture and overwrite the
/// `RenderAssets<GpuImage>` entry for each extracted webview surface.
fn inject_magenta_gpu_images(
    extracted: Res<ExtractedWebviewSurfaces>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    mut logged: Local<bool>,
) {
    if extracted.ids.is_empty() {
        return;
    }

    if !*logged {
        info!(
            "[poc-osr] render-world injection running for {} surface(s): {:?}",
            extracted.ids.len(),
            extracted.ids
        );
        *logged = true;
    }

    let size = Extent3d {
        width: SURFACE_WIDTH,
        height: SURFACE_HEIGHT,
        depth_or_array_layers: 1,
    };

    // Solid-magenta BGRA buffer (re-created each frame; fine for a spike).
    let mut pixels = Vec::with_capacity((SURFACE_WIDTH * SURFACE_HEIGHT * 4) as usize);
    for _ in 0..(SURFACE_WIDTH * SURFACE_HEIGHT) {
        pixels.extend_from_slice(&MAGENTA_BGRA);
    }
    let bytes_per_row = SURFACE_WIDTH * 4;

    for &id in &extracted.ids {
        let texture = render_device.create_texture(&TextureDescriptor {
            label: Some("poc_osr_magenta_surface"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        render_queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(SURFACE_HEIGHT),
            },
            size,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let gpu_image = GpuImage {
            texture,
            texture_view,
            texture_format: TextureFormat::Bgra8UnormSrgb,
            texture_view_format: None,
            sampler,
            size,
            mip_level_count: 1,
            had_data: true,
        };

        gpu_images.insert(id, gpu_image);
    }
}

// Reference WebviewMaterial so its type path stays linked even if the field
// access above is refactored; keeps the spike self-documenting.
#[allow(dead_code)]
fn _assert_material_type(_m: &WebviewMaterial) {}
