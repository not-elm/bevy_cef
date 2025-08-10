use async_channel::{Receiver, Sender};
use bevy::prelude::*;
use bevy_cef_core::prelude::*;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

pub struct JsEmitEventPlugin<E: Event + DeserializeOwned>(PhantomData<E>);

impl<E: Event + DeserializeOwned> Plugin for JsEmitEventPlugin<E> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, receive_events::<E>);
    }
}

impl<E: Event + DeserializeOwned> Default for JsEmitEventPlugin<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn receive_events<E: Event + DeserializeOwned>(
    mut commands: Commands,
    receiver: ResMut<IpcEventRawReceiver>,
) {
    while let Ok(event) = receiver.0.try_recv() {
        if let Ok(payload) = serde_json::from_str::<E>(&event.payload) {
            commands.entity(event.webview).trigger(payload);
        }
    }
}

pub(crate) struct IpcRawEventPlugin;

impl Plugin for IpcRawEventPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();
        app.insert_resource(IpcEventRawSender(tx))
            .insert_resource(IpcEventRawReceiver(rx));
    }
}

#[derive(Resource)]
pub(crate) struct IpcEventRawSender(pub Sender<IpcEventRaw>);

#[derive(Resource)]
pub(crate) struct IpcEventRawReceiver(pub Receiver<IpcEventRaw>);
