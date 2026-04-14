//! Cursor override for resize edge hover feedback.

use bevy::prelude::*;
use bevy::window::SystemCursorIcon;

use super::ResizeZone;

/// When set, overrides CEF's page cursor (e.g., during resize edge hover).
#[derive(Resource, Default, Debug)]
pub struct SystemCursorOverride(Option<SystemCursorIcon>);

impl SystemCursorOverride {
    pub fn set(&mut self, icon: SystemCursorIcon) {
        self.0 = Some(icon);
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn get(&self) -> Option<SystemCursorIcon> {
        self.0
    }
}

/// Map a resize zone to the appropriate directional cursor.
pub(crate) fn cursor_for_zone(zone: ResizeZone) -> SystemCursorIcon {
    match zone {
        ResizeZone::N | ResizeZone::S => SystemCursorIcon::NsResize,
        ResizeZone::E | ResizeZone::W => SystemCursorIcon::EwResize,
        ResizeZone::NE | ResizeZone::SW => SystemCursorIcon::NeswResize,
        ResizeZone::NW | ResizeZone::SE => SystemCursorIcon::NwseResize,
    }
}
