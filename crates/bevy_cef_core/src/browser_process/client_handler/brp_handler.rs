use crate::browser_process::client_handler::ProcessMessageHandler;
use crate::prelude::PROCESS_MESSAGE_BRP;
use crate::render_process::render_process_handler::{BrpReply, BrpReplyError};
use crate::util::IntoString;
use async_channel::Sender;
use bevy::tasks::IoTaskPool;
use bevy_remote::{BrpError, BrpMessage, BrpRequest};
use cef::{
    Browser, Frame, ImplFrame, ImplListValue, ImplProcessMessage, ListValue, ProcessId,
    process_message_create,
};
use cef_dll_sys::cef_process_id_t;

pub struct BrpHandler {
    sender: Sender<BrpMessage>,
}

impl BrpHandler {
    pub const fn new(sender: Sender<BrpMessage>) -> Self {
        Self { sender }
    }
}

impl ProcessMessageHandler for BrpHandler {
    fn process_name(&self) -> &'static str {
        PROCESS_MESSAGE_BRP
    }

    fn handle_message(&self, _browser: &mut Browser, frame: &mut Frame, args: Option<ListValue>) {
        if let Some(args) = args
            && let Ok(request) = serde_json::from_str::<BrpRequest>(&args.string(1).into_string())
        {
            let id = args.string(0).into_string();
            let frame = frame.clone();
            let brp_sender = self.sender.clone();
            IoTaskPool::get()
                .spawn(async move {
                    let (tx, rx) = async_channel::unbounded();
                    if brp_sender
                        .send(BrpMessage {
                            method: request.method,
                            params: request.params,
                            sender: tx,
                        })
                        .await
                        .is_err()
                    {
                        return;
                    }
                    if let Ok(result) = rx.recv().await
                        && let Ok(payload) =
                            serde_json::to_string::<BrpReply>(&result.map_err(BrpReplyError::from))
                        && let Some(mut message) =
                            process_message_create(Some(&PROCESS_MESSAGE_BRP.into()))
                        && let Some(argument_list) = message.argument_list()
                    {
                        argument_list.set_string(0, Some(&id.as_str().into()));
                        argument_list.set_string(1, Some(&payload.as_str().into()));
                        frame.send_process_message(
                            ProcessId::from(cef_process_id_t::PID_RENDERER),
                            Some(&mut message),
                        );
                    }
                })
                .detach();
        }
    }
}

impl From<BrpError> for BrpReplyError {
    fn from(error: BrpError) -> Self {
        Self {
            code: error.code,
            message: error.message,
            data: error.data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_remote::BrpResult;
    use serde_json::json;

    /// Asserts that a host result mapped into a [`BrpReply`] serializes to exactly the JSON of
    /// the `bevy_remote::BrpResult` it came from.
    ///
    /// Case: a 0.14.1 host answers a render process installed from 0.14.0, which decodes the
    /// payload as `bevy_remote::BrpResult`.
    #[test]
    fn mapped_reply_serializes_like_bevy_remote() {
        let results: [BrpResult; 3] = [
            Ok(json!({ "greeting": "Hello" })),
            Err(BrpError {
                code: -32601,
                message: "Method `nope` not found".into(),
                data: None,
            }),
            Err(BrpError {
                code: -32602,
                message: "Invalid params".into(),
                data: Some(json!({ "field": "name" })),
            }),
        ];
        for result in results {
            let expected = serde_json::to_string(&result).expect("a serializable BrpResult");
            let reply: BrpReply = result.map_err(BrpReplyError::from);
            let actual = serde_json::to_string(&reply).expect("a serializable BrpReply");
            assert_eq!(actual, expected);
        }
    }
}
