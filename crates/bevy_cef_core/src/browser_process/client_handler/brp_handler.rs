use crate::browser_process::client_handler::ProcessMessageHandler;
use crate::prelude::PROCESS_MESSAGE_BRP;
use crate::util::IntoString;
use async_channel::Sender;
use bevy::tasks::IoTaskPool;
use bevy_remote::{BrpMessage, BrpRequest};
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
                        && let Some(mut message) =
                            process_message_create(Some(&PROCESS_MESSAGE_BRP.into()))
                        && let Some(argument_list) = message.argument_list()
                    {
                        argument_list.set_string(0, Some(&id.as_str().into()));
                        argument_list.set_string(
                            1,
                            Some(&serde_json::to_string(&result).unwrap().as_str().into()),
                        );
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
