use crate::browser_process::client_handler::ProcessMessageHandler;
use crate::prelude::{IntoString, PROCESS_MESSAGE_JS_EMIT};
use async_channel::Sender;
use bevy::prelude::Entity;
use cef::{Browser, Frame, ImplListValue, ListValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IpcEventRaw {
    pub webview: Entity,
    pub payload: String,
}

pub struct JsEmitEventHandler {
    webview: Entity,
    sender: Sender<IpcEventRaw>,
}

impl JsEmitEventHandler {
    pub const fn new(webview: Entity, sender: Sender<IpcEventRaw>) -> Self {
        Self { sender, webview }
    }
}

impl ProcessMessageHandler for JsEmitEventHandler {
    fn process_name(&self) -> &'static str {
        PROCESS_MESSAGE_JS_EMIT
    }

    fn handle_message(&self, _browser: &mut Browser, _frame: &mut Frame, args: Option<ListValue>) {
        if let Some(args) = args {
            let event = IpcEventRaw {
                webview: self.webview, // Placeholder, should be set correctly
                payload: args.string(0).into_string(),
            };
            let _ = self.sender.send_blocking(event);
        }
    }
}
