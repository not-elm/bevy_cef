#[cfg(target_os = "macos")]
use crate::common::WebviewIoSurface;
use crate::common::{WebviewSize, WebviewSource};
#[cfg(not(target_os = "macos"))]
use crate::prelude::update_webview_image;
use crate::webview::scroll_delta;
use crate::webview::touch_asset;
use bevy::asset::AssetEventSystems;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_cef_core::prelude::Browsers;
#[cfg(not(target_os = "macos"))]
use bevy_cef_core::prelude::RenderTextureMessage;

pub(in crate::webview) struct WebviewSpritePlugin;

impl Plugin for WebviewSpritePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SpritePickingPlugin>() {
            app.add_plugins(SpritePickingPlugin);
        }

        app.add_plugins(SpriteWebviewMaterialRebindPlugin)
            .add_systems(
                Update,
                (
                    setup_observers,
                    on_mouse_wheel.run_if(on_message::<MouseWheel>),
                ),
            );

        #[cfg(not(target_os = "macos"))]
        app.add_plugins(SpriteWebviewPaintPlugin);
    }
}

/// Rebinds the sprite material of every sprite webview whenever Bevy assigns or reassigns it,
/// so the material's bind group captures the webview's current texture.
struct SpriteWebviewMaterialRebindPlugin;

impl Plugin for SpriteWebviewMaterialRebindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            touch_reassigned_sprite_materials
                .after(AssetEventSystems)
                .run_if(resource_exists::<Assets<SpriteMeshMaterial>>),
        );
    }
}

/// Copies each CPU `OnPaint` frame into its sprite webview's `Sprite.image`. When a frame
/// reallocates the image, the sprite material is touched in the same frame, so its
/// `AssetEvent::Modified` reaches the same extraction as the image's.
#[cfg(not(target_os = "macos"))]
struct SpriteWebviewPaintPlugin;

#[cfg(not(target_os = "macos"))]
impl Plugin for SpriteWebviewPaintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            render
                .run_if(on_message::<RenderTextureMessage>)
                .before(AssetEventSystems),
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn render(
    mut er: MessageReader<RenderTextureMessage>,
    mut images: ResMut<Assets<Image>>,
    mut materials: Option<ResMut<Assets<SpriteMeshMaterial>>>,
    webviews: Query<(&Sprite, Option<&MeshMaterial2d<SpriteMeshMaterial>>), With<WebviewSource>>,
) {
    for texture in er.read() {
        let Ok((sprite, material)) = webviews.get(texture.webview) else {
            continue;
        };
        let Some(image) = images.get_mut(sprite.image.id()) else {
            continue;
        };
        if update_webview_image(texture, image)
            && let (Some(materials), Some(material)) = (materials.as_deref_mut(), material)
        {
            touch_asset(materials, material.id());
        }
    }
}

/// Touches the sprite material of every sprite webview whose `MeshMaterial2d` changed this
/// frame, including its first assignment.
fn touch_reassigned_sprite_materials(
    mut materials: ResMut<Assets<SpriteMeshMaterial>>,
    webviews: Query<
        &MeshMaterial2d<SpriteMeshMaterial>,
        (
            With<WebviewSource>,
            With<Sprite>,
            Changed<MeshMaterial2d<SpriteMeshMaterial>>,
        ),
    >,
) {
    for material in webviews.iter() {
        touch_asset(&mut materials, material.id());
    }
}

fn setup_observers(
    mut commands: Commands,
    webviews: Query<Entity, (Added<WebviewSource>, With<Sprite>)>,
) {
    for entity in webviews.iter() {
        commands
            .entity(entity)
            .observe(apply_on_pointer_move)
            .observe(apply_on_pointer_pressed)
            .observe(apply_on_pointer_released);
    }
}

/// [macOS GPU OSR] Returns `true` when `pos` (webview DIP space) lands on a
/// fully transparent pixel of the webview's retained IOSurface, in which case
/// the event should pass through instead of reaching CEF.
///
/// On macOS `Sprite.image` is an opaque black placeholder (the real pixels only
/// exist on the GPU), so bevy's sprite picking backend cannot alpha-test — the
/// test happens here instead, mirroring the mesh/UI input paths. Limitation:
/// the picking backend still treats the whole sprite rect as a hit, so
/// transparent regions continue to block lower pickable entities.
#[cfg(target_os = "macos")]
fn sprite_pos_transparent(
    entity: Entity,
    pos: Vec2,
    webviews: &Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
    io_surfaces: &Query<&WebviewIoSurface>,
) -> bool {
    let Ok((_, webview_size, _)) = webviews.get(entity) else {
        return false;
    };
    io_surfaces.get(entity).is_ok_and(|io_surface| {
        crate::webview::alpha::is_pixel_transparent_surface(&io_surface.0, webview_size.0, pos)
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_on_pointer_move(
    trigger: On<PointerMove>,
    input: Res<ButtonInput<MouseButton>>,
    browsers: NonSend<Browsers>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
    #[cfg(target_os = "macos")] io_surfaces: Query<&WebviewIoSurface>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if sprite_pos_transparent(trigger.entity, pos, &webviews, &io_surfaces) {
        return;
    }
    browsers.send_mouse_move(&trigger.entity, input.get_pressed(), pos, false);
}

#[allow(clippy::too_many_arguments)]
fn apply_on_pointer_pressed(
    trigger: On<PointerPress>,
    browsers: NonSend<Browsers>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
    #[cfg(target_os = "macos")] io_surfaces: Query<&WebviewIoSurface>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if sprite_pos_transparent(trigger.entity, pos, &webviews, &io_surfaces) {
        return;
    }
    browsers.send_mouse_click(&trigger.entity, pos, trigger.button, false);
}

#[allow(clippy::too_many_arguments)]
fn apply_on_pointer_released(
    trigger: On<PointerRelease>,
    browsers: NonSend<Browsers>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    webviews: Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
    #[cfg(target_os = "macos")] io_surfaces: Query<&WebviewIoSurface>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some(pos) = obtain_relative_pos_from_trigger(&trigger, &webviews, &cameras) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if sprite_pos_transparent(trigger.entity, pos, &webviews, &io_surfaces) {
        return;
    }
    browsers.send_mouse_click(&trigger.entity, pos, trigger.button, true);
}

#[allow(clippy::too_many_arguments)]
fn on_mouse_wheel(
    mut er: MessageReader<MouseWheel>,
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &Sprite, &WebviewSize, &GlobalTransform)>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    #[cfg(target_os = "macos")] io_surfaces: Query<&WebviewIoSurface>,
    drag_state: Res<crate::drag::DragState>,
    resize_state: Res<crate::resize::ResizeState>,
) {
    if drag_state.is_dragging() {
        return;
    }
    if resize_state.is_resizing() {
        return;
    }
    let Some(cursor_pos) = windows.iter().find_map(|window| window.cursor_position()) else {
        return;
    };
    for event in er.read() {
        let delta = scroll_delta(event.unit, event.x, event.y);
        for (webview, sprite, webview_size, gtf) in webviews.iter() {
            let Some(pos) = obtain_relative_pos(sprite, webview_size, gtf, &cameras, cursor_pos)
            else {
                continue;
            };
            #[cfg(target_os = "macos")]
            if io_surfaces.get(webview).is_ok_and(|io_surface| {
                crate::webview::alpha::is_pixel_transparent_surface(
                    &io_surface.0,
                    webview_size.0,
                    pos,
                )
            }) {
                continue;
            }
            browsers.send_mouse_wheel(&webview, pos, delta);
        }
    }
}

fn obtain_relative_pos_from_trigger<E: PointerEvent>(
    trigger: &On<E>,
    webviews: &Query<(&Sprite, &WebviewSize, &GlobalTransform)>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (sprite, webview_size, gtf) = webviews.get(trigger.event_target()).ok()?;
    obtain_relative_pos(
        sprite,
        webview_size,
        gtf,
        cameras,
        trigger.pointer().position,
    )
}

pub(crate) fn obtain_relative_pos(
    sprite: &Sprite,
    webview_size: &WebviewSize,
    transform: &GlobalTransform,
    cameras: &Query<(&Camera, &GlobalTransform)>,
    cursor_pos: Vec2,
) -> Option<Vec2> {
    let size = sprite.custom_size?;
    let viewport_pos = cameras.iter().find_map(|(camera, camera_gtf)| {
        camera
            .world_to_viewport(camera_gtf, transform.translation())
            .ok()
    })?;
    let relative_pos = (cursor_pos - viewport_pos + size / 2.0) / size;
    Some(relative_pos * webview_size.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetPlugin;
    #[cfg(not(target_os = "macos"))]
    use bevy_cef_core::prelude::RenderPaintElementType;

    fn rebind_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            SpriteWebviewMaterialRebindPlugin,
        ))
        .init_asset::<SpriteMeshMaterial>();
        app
    }

    fn add_sprite_material(app: &mut App) -> Handle<SpriteMeshMaterial> {
        app.world_mut()
            .resource_mut::<Assets<SpriteMeshMaterial>>()
            .add(SpriteMeshMaterial::default())
    }

    fn modified_sprite_materials(app: &mut App) -> Vec<AssetId<SpriteMeshMaterial>> {
        app.world_mut()
            .resource_mut::<Messages<AssetEvent<SpriteMeshMaterial>>>()
            .drain()
            .filter_map(|event| match event {
                AssetEvent::Modified { id } => Some(id),
                _ => None,
            })
            .collect()
    }

    #[cfg(not(target_os = "macos"))]
    fn paint(webview: Entity, width: u32, height: u32) -> RenderTextureMessage {
        RenderTextureMessage {
            webview,
            ty: RenderPaintElementType::View,
            width,
            height,
            buffer: vec![0; (width * height * 4) as usize],
        }
    }

    /// Asserts that the sprite material assigned to a sprite webview is touched, queuing
    /// `AssetEvent::Modified` for it.
    ///
    /// Case: a resize changes the sprite's `custom_size`, and Bevy hands the webview a cached
    /// sprite material whose bind group still references the texture from before the resize.
    #[test]
    fn assigned_sprite_material_is_touched() {
        let mut app = rebind_app();
        let material = add_sprite_material(&mut app);
        let webview = app
            .world_mut()
            .spawn((
                WebviewSource::new("https://example.com"),
                Sprite::default(),
                MeshMaterial2d(material.clone()),
            ))
            .id();

        app.update();
        app.update();
        assert!(modified_sprite_materials(&mut app).contains(&material.id()));

        let reassigned = add_sprite_material(&mut app);
        app.world_mut()
            .entity_mut(webview)
            .insert(MeshMaterial2d(reassigned.clone()));

        app.update();
        app.update();
        assert!(modified_sprite_materials(&mut app).contains(&reassigned.id()));
    }

    /// Asserts that the sprite material of a sprite without `WebviewSource` is not touched.
    ///
    /// Case: an app draws an ordinary sprite next to its sprite webviews, and Bevy assigns that
    /// sprite a sprite material of its own.
    #[test]
    fn non_webview_sprite_material_is_not_touched() {
        let mut app = rebind_app();
        let material = add_sprite_material(&mut app);
        app.world_mut()
            .spawn((Sprite::default(), MeshMaterial2d(material.clone())));

        app.update();
        app.update();
        assert!(!modified_sprite_materials(&mut app).contains(&material.id()));
    }

    /// Asserts that a sprite webview's material is touched in the frame it is assigned but not
    /// again in later frames while it stays assigned.
    ///
    /// Case: a sprite webview keeps showing a static page at a fixed size after its first frame.
    #[test]
    fn unchanged_sprite_material_is_not_touched_again() {
        let mut app = rebind_app();
        let material = add_sprite_material(&mut app);
        app.world_mut().spawn((
            WebviewSource::new("https://example.com"),
            Sprite::default(),
            MeshMaterial2d(material.clone()),
        ));

        app.update();
        app.update();
        assert!(modified_sprite_materials(&mut app).contains(&material.id()));

        app.update();
        app.update();
        assert!(!modified_sprite_materials(&mut app).contains(&material.id()));
    }

    /// Asserts that a paint which reallocates a sprite webview's image touches its sprite
    /// material within the same frame, while a paint at the image's current size does not.
    ///
    /// Case: the user resizes a sprite webview showing a static page, so CEF sends one paint at
    /// the new size and then repaints at that size.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn resized_paint_touches_sprite_material() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            SpriteWebviewPaintPlugin,
        ))
        .init_asset::<Image>()
        .init_asset::<SpriteMeshMaterial>()
        .add_message::<RenderTextureMessage>();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::default());
        let material = add_sprite_material(&mut app);
        let webview = app
            .world_mut()
            .spawn((
                WebviewSource::new("https://example.com"),
                Sprite::from_image(image),
                MeshMaterial2d(material.clone()),
            ))
            .id();
        app.update();

        app.world_mut().write_message(paint(webview, 4, 2));
        app.update();
        assert!(modified_sprite_materials(&mut app).contains(&material.id()));

        app.world_mut().write_message(paint(webview, 4, 2));
        app.update();
        app.update();
        assert!(!modified_sprite_materials(&mut app).contains(&material.id()));
    }

    /// Asserts that the sprite material rebind is skipped, rather than panicking, when
    /// `Assets<SpriteMeshMaterial>` does not exist.
    ///
    /// Case: an app built without Bevy's sprite renderer still spawns a `Sprite` webview.
    #[test]
    fn missing_sprite_material_assets_do_not_panic() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SpriteWebviewMaterialRebindPlugin));
        app.world_mut().spawn((
            WebviewSource::new("https://example.com"),
            Sprite::default(),
            MeshMaterial2d::<SpriteMeshMaterial>(Handle::default()),
        ));

        app.update();
    }
}
