//! Resize plugin: wires derive pipeline, resize state machine, and unified observer.

use bevy::prelude::*;
use bevy::camera::primitives::MeshAabb as _;

use super::components::*;
use super::cursor::*;
use super::pipeline::*;
use super::*;
use crate::common::{WebviewSize, WebviewSource};
use crate::drag::{DragState, DraggableRegions, DraggingState, InteractionEndPending};
use crate::system_param::pointer::WebviewPointer;
use crate::webview::WebviewSet;

pub struct ResizePlugin;

impl Plugin for ResizePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResizeState>()
            .init_resource::<SystemCursorOverride>()
            .add_systems(
                Update,
                (
                    cursor_hover_system,
                    resize_tracking_system,
                )
                    .in_set(WebviewSet::ResizeInteraction),
            )
            .add_systems(
                Update,
                (
                    (init_resizable_system, pending_basis_init_system),
                    derive_pipeline_system,
                    (apply_display_to_mesh_system, apply_display_to_sprite_system),
                )
                    .chain()
                    .in_set(WebviewSet::DerivePipeline),
            )
            .add_systems(Update, attach_resize_observers);
    }
}

/// Attach a unified press observer to newly-created resizable webviews.
/// These entities are excluded from `attach_drag_observers` (which uses `Without<WebviewResizable>`).
fn attach_resize_observers(
    mut commands: Commands,
    webviews: Query<
        Entity,
        (
            Added<WebviewSource>,
            With<Transform>,
            With<WebviewResizable>,
            Or<(With<Mesh3d>, With<Mesh2d>)>,
        ),
    >,
) {
    for entity in webviews.iter() {
        commands.entity(entity).observe(on_resizable_press);
    }
}

/// Unified press observer for resizable webviews.
/// Classifies the hit as resize, drag, or page input and routes accordingly.
#[allow(clippy::too_many_arguments)]
fn on_resizable_press(
    trigger: On<Pointer<Press>>,
    mut resize_state: ResMut<ResizeState>,
    mut drag_state: ResMut<DragState>,
    mut commands: Commands,
    pointer: WebviewPointer,
    regions_q: Query<(&DraggableRegions, &WebviewResizable, &WebviewSize)>,
    transforms_q: Query<(&GlobalTransform, &Transform, &DisplaySize), With<WebviewSource>>,
    cameras_q: Query<(&Camera, &GlobalTransform)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    #[cfg(not(target_os = "windows"))] browsers: NonSend<bevy_cef_core::prelude::Browsers>,
    #[cfg(target_os = "windows")] browsers: Res<bevy_cef_core::prelude::BrowsersProxy>,
) {
    // Ignore if already interacting.
    if resize_state.is_resizing() || drag_state.is_dragging() {
        return;
    }

    let Some((webview, pixel_pos, camera_entity)) = pointer.pos_from_trigger_raw(&trigger) else {
        return;
    };
    let Ok((regions, resizable, webview_size)) = regions_q.get(webview) else {
        return;
    };

    let frame = ResizeFrame {
        width: webview_size.0.x as u32,
        height: webview_size.0.y as u32,
        edge_thickness: resizable.edge_thickness,
    };
    let hit = classify_hit(regions, Some(&frame), pixel_pos);

    match hit {
        HitResult::Resize(zone) => {
            // Start resize.
            let Ok((webview_gtf, webview_tf, display_size)) = transforms_q.get(webview) else {
                return;
            };
            let Ok((cam, cam_gtf)) = cameras_q.get(camera_entity) else {
                return;
            };
            let viewport_pos = trigger.pointer_location.position;
            let Ok(ray) = cam.viewport_to_world(cam_gtf, viewport_pos) else {
                return;
            };
            let plane_origin = webview_gtf.translation();
            let plane_normal = webview_gtf.forward();
            let Some(t) =
                ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))
            else {
                return;
            };
            let start_hit_world = ray.origin + ray.direction * t;

            // Compute u/v axes from the global transform.
            let u_axis = webview_gtf.right().as_vec3();
            let v_axis = webview_gtf.up().as_vec3();

            let aspect_lock_mode = resizable.aspect_lock;

            *resize_state = ResizeState::Resizing {
                webview,
                zone,
                start_display_size: display_size.0,
                start_translation: webview_tf.translation,
                start_hit_world,
                plane_origin,
                plane_normal,
                camera: camera_entity,
                u_axis,
                v_axis,
                aspect_lock_mode,
            };

            // Clear CEF hover state.
            #[cfg(not(target_os = "windows"))]
            browsers.send_mouse_move(
                &webview,
                std::iter::empty::<&MouseButton>(),
                pixel_pos,
                true,
            );
            #[cfg(target_os = "windows")]
            browsers.send_mouse_move(&webview, &[], pixel_pos, true);
        }
        HitResult::Drag => {
            // Start drag (duplicated from on_drag_press for resizable webviews).
            let Ok((webview_gtf, webview_tf, _display_size)) = transforms_q.get(webview) else {
                return;
            };
            let Ok((cam, cam_gtf)) = cameras_q.get(camera_entity) else {
                return;
            };
            let viewport_pos = trigger.pointer_location.position;
            let Ok(ray) = cam.viewport_to_world(cam_gtf, viewport_pos) else {
                return;
            };
            let plane_origin = webview_gtf.translation();
            let plane_normal = webview_gtf.forward();
            let Some(t) =
                ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal))
            else {
                return;
            };
            let start_hit = ray.origin + ray.direction * t;

            *drag_state = DragState::Dragging { webview };
            commands.entity(webview).insert(DraggingState {
                camera: camera_entity,
                start_hit,
                start_translation: webview_tf.translation,
                plane_normal,
                plane_origin,
            });

            // Clear CEF hover state.
            #[cfg(not(target_os = "windows"))]
            browsers.send_mouse_move(
                &webview,
                std::iter::empty::<&MouseButton>(),
                pixel_pos,
                true,
            );
            #[cfg(target_os = "windows")]
            browsers.send_mouse_move(&webview, &[], pixel_pos, true);
        }
        HitResult::None => {
            // Normal page input — do nothing, let CEF handle it.
        }
    }

    // Suppress the unused variable warning for keyboard — it is used for aspect lock detection.
    let _ = &keyboard;
}

/// Tracks an active resize drag each frame. Mirrors `drag_tracking_system`.
fn resize_tracking_system(
    mut resize_state: ResMut<ResizeState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    windows: Query<&Window>,
    mut webviews: Query<(
        &mut Transform,
        &mut DisplaySize,
        &WebviewResizable,
        &BaseRenderScale,
        &QualityMultiplier,
        &WebviewDpr,
    )>,
    cameras_q: Query<(&Camera, &GlobalTransform)>,
) {
    let ResizeState::Resizing {
        webview,
        zone,
        start_display_size,
        start_translation,
        start_hit_world,
        plane_origin,
        plane_normal,
        camera,
        u_axis,
        v_axis,
        aspect_lock_mode,
    } = &*resize_state
    else {
        return;
    };

    // Copy values we need before potential mutation.
    let webview = *webview;
    let zone = *zone;
    let start_display_size = *start_display_size;
    let start_translation = *start_translation;
    let start_hit_world = *start_hit_world;
    let plane_origin = *plane_origin;
    let plane_normal = *plane_normal;
    let camera = *camera;
    let u_axis = *u_axis;
    let v_axis = *v_axis;
    let aspect_lock_mode = *aspect_lock_mode;

    // Release detection.
    if !mouse_button.pressed(MouseButton::Left) {
        *resize_state = ResizeState::Idle;
        commands.insert_resource(InteractionEndPending {
            webview: Some(webview),
        });
        return;
    }

    // Raycast to snapshotted plane.
    let Some(cursor) = windows.iter().find_map(|w| w.cursor_position()) else {
        return;
    };
    let Ok((cam, cam_gtf)) = cameras_q.get(camera) else {
        return;
    };
    let Ok(ray) = cam.viewport_to_world(cam_gtf, cursor) else {
        return;
    };
    let Some(t) = ray.intersect_plane(plane_origin, InfinitePlane3d::new(plane_normal)) else {
        return;
    };
    let current_hit = ray.origin + ray.direction * t;
    let delta = current_hit - start_hit_world;
    let du = delta.dot(u_axis);
    let dv = delta.dot(v_axis);

    let Ok((mut tf, mut display_size, resizable, base, quality, dpr)) =
        webviews.get_mut(webview)
    else {
        return;
    };

    // Convert min/max from texture pixels to display-size units.
    let scale_factor = Vec2::new(
        base.0.x * quality.0 * dpr.0,
        base.0.y * quality.0 * dpr.0,
    );
    let min_display = Vec2::new(
        resizable.min_size.x as f32 / scale_factor.x,
        resizable.min_size.y as f32 / scale_factor.y,
    );
    let max_display = resizable.max_size.map(|max| {
        Vec2::new(
            max.x as f32 / scale_factor.x,
            max.y as f32 / scale_factor.y,
        )
    });

    // Determine if aspect lock is active.
    let lock_aspect = match aspect_lock_mode {
        AspectLockMode::Always => true,
        AspectLockMode::Never => false,
        AspectLockMode::LockOnShift => {
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
        }
    };

    let (new_size, new_translation) = apply_resize(
        zone,
        start_display_size,
        start_translation,
        du,
        dv,
        u_axis,
        v_axis,
        lock_aspect,
        min_display,
        max_display,
    );

    display_size.0 = new_size;
    tf.translation = new_translation;
}

/// Detects cursor hovering over a resize edge and sets the appropriate cursor icon.
#[allow(clippy::too_many_arguments)]
fn cursor_hover_system(
    resize_state: Res<ResizeState>,
    windows: Query<&Window>,
    mut cursor_override: ResMut<SystemCursorOverride>,
    // Mesh path
    pointer: WebviewPointer,
    mesh_resizables: Query<
        (Entity, &WebviewResizable, &WebviewSize),
        (With<WebviewResizable>, With<Mesh3d>),
    >,
    // Sprite path
    sprite_resizables: Query<
        (Entity, &WebviewResizable, &WebviewSize, &Sprite, &GlobalTransform),
        (With<WebviewResizable>, With<Sprite>),
    >,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if resize_state.is_resizing() {
        return;
    }

    let Some(cursor_pos) = windows.iter().find_map(|w| w.cursor_position()) else {
        cursor_override.clear();
        return;
    };

    // Try mesh path first
    for (entity, resizable, size) in mesh_resizables.iter() {
        if let Some((pixel_pos, _cam)) = pointer.pointer_pos_raw(entity, cursor_pos) {
            let frame = ResizeFrame {
                width: size.0.x as u32,
                height: size.0.y as u32,
                edge_thickness: resizable.edge_thickness,
            };
            if let Some(zone) = frame.classify(pixel_pos) {
                cursor_override.set(cursor_for_zone(zone));
                return;
            }
        }
    }

    // Try sprite path
    for (_entity, resizable, size, sprite, gtf) in sprite_resizables.iter() {
        if let Some(pixel_pos) =
            crate::webview::webview_sprite::obtain_relative_pos(sprite, size, gtf, &cameras, cursor_pos)
        {
            let frame = ResizeFrame {
                width: size.0.x as u32,
                height: size.0.y as u32,
                edge_thickness: resizable.edge_thickness,
            };
            if let Some(zone) = frame.classify(pixel_pos) {
                cursor_override.set(cursor_for_zone(zone));
                return;
            }
        }
    }

    // No resize edge hit -- clear override
    cursor_override.clear();
}

/// Initializes pipeline components when `WebviewResizable` is first added to an entity.
#[allow(clippy::too_many_arguments)]
fn init_resizable_system(
    mut commands: Commands,
    new_resizables: Query<
        (
            Entity,
            &WebviewSize,
            &Transform,
            &GlobalTransform,
            Option<&Mesh3d>,
            Option<&Sprite>,
        ),
        Added<WebviewResizable>,
    >,
    mesh_assets: Res<Assets<Mesh>>,
    windows: Query<&Window>,
) {
    let dpr = windows
        .iter()
        .next()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0);

    for (entity, webview_size, _tf, gtf, mesh3d, sprite) in new_resizables.iter() {
        if let Some(mesh3d) = mesh3d {
            // 3D mesh path
            if let Some(mesh) = mesh_assets.get(&mesh3d.0) {
                if let Some(aabb) = mesh.compute_aabb() {
                    let local_size =
                        Vec2::new(aabb.half_extents.x * 2.0, aabb.half_extents.y * 2.0);
                    let scale = gtf.compute_transform().scale;
                    let world_size =
                        Vec2::new(local_size.x * scale.x, local_size.y * scale.y);
                    let base = Vec2::new(
                        webview_size.0.x / (world_size.x * dpr),
                        webview_size.0.y / (world_size.y * dpr),
                    );
                    commands.entity(entity).insert((
                        DisplaySize(world_size),
                        BaseRenderScale(base),
                        QualityMultiplier::default(),
                        WebviewDpr(dpr),
                        WebviewBasis2d {
                            u_axis: Vec3::X,
                            v_axis: Vec3::Y,
                            local_size,
                        },
                    ));
                } else {
                    // AABB not ready -- mark for deferred init
                    commands.entity(entity).insert((
                        PendingBasisInit,
                        QualityMultiplier::default(),
                        WebviewDpr(dpr),
                    ));
                }
            } else {
                commands.entity(entity).insert((
                    PendingBasisInit,
                    QualityMultiplier::default(),
                    WebviewDpr(dpr),
                ));
            }
        } else if let Some(sprite) = sprite {
            // 2D sprite path
            let display_size = sprite
                .custom_size
                .unwrap_or(Vec2::new(webview_size.0.x, webview_size.0.y));
            let base = Vec2::new(
                webview_size.0.x / (display_size.x * dpr),
                webview_size.0.y / (display_size.y * dpr),
            );
            commands.entity(entity).insert((
                DisplaySize(display_size),
                BaseRenderScale(base),
                QualityMultiplier::default(),
                WebviewDpr(dpr),
            ));
        }
    }
}

/// Retries initialization for entities with `PendingBasisInit` once their mesh AABB becomes available.
fn pending_basis_init_system(
    mut commands: Commands,
    pending: Query<
        (Entity, &WebviewSize, &GlobalTransform, &Mesh3d, &WebviewDpr),
        With<PendingBasisInit>,
    >,
    mesh_assets: Res<Assets<Mesh>>,
) {
    for (entity, webview_size, gtf, mesh3d, dpr) in pending.iter() {
        let Some(mesh) = mesh_assets.get(&mesh3d.0) else {
            continue;
        };
        let Some(aabb) = mesh.compute_aabb() else {
            continue;
        };
        let local_size = Vec2::new(aabb.half_extents.x * 2.0, aabb.half_extents.y * 2.0);
        let scale = gtf.compute_transform().scale;
        let world_size = Vec2::new(local_size.x * scale.x, local_size.y * scale.y);
        let base = Vec2::new(
            webview_size.0.x / (world_size.x * dpr.0),
            webview_size.0.y / (world_size.y * dpr.0),
        );
        commands.entity(entity).insert((
            DisplaySize(world_size),
            BaseRenderScale(base),
            WebviewBasis2d {
                u_axis: Vec3::X,
                v_axis: Vec3::Y,
                local_size,
            },
        ));
        commands.entity(entity).remove::<PendingBasisInit>();
    }
}
