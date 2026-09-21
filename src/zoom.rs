use crate::common::ZoomLevel;
use bevy::prelude::*;
use bevy_cef_core::prelude::Browsers;

pub(crate) struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_zoom.run_if(any_changed_zoom));
    }
}

fn any_changed_zoom(zoom: Query<&ZoomLevel, Changed<ZoomLevel>>) -> bool {
    !zoom.is_empty()
}

fn sync_zoom(browsers: NonSend<Browsers>, zoom: Query<(Entity, &ZoomLevel), Changed<ZoomLevel>>) {
    for (entity, zoom_level) in zoom.iter() {
        browsers.set_zoom_level(&entity, zoom_level.0);
    }
}
