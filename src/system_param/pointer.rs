use crate::prelude::{CefWebviewUri, WebviewSize};
use crate::system_param::mesh_aabb::MeshAabb;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::fmt::Debug;

#[derive(SystemParam)]
pub struct WebviewPointer<'w, 's, C: Component = Camera3d> {
    aabb: MeshAabb<'w, 's>,
    cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<C>>,
    webviews: Query<
        'w,
        's,
        (&'static GlobalTransform, &'static WebviewSize),
        (With<CefWebviewUri>, Without<Camera>),
    >,
    parents: Query<'w, 's, (Option<&'static ChildOf>, Has<CefWebviewUri>)>,
}

impl<C: Component> WebviewPointer<'_, '_, C> {
    pub fn pos_from_trigger<P>(&self, trigger: &Trigger<Pointer<P>>) -> Option<(Entity, Vec2)>
    where
        P: Clone + Reflect + Debug,
    {
        let webview = find_webview_entity(trigger.target, &self.parents)?;
        let pos = self.pointer_pos(webview, trigger.pointer_location.position)?;
        Some((webview, pos))
    }

    pub fn pointer_pos(&self, webview: Entity, viewport_pos: Vec2) -> Option<Vec2> {
        let (min, max) = self.aabb.calculate(webview);
        let aabb_size = Vec2::new(max.x - min.x, max.y - min.y);
        let (webview_gtf, webview_size) = self.webviews.get(webview).ok()?;
        self.cameras.iter().find_map(|(camera, camera_gtf)| {
            pointer_to_webview_uv(
                viewport_pos,
                camera,
                camera_gtf,
                webview_gtf,
                aabb_size,
                webview_size.0,
            )
        })
    }
}

fn find_webview_entity(
    entity: Entity,
    parents: &Query<(Option<&ChildOf>, Has<CefWebviewUri>)>,
) -> Option<Entity> {
    let (child_of, has_webview) = parents.get(entity).ok()?;
    if has_webview {
        return Some(entity);
    }
    if let Some(parent) = child_of {
        return find_webview_entity(parent.0, parents);
    }
    None
}

fn pointer_to_webview_uv(
    cursor_pos: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
    plane_tf: &GlobalTransform,
    plane_size: Vec2,
    tex_size: Vec2,
) -> Option<Vec2> {
    let ray = camera.viewport_to_world(cam_tf, cursor_pos).ok()?;
    let n = plane_tf.forward().as_vec3();
    let t = ray.intersect_plane(
        plane_tf.translation(),
        InfinitePlane3d::new(plane_tf.forward()),
    )?;
    let hit_world = ray.origin + ray.direction * t;
    let local_hit = plane_tf.affine().inverse().transform_point(hit_world);
    let local_normal = plane_tf.affine().inverse().transform_vector3(n).normalize();
    let abs_normal = local_normal.abs();
    let (u_coord, v_coord) = if abs_normal.z > abs_normal.x && abs_normal.z > abs_normal.y {
        (local_hit.x, local_hit.y)
    } else if abs_normal.y > abs_normal.x {
        (local_hit.x, local_hit.z)
    } else {
        (local_hit.y, local_hit.z)
    };

    let w = plane_size.x;
    let h = plane_size.y;
    let u = (u_coord + w * 0.5) / w;
    let v = (v_coord + h * 0.5) / h;
    if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
        // outside plane bounds
        return None;
    }
    let px = u * tex_size.x;
    let py = (1.0 - v) * tex_size.y;
    Some(Vec2::new(px, py))
}
