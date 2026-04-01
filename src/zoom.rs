use crate::common::ZoomLevel;
use bevy::prelude::*;
#[cfg(not(target_os = "windows"))]
use bevy_cef_core::prelude::Browsers;
#[cfg(target_os = "windows")]
use bevy_cef_core::prelude::BrowsersProxy;

pub(crate) struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_os = "windows"))]
        app.add_systems(Update, sync_zoom.run_if(any_changed_zoom));

        #[cfg(target_os = "windows")]
        app.add_systems(Update, sync_zoom_win.run_if(any_changed_zoom));
    }
}

fn any_changed_zoom(zoom: Query<&ZoomLevel, Changed<ZoomLevel>>) -> bool {
    !zoom.is_empty()
}

#[cfg(not(target_os = "windows"))]
fn sync_zoom(browsers: NonSend<Browsers>, zoom: Query<(Entity, &ZoomLevel), Changed<ZoomLevel>>) {
    for (entity, zoom_level) in zoom.iter() {
        browsers.set_zoom_level(&entity, zoom_level.0);
    }
}

#[cfg(target_os = "windows")]
fn sync_zoom_win(proxy: Res<BrowsersProxy>, zoom: Query<(Entity, &ZoomLevel), Changed<ZoomLevel>>) {
    for (entity, zoom_level) in zoom.iter() {
        proxy.set_zoom_level(&entity, zoom_level.0);
    }
}
