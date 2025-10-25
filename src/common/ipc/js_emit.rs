use async_channel::{Receiver, Sender};
use bevy::prelude::*;
use bevy_cef_core::prelude::*;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

#[derive(Debug, EntityEvent)]
pub struct Receive<M: Sync + Send + 'static>{
    #[event_target]
    pub webview: Entity,
    pub payload: M,
}

pub struct JsEmitEventPlugin<E: DeserializeOwned>(PhantomData<E>);

impl<E: DeserializeOwned + Send + Sync + 'static> Plugin for JsEmitEventPlugin<E> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, receive_events::<E>);
    }
}

impl<E: DeserializeOwned> Default for JsEmitEventPlugin<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn receive_events<E: DeserializeOwned + Send + Sync + 'static>(
    mut commands: Commands,
    receiver: ResMut<IpcEventRawReceiver>,
) {
    while let Ok(event) = receiver.0.try_recv() {
        if let Ok(payload) = serde_json::from_str::<E>(&event.payload) {
            commands.trigger(Receive {
                webview: event.webview,
                payload,
            });
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
