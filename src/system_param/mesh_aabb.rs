use bevy::camera::primitives::Aabb;
use bevy::ecs::system::SystemParam;
use bevy::math::{Affine3A, Vec3};
use bevy::prelude::*;

#[derive(SystemParam)]
pub struct MeshAabb<'w, 's> {
    meshes: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            Option<&'static Aabb>,
            Option<&'static Children>,
        ),
    >,
}

impl MeshAabb<'_, '_> {
    pub fn calculate_local(&self, mesh_root: Entity) -> (Vec3, Vec3) {
        let Ok((root_tf, _, _)) = self.meshes.get(mesh_root) else {
            return (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        };
        let root_inv = root_tf.affine().inverse();
        calculate_aabb_with_root_inv(&[mesh_root], true, &self.meshes, &root_inv)
    }
}

fn calculate_aabb_with_root_inv(
    entities: &[Entity],
    include_children: bool,
    entities_query: &Query<(&GlobalTransform, Option<&Aabb>, Option<&Children>)>,
    root_inv: &Affine3A,
) -> (Vec3, Vec3) {
    let combine_bounds = |(a_min, a_max): (Vec3, Vec3), (b_min, b_max): (Vec3, Vec3)| {
        (a_min.min(b_min), a_max.max(b_max))
    };
    let default_bounds = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
    entities
        .iter()
        .filter_map(|&entity| {
            entities_query
                .get(entity)
                .map(|(&tf, bounds, children)| {
                    let local_tf = *root_inv * tf.affine();
                    let mut entity_bounds = bounds.map_or(default_bounds, |bounds| {
                        (
                            local_tf.transform_point3(Vec3::from(bounds.min())),
                            local_tf.transform_point3(Vec3::from(bounds.max())),
                        )
                    });
                    if include_children && let Some(children) = children {
                        let children_bounds = calculate_aabb_with_root_inv(
                            children,
                            include_children,
                            entities_query,
                            root_inv,
                        );
                        entity_bounds = combine_bounds(entity_bounds, children_bounds);
                    }
                    entity_bounds
                })
                .ok()
        })
        .fold(default_bounds, combine_bounds)
}
