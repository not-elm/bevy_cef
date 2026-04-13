use crate::common::WebviewSurface;
use crate::prelude::{WebviewSize, WebviewSource};
use crate::system_param::mesh_aabb::MeshAabb;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::fmt::Debug;

/// Convert a DIP (logical-pixel) coordinate to a physical pixel index
/// inside an image of size `img_size`, given a logical viewport of `dip_size`.
///
/// Clamps to `img_size - 1` on each axis so it is safe to use as an index
/// into the image's byte buffer. Returns `UVec2::ZERO` when `dip_size` has
/// a zero component (caller is expected to early-out on invalid inputs).
fn dip_to_pixel(pos: Vec2, img_size: UVec2, dip_size: Vec2) -> UVec2 {
    if dip_size.x <= 0.0 || dip_size.y <= 0.0 || img_size.x == 0 || img_size.y == 0 {
        return UVec2::ZERO;
    }
    let sx = img_size.x as f32 / dip_size.x;
    let sy = img_size.y as f32 / dip_size.y;
    let x = ((pos.x * sx).floor() as u32).min(img_size.x - 1);
    let y = ((pos.y * sy).floor() as u32).min(img_size.y - 1);
    UVec2::new(x, y)
}

#[derive(SystemParam)]
pub struct WebviewPointer<'w, 's, C: Component = Camera3d> {
    aabb: MeshAabb<'w, 's>,
    cameras: Query<'w, 's, (Entity, &'static Camera, &'static GlobalTransform), With<C>>,
    webviews: Query<
        'w,
        's,
        (&'static GlobalTransform, &'static WebviewSize),
        (With<WebviewSource>, Without<Camera>),
    >,
    parents: Query<'w, 's, (Option<&'static ChildOf>, Has<WebviewSource>)>,
    surfaces: Query<'w, 's, &'static WebviewSurface>,
    images: Res<'w, Assets<Image>>,
}

impl<C: Component> WebviewPointer<'_, '_, C> {
    pub fn pos_from_trigger<P>(&self, trigger: &On<Pointer<P>>) -> Option<(Entity, Vec2)>
    where
        P: Clone + Reflect + Debug,
    {
        let webview = find_webview_entity(trigger.entity, &self.parents)?;
        let pos = self.pointer_pos(webview, trigger.pointer_location.position)?;
        Some((webview, pos))
    }

    pub fn pointer_pos(&self, webview: Entity, viewport_pos: Vec2) -> Option<Vec2> {
        let (min, max) = self.aabb.calculate_local(webview);
        let aabb_size = Vec2::new(max.x - min.x, max.y - min.y);
        let (webview_gtf, webview_size) = self.webviews.get(webview).ok()?;
        let pos = self.cameras.iter().find_map(|(_, camera, camera_gtf)| {
            pointer_to_webview_uv(
                viewport_pos,
                camera,
                camera_gtf,
                webview_gtf,
                aabb_size,
                webview_size.0,
            )
        })?;
        if self.is_transparent_at(webview, pos) {
            return None;
        }
        Some(pos)
    }

    /// Like [`Self::pointer_pos`], but does NOT check pixel transparency.
    /// Returns the pixel position AND the camera entity that produced the hit.
    /// Used for drag region hit-testing.
    pub fn pointer_pos_raw(&self, webview: Entity, viewport_pos: Vec2) -> Option<(Vec2, Entity)> {
        let (min, max) = self.aabb.calculate_local(webview);
        let aabb_size = Vec2::new(max.x - min.x, max.y - min.y);
        let (webview_gtf, webview_size) = self.webviews.get(webview).ok()?;
        self.cameras
            .iter()
            .find_map(|(cam_entity, camera, camera_gtf)| {
                pointer_to_webview_uv(
                    viewport_pos,
                    camera,
                    camera_gtf,
                    webview_gtf,
                    aabb_size,
                    webview_size.0,
                )
                .map(|pos| (pos, cam_entity))
            })
    }

    /// Like [`Self::pos_from_trigger`], but skips transparency check.
    /// Returns (webview, pixel_pos, camera_entity).
    pub fn pos_from_trigger_raw<P>(
        &self,
        trigger: &On<Pointer<P>>,
    ) -> Option<(Entity, Vec2, Entity)>
    where
        P: Clone + Reflect + Debug,
    {
        let webview = find_webview_entity(trigger.entity, &self.parents)?;
        let (pos, cam_entity) = self.pointer_pos_raw(webview, trigger.pointer_location.position)?;
        Some((webview, pos, cam_entity))
    }

    fn is_transparent_at(&self, webview: Entity, pos: Vec2) -> bool {
        let Ok(surface) = self.surfaces.get(webview) else {
            return false;
        };
        let Some(image) = self.images.get(surface.0.id()) else {
            return false;
        };
        let Ok((_, webview_size)) = self.webviews.get(webview) else {
            return false;
        };
        let img_size = UVec2::new(image.width(), image.height());
        if img_size.x == 0 || img_size.y == 0 || webview_size.0.x <= 0.0 || webview_size.0.y <= 0.0
        {
            return false;
        }
        let px = dip_to_pixel(pos, img_size, webview_size.0);
        let offset = ((px.y * img_size.x + px.x) * 4 + 3) as usize;
        let Some(data) = image.data.as_ref() else {
            return false;
        };
        data.len() > offset && data[offset] == 0
    }
}

fn find_webview_entity(
    entity: Entity,
    parents: &Query<(Option<&ChildOf>, Has<WebviewSource>)>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dip_to_pixel_identity_at_dpr_1() {
        // 800 DIP window, 800 px image → 1.0x scaling
        let result = dip_to_pixel(
            Vec2::new(100.0, 200.0),
            UVec2::new(800, 800),
            Vec2::new(800.0, 800.0),
        );
        assert_eq!(result, UVec2::new(100, 200));
    }

    #[test]
    fn dip_to_pixel_scales_by_dpr_2() {
        // 800 DIP window, 1600 px image → 2.0x scaling
        let result = dip_to_pixel(
            Vec2::new(100.0, 200.0),
            UVec2::new(1600, 1600),
            Vec2::new(800.0, 800.0),
        );
        assert_eq!(result, UVec2::new(200, 400));
    }

    #[test]
    fn dip_to_pixel_scales_by_dpr_1_5() {
        // 800×600 DIP window, 1200×900 px image → 1.5x scaling
        let result = dip_to_pixel(
            Vec2::new(100.0, 100.0),
            UVec2::new(1200, 900),
            Vec2::new(800.0, 600.0),
        );
        assert_eq!(result, UVec2::new(150, 150));
    }

    #[test]
    fn dip_to_pixel_clamps_to_image_bounds() {
        // pos larger than dip size must clamp to img_size - 1
        let result = dip_to_pixel(
            Vec2::new(1000.0, 1000.0),
            UVec2::new(800, 800),
            Vec2::new(800.0, 800.0),
        );
        assert_eq!(result, UVec2::new(799, 799));
    }

    #[test]
    fn dip_to_pixel_zero_position_is_origin() {
        let result = dip_to_pixel(Vec2::ZERO, UVec2::new(1600, 1600), Vec2::new(800.0, 800.0));
        assert_eq!(result, UVec2::ZERO);
    }
}
