//! Drag plugin: provides the async_channel plumbing that delivers
//! CEF `OnDraggableRegionsChanged` callbacks to Bevy ECS.

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_cef_core::prelude::{DraggableRegion, DraggableRegionSenderInner};

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();
        app.insert_resource(DraggableRegionSender(tx))
            .insert_resource(DraggableRegionReceiver(rx));
    }
}

#[derive(Resource, Debug, Deref)]
pub(crate) struct DraggableRegionSender(pub(crate) DraggableRegionSenderInner);

#[derive(Resource, Debug)]
pub(crate) struct DraggableRegionReceiver(pub(crate) Receiver<(Entity, Vec<DraggableRegion>)>);
