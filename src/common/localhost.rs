use bevy::prelude::*;

mod asset_loader;
pub(crate) mod responser;

use crate::common::localhost::asset_loader::LocalSchemeAssetLoaderPlugin;

/// A plugin that adds support for handling local scheme requests in Bevy applications.
pub(crate) struct LocalHostPlugin;

impl Plugin for LocalHostPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((responser::ResponserPlugin, LocalSchemeAssetLoaderPlugin));
    }
}
