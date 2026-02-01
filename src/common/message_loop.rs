use crate::RunOnMainThread;
use bevy::prelude::*;
use bevy_cef_core::prelude::*;
use cef::args::Args;
use cef::{Settings, api_hash, execute_process, initialize, shutdown, sys};

/// Controls the CEF message loop.
///
/// - Windows and Linux: Support [`multi_threaded_message_loop`](https://cef-builds.spotifycdn.com/docs/106.1/structcef__settings__t.html#a518ac90db93ca5133a888faa876c08e0), so it is used.
/// - macOS: Calls [`CefDoMessageLoopWork`](https://cef-builds.spotifycdn.com/docs/106.1/cef__app_8h.html#a830ae43dcdffcf4e719540204cefdb61) every frame.
pub struct MessageLoopPlugin;

impl Plugin for MessageLoopPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "macos")]
        load_cef_library(app);

        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
        let args = Args::new();
        let (tx, rx) = std::sync::mpsc::channel();

        let mut cef_app = BrowserProcessAppBuilder::build(tx);
        let ret = execute_process(
            Some(args.as_main_args()),
            Some(&mut cef_app),
            std::ptr::null_mut(),
        );
        assert_eq!(ret, -1, "cannot execute browser process");

        cef_initialize(&args, &mut cef_app);

        app.insert_non_send_resource(cef_app);
        app.insert_non_send_resource(MessageLoopWorkingReceiver(rx));
        app.insert_non_send_resource(RunOnMainThread)
            .add_systems(Main, cef_do_message_loop_work)
            .add_systems(Update, cef_shutdown.run_if(on_message::<AppExit>));
    }
}

#[cfg(target_os = "macos")]
fn load_cef_library(app: &mut App) {
    macos::install_cef_app_protocol();
    #[cfg(all(target_os = "macos", feature = "debug"))]
    let loader = DebugLibraryLoader::new();
    #[cfg(all(target_os = "macos", not(feature = "debug")))]
    let loader = cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
    assert!(loader.load());
    app.insert_non_send_resource(loader);
}

fn cef_initialize(args: &Args, cef_app: &mut cef::App) {
    let settings = Settings {
        #[cfg(all(target_os = "macos", feature = "debug"))]
        framework_dir_path: debug_chromium_embedded_framework_dir_path()
            .to_str()
            .unwrap()
            .into(),
        #[cfg(all(target_os = "macos", feature = "debug"))]
        browser_subprocess_path: debug_render_process_path().to_str().unwrap().into(),
        #[cfg(all(target_os = "macos", feature = "debug"))]
        no_sandbox: true as _,
        windowless_rendering_enabled: true as _,
        // #[cfg(any(target_os = "windows", target_os = "linux"))]
        // multi_threaded_message_loop: true as _,
        multi_threaded_message_loop: false as _,
        external_message_pump: true as _,
        ..Default::default()
    };
    assert_eq!(
        initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(cef_app),
            std::ptr::null_mut(),
        ),
        1
    );
}

fn cef_do_message_loop_work(
    receiver: NonSend<MessageLoopWorkingReceiver>,
    mut timer: Local<Option<MessageLoopTimer>>,
    mut max_delay_timer: Local<MessageLoopWorkingMaxDelayTimer>,
) {
    while let Ok(t) = receiver.try_recv() {
        timer.replace(t);
    }
    if timer.as_ref().map(|t| t.is_finished()).unwrap_or(false) || max_delay_timer.is_finished() {
        cef::do_message_loop_work();
        *max_delay_timer = MessageLoopWorkingMaxDelayTimer::default();
        timer.take();
    }
}

fn cef_shutdown(_: NonSend<RunOnMainThread>) {
    shutdown();
}

#[cfg(target_os = "macos")]
mod macos {
    use core::sync::atomic::AtomicBool;
    use objc::runtime::{Class, Object, Sel};
    use objc::{sel, sel_impl};
    use std::os::raw::c_char;
    use std::os::raw::c_void;
    use std::sync::atomic::Ordering;

    unsafe extern "C" {
        fn class_addMethod(
            cls: *const Class,
            name: Sel,
            imp: *const c_void,
            types: *const c_char,
        ) -> bool;
    }

    static IS_HANDLING_SEND_EVENT: AtomicBool = AtomicBool::new(false);

    extern "C" fn is_handling_send_event(_: &Object, _: Sel) -> bool {
        IS_HANDLING_SEND_EVENT.load(Ordering::Relaxed)
    }

    extern "C" fn set_handling_send_event(_: &Object, _: Sel, flag: bool) {
        IS_HANDLING_SEND_EVENT.swap(flag, Ordering::Relaxed);
    }

    pub fn install_cef_app_protocol() {
        unsafe {
            let cls = Class::get("NSApplication").expect("NSApplication クラスが見つかりません");
            #[allow(unexpected_cfgs)]
            let sel_name = sel!(isHandlingSendEvent);
            let success = class_addMethod(
                cls as *const _,
                sel_name,
                is_handling_send_event as *const c_void,
                c"c@:".as_ptr() as *const c_char,
            );
            assert!(success, "メソッド追加に失敗しました");

            #[allow(unexpected_cfgs)]
            let sel_set = sel!(setHandlingSendEvent:);
            let success2 = class_addMethod(
                cls as *const _,
                sel_set,
                set_handling_send_event as *const c_void,
                c"v@:c".as_ptr() as *const c_char,
            );
            assert!(
                success2,
                "Failed to add setHandlingSendEvent: to NSApplication"
            );
        }
    }
}
