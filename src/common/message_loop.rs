use crate::RunOnMainThread;
use bevy::prelude::*;
use bevy_cef_core::prelude::*;
use cef::args::Args;
use cef::{Settings, api_hash, execute_process, initialize, shutdown, sys};

/// Controls the CEF message loop.
///
/// Uses `external_message_pump` on all platforms and calls
/// [`CefDoMessageLoopWork`](https://cef-builds.spotifycdn.com/docs/106.1/cef__app_8h.html#a830ae43dcdffcf4e719540204cefdb61)
/// every frame.
pub struct MessageLoopPlugin {
    pub config: CommandLineConfig,
    pub extensions: CefExtensions,
    pub root_cache_path: Option<String>,
}

impl Plugin for MessageLoopPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_os = "macos"))]
        let render_process_binary = render_process_path();

        #[cfg(target_os = "macos")]
        load_cef_library(app);

        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
        let args = Args::new();
        let (tx, rx) = std::sync::mpsc::channel();

        let mut cef_app =
            BrowserProcessAppBuilder::build(tx, self.config.clone(), self.extensions.clone());

        // On macOS and when a separate render process binary is available,
        // execute_process is called here. For the browser process it returns -1
        // and falls through; subprocesses exit immediately.
        #[cfg(target_os = "macos")]
        {
            let ret = execute_process(
                Some(args.as_main_args()),
                Some(&mut cef_app),
                std::ptr::null_mut(),
            );
            if ret >= 0 {
                std::process::exit(ret);
            }
        }

        #[cfg(not(target_os = "macos"))]
        cef_initialize(
            &args,
            &mut cef_app,
            self.root_cache_path.as_deref(),
            render_process_binary.as_deref(),
        );
        #[cfg(target_os = "macos")]
        cef_initialize(&args, &mut cef_app, self.root_cache_path.as_deref());

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

#[cfg(target_os = "macos")]
fn cef_initialize(args: &Args, cef_app: &mut cef::App, root_cache_path: Option<&str>) {
    // Ensure the cache directory exists before CEF tries to use it.
    // Empty/whitespace paths are valid (CEF treats them as "use default"), so skip those.
    if let Some(path) = root_cache_path.filter(|p| !p.trim().is_empty()) {
        std::fs::create_dir_all(path)
            .unwrap_or_else(|e| panic!("failed to create root_cache_path directory '{path}': {e}"));
    }

    let settings = Settings {
        #[cfg(feature = "debug")]
        framework_dir_path: debug_chromium_embedded_framework_dir_path()
            .to_str()
            .unwrap()
            .into(),
        #[cfg(feature = "debug")]
        browser_subprocess_path: debug_render_process_path().to_str().unwrap().into(),
        #[cfg(feature = "debug")]
        no_sandbox: true as _,
        root_cache_path: root_cache_path.unwrap_or_default().into(),
        windowless_rendering_enabled: true as _,
        external_message_pump: true as _,
        disable_signal_handlers: true as _,
        ..Default::default()
    };
    assert_eq!(
        initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(cef_app),
            std::ptr::null_mut(),
        ),
        1,
        "cef_initialize failed: root_cache_path={root_cache_path:?}",
    );
}

#[cfg(not(target_os = "macos"))]
fn cef_initialize(
    args: &Args,
    cef_app: &mut cef::App,
    root_cache_path: Option<&str>,
    render_process_binary: Option<&std::path::Path>,
) {
    // Ensure the cache directory exists before CEF tries to use it.
    // Empty/whitespace paths are valid (CEF treats them as "use default"), so skip those.
    if let Some(path) = root_cache_path.filter(|p| !p.trim().is_empty()) {
        std::fs::create_dir_all(path)
            .unwrap_or_else(|e| panic!("failed to create root_cache_path directory '{path}': {e}"));
    }

    let subprocess_path: String = render_process_binary
        .and_then(|p| p.to_str())
        .unwrap_or_default()
        .into();

    let settings = Settings {
        browser_subprocess_path: subprocess_path.as_str().into(),
        no_sandbox: true as _,
        root_cache_path: root_cache_path.unwrap_or_default().into(),
        windowless_rendering_enabled: true as _,
        external_message_pump: true as _,
        disable_signal_handlers: false as _,
        ..Default::default()
    };
    assert_eq!(
        initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(cef_app),
            std::ptr::null_mut(),
        ),
        1,
        "cef_initialize failed: root_cache_path={root_cache_path:?}, subprocess={subprocess_path:?}",
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

#[allow(clippy::needless_doctest_main)]
/// On non-macOS platforms, this detects if the current process is a CEF subprocess
/// (renderer, GPU, utility) and exits immediately if so.
///
/// When no separate render process binary is installed, CEF re-launches the main
/// executable as a subprocess. Call this function at the very beginning of `main()`
/// — **before** any Bevy initialization — so that subprocess instances exit
/// immediately without creating a visible window.
///
/// ```no_run
/// fn main() {
///     bevy_cef::prelude::early_exit_if_subprocess();
///     // ... Bevy App setup ...
/// }
/// ```
///
/// If a dedicated render process binary (`bevy_cef_render_process`) is installed
/// next to your executable, this function is unnecessary because CEF will launch
/// that binary instead of re-using the main executable.
///
/// On macOS this function is not available; macOS always uses a separate render
/// process binary.
#[cfg(not(target_os = "macos"))]
pub fn early_exit_if_subprocess() {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = Args::new();
    let mut app = RenderProcessAppBuilder::build();
    let ret = execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    );
    if ret >= 0 {
        std::process::exit(ret);
    }
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
            let sel_name = sel!(isHandlingSendEvent);
            let success = class_addMethod(
                cls as *const _,
                sel_name,
                is_handling_send_event as *const c_void,
                c"c@:".as_ptr() as *const c_char,
            );
            assert!(success, "メソッド追加に失敗しました");

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
