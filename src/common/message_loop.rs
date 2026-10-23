use crate::RunOnMainThread;
use crate::common::WebviewCoreComponentsPlugin;
use crate::common::ipc::js_emit::IpcRawEventPlugin;
use crate::focus::FocusedWebview;
use crate::keyboard::CefKeyboardFilter;
use crate::prelude::{
    BeginFrameInterval, WebviewExtendStandardMaterial, WebviewMaterial, WebviewUiMaterial,
};
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;
use bevy_cef_core::prelude::*;
use cef::args::Args;
use cef::{Settings, api_hash, execute_process, initialize, shutdown, sys};
use std::time::{Duration, Instant};

/// Why CEF did not start.
#[derive(Debug, Clone)]
pub enum CefStartupError {
    /// The root cache directory could not be created.
    RootCacheDir {
        /// The directory that could not be created.
        path: String,
        /// The I/O error message.
        error: String,
    },
    /// The CEF library could not be loaded.
    LibraryLoad,
    /// `cef_initialize` returned failure.
    Initialize {
        /// The process exit code CEF reported through `cef_get_exit_code`.
        exit_code: i32,
    },
}

impl CefStartupError {
    /// Whether another process holds or locks the root cache directory.
    ///
    /// True exactly for an [`Initialize`](Self::Initialize) failure whose exit code is
    /// `CEF_RESULT_CODE_PROFILE_IN_USE` or `CEF_RESULT_CODE_NORMAL_EXIT_PROCESS_NOTIFIED`.
    pub fn is_root_in_use(&self) -> bool {
        match self {
            Self::Initialize { exit_code } => {
                *exit_code == sys::cef_resultcode_t::CEF_RESULT_CODE_PROFILE_IN_USE as i32
                    || *exit_code
                        == sys::cef_resultcode_t::CEF_RESULT_CODE_NORMAL_EXIT_PROCESS_NOTIFIED
                            as i32
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for CefStartupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootCacheDir { path, error } => {
                write!(
                    f,
                    "failed to create root_cache_path directory '{path}': {error}"
                )
            }
            Self::LibraryLoad => write!(f, "failed to load the CEF library"),
            Self::Initialize { exit_code } => {
                write!(f, "cef_initialize failed with exit code {exit_code}")
            }
        }
    }
}

impl std::error::Error for CefStartupError {}

/// Whether CEF started. Inserted by [`CefPlugin`](crate::CefPlugin) during `build`.
#[derive(Resource, Debug, Clone)]
pub enum CefStatus {
    /// CEF initialized and every bevy_cef plugin was added.
    Running,
    /// CEF did not start; no plugin that calls into CEF was added.
    Failed(CefStartupError),
}

/// The `root_cache_path` the plugin was configured with.
#[derive(Resource)]
pub(crate) struct CefRootCachePath(pub(crate) Option<String>);

/// Present once bevy_cef has called `cef::shutdown()`. The message pump, the shutdown system and
/// browser creation do not run while it is present.
#[derive(Resource)]
pub(crate) struct CefShutDown;

/// Controls the CEF message loop.
///
/// Every platform runs CEF in `external_message_pump` mode: CEF asks for work through
/// `on_schedule_message_pump_work`, and [`CefDoMessageLoopWork`](https://cef-builds.spotifycdn.com/docs/106.1/cef__app_8h.html#a830ae43dcdffcf4e719540204cefdb61)
/// runs from a system in the `Main` schedule (throttled to a 4 ms minimum interval, with a
/// 30 Hz max-delay fallback). The CEF UI thread is therefore the Bevy main thread, which is
/// why `Browsers` is a `NonSend` resource.
///
/// `multi_threaded_message_loop` is not used: CEF does not support it on macOS, and
/// `cef_do_message_loop_work` does not block.
pub struct MessageLoopPlugin {
    pub config: CommandLineConfig,
    pub extensions: CefExtensions,
    pub root_cache_path: Option<String>,
    pub no_sandbox: bool,
    pub cef_path: Option<std::path::PathBuf>,
    pub subprocess_path: Option<std::path::PathBuf>,
}

impl MessageLoopPlugin {
    /// Runs CEF initialization and inserts the `cef::App` and message-pump receiver as
    /// non-send resources.
    ///
    /// # Errors
    ///
    /// Returns a [`CefStartupError`] when the root cache directory cannot be created, the CEF
    /// library cannot be loaded (macOS), or `cef_initialize` fails.
    pub(crate) fn start(&self, app: &mut App) -> Result<(), CefStartupError> {
        let (custom_framework_dir, custom_framework_binary, custom_subprocess) =
            resolve_custom_cef_paths(self.cef_path.as_deref(), self.subprocess_path.as_deref());

        #[cfg(not(target_os = "macos"))]
        let render_process_binary = custom_subprocess.or_else(render_process_path);

        #[cfg(target_os = "macos")]
        load_cef_library(app, custom_framework_binary.as_deref())?;

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
            self.no_sandbox,
        )?;
        #[cfg(target_os = "macos")]
        cef_initialize(
            &args,
            &mut cef_app,
            self.root_cache_path.as_deref(),
            custom_framework_dir.as_deref(),
            custom_subprocess.as_deref(),
            self.no_sandbox,
        )?;

        app.insert_non_send(cef_app);
        app.insert_non_send(MessageLoopWorkingReceiver(rx));
        Ok(())
    }
}

impl Plugin for MessageLoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Main,
            cef_do_message_loop_work.run_if(not(resource_exists::<CefShutDown>)),
        );

        #[cfg(all(target_os = "macos", feature = "debug"))]
        app.add_systems(
            Main,
            macos::observe_terminate_request.before(cef_do_message_loop_work),
        );

        app.insert_non_send(RunOnMainThread);
        register_on_app_exit(
            app,
            shutdown_cef_on_exit.run_if(not(resource_exists::<CefShutDown>)),
        );
    }
}

/// Adds `system` to `Main` after `Main::run_main` and the CEF message pump, running only in a
/// frame in which an `AppExit` was written.
///
/// Such a system sees an `AppExit` written anywhere in the frame, `Last` included, before the
/// runner exits without another frame.
pub(crate) fn register_on_app_exit<M>(
    app: &mut App,
    system: impl IntoScheduleConfigs<ScheduleSystem, M>,
) {
    app.add_systems(
        Main,
        system
            .after(Main::run_main)
            .after(cef_do_message_loop_work)
            .run_if(on_message::<AppExit>),
    );
}

/// Adds the minimal state an app needs when CEF failed to start: the failure in [`CefStatus`],
/// the [`CefRootCachePath`], and the resources and messages that user-added bevy_cef plugins
/// read, and the public resources, markers and assets an app's own systems can name. No plugin
/// that calls into CEF and no system is added; the assets are registered only when `AssetPlugin`
/// is present.
pub(crate) fn add_failed_startup(app: &mut App, error: CefStartupError, root: Option<String>) {
    app.insert_resource(CefStatus::Failed(error))
        .insert_resource(CefRootCachePath(root))
        .add_plugins((WebviewCoreComponentsPlugin, IpcRawEventPlugin))
        .add_message::<RenderTextureMessage>()
        .init_resource::<FocusedWebview>()
        .init_resource::<CefKeyboardFilter>()
        .init_resource::<BeginFrameInterval>()
        .insert_non_send(RunOnMainThread);

    // NOTE: `init_asset` panics without an `AssetServer`; the skipped plugins assume
    // `AssetPlugin` as well, but a failed start must not add a second panic.
    if app.world().contains_resource::<AssetServer>() {
        app.init_asset::<WebviewMaterial>()
            .init_asset::<WebviewExtendStandardMaterial>()
            .init_asset::<WebviewUiMaterial>()
            .init_asset::<CefResponse>();
    }
}

#[cfg(target_os = "macos")]
pub enum MacLibraryLoader {
    Custom(CustomLibraryLoader),
    #[cfg(not(feature = "debug"))]
    Default(cef::library_loader::LibraryLoader),
    #[cfg(feature = "debug")]
    Debug(DebugLibraryLoader),
}

#[cfg(target_os = "macos")]
impl MacLibraryLoader {
    pub fn load(&self) -> bool {
        match self {
            Self::Custom(l) => l.load(),
            #[cfg(not(feature = "debug"))]
            Self::Default(l) => l.load(),
            #[cfg(feature = "debug")]
            Self::Debug(l) => l.load(),
        }
    }
}

#[cfg(target_os = "macos")]
fn load_cef_library(
    app: &mut App,
    custom_framework_binary: Option<&std::path::Path>,
) -> Result<(), CefStartupError> {
    macos::install_cef_app_protocol();

    let loader = if let Some(bin) = custom_framework_binary {
        MacLibraryLoader::Custom(CustomLibraryLoader::new(bin.to_path_buf()))
    } else {
        #[cfg(all(target_os = "macos", feature = "debug"))]
        {
            MacLibraryLoader::Debug(DebugLibraryLoader::new())
        }
        #[cfg(all(target_os = "macos", not(feature = "debug")))]
        {
            let exe = std::env::current_exe().map_err(|_| CefStartupError::LibraryLoad)?;
            MacLibraryLoader::Default(cef::library_loader::LibraryLoader::new(&exe, false))
        }
    };

    if !loader.load() {
        return Err(CefStartupError::LibraryLoad);
    }
    app.insert_non_send(loader);
    Ok(())
}

fn ensure_root_cache_dir(root_cache_path: Option<&str>) -> Result<(), CefStartupError> {
    // Empty/whitespace paths are valid (CEF treats them as "use default"), so skip those.
    if let Some(path) = root_cache_path.filter(|p| !p.trim().is_empty()) {
        std::fs::create_dir_all(path).map_err(|e| CefStartupError::RootCacheDir {
            path: path.to_string(),
            error: e.to_string(),
        })?;
    }
    Ok(())
}

fn initialize_result(initialized: i32) -> Result<(), CefStartupError> {
    if initialized == 1 {
        Ok(())
    } else {
        Err(CefStartupError::Initialize {
            exit_code: cef::get_exit_code(),
        })
    }
}

#[cfg(target_os = "macos")]
fn cef_initialize(
    args: &Args,
    cef_app: &mut cef::App,
    root_cache_path: Option<&str>,
    custom_framework_dir: Option<&std::path::Path>,
    custom_subprocess: Option<&std::path::Path>,
    no_sandbox: bool,
) -> Result<(), CefStartupError> {
    ensure_root_cache_dir(root_cache_path)?;

    #[cfg(feature = "debug")]
    let default_fw_dir = debug_chromium_embedded_framework_dir_path();
    #[cfg(not(feature = "debug"))]
    let default_fw_dir = std::path::PathBuf::new();

    #[cfg(feature = "debug")]
    let default_subprocess = debug_render_process_path();
    #[cfg(not(feature = "debug"))]
    let default_subprocess = std::path::PathBuf::new();

    let fw_dir_str = custom_framework_dir
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| default_fw_dir.to_string_lossy().to_string());

    let subprocess_str = custom_subprocess
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| default_subprocess.to_string_lossy().to_string());

    let settings = Settings {
        framework_dir_path: fw_dir_str.as_str().into(),
        browser_subprocess_path: subprocess_str.as_str().into(),
        no_sandbox: no_sandbox as _,
        root_cache_path: root_cache_path.unwrap_or_default().into(),
        windowless_rendering_enabled: true as _,
        external_message_pump: true as _,
        disable_signal_handlers: true as _,
        ..Default::default()
    };
    initialize_result(initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(cef_app),
        std::ptr::null_mut(),
    ))
}

#[cfg(not(target_os = "macos"))]
fn cef_initialize(
    args: &Args,
    cef_app: &mut cef::App,
    root_cache_path: Option<&str>,
    render_process_binary: Option<&std::path::Path>,
    no_sandbox: bool,
) -> Result<(), CefStartupError> {
    ensure_root_cache_dir(root_cache_path)?;

    let subprocess_path: String = render_process_binary
        .and_then(|p| p.to_str())
        .unwrap_or_default()
        .into();

    let settings = Settings {
        browser_subprocess_path: subprocess_path.as_str().into(),
        no_sandbox: no_sandbox as _,
        root_cache_path: root_cache_path.unwrap_or_default().into(),
        windowless_rendering_enabled: true as _,
        external_message_pump: true as _,
        disable_signal_handlers: false as _,
        ..Default::default()
    };
    initialize_result(initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(cef_app),
        std::ptr::null_mut(),
    ))
}

fn cef_do_message_loop_work(
    receiver: NonSend<MessageLoopWorkingReceiver>,
    mut timer: Local<Option<MessageLoopTimer>>,
    mut max_delay_timer: Local<MessageLoopWorkingMaxDelayTimer>,
    mut last_execution: Local<Option<std::time::Instant>>,
) {
    while let Ok(t) = receiver.try_recv() {
        timer.replace(t);
    }
    let should_execute =
        timer.as_ref().map(|t| t.is_finished()).unwrap_or(false) || max_delay_timer.is_finished();
    if should_execute {
        // Enforce a minimum interval between executions to prevent
        // delay_ms=0 requests from causing excessive pump calls.
        const MIN_PUMP_INTERVAL: std::time::Duration = std::time::Duration::from_millis(4);
        let now = std::time::Instant::now();
        if let Some(last) = *last_execution
            && now.duration_since(last) < MIN_PUMP_INTERVAL
        {
            return;
        }
        *last_execution = Some(now);
        cef::do_message_loop_work();
        *max_delay_timer = MessageLoopWorkingMaxDelayTimer::default();
        timer.take();
    }
}

/// Closes every browser, waits up to 3 s for CEF to report them closed, releases bevy_cef's CEF
/// references and shuts CEF down, all within the exiting frame.
///
/// Browsers still open at the time limit are logged with `warn!`, and the shutdown continues.
fn shutdown_cef_on_exit(mut browsers: NonSendMut<Browsers>, mut commands: Commands) {
    const SHUTDOWN_TIME_LIMIT: Duration = Duration::from_secs(3);
    const PUMP_INTERVAL: Duration = Duration::from_millis(5);

    browsers.close_all();
    let deadline = Instant::now() + SHUTDOWN_TIME_LIMIT;
    while browsers.live_browser_count() > 0 {
        if Instant::now() >= deadline {
            warn!(
                "bevy_cef: {} browser(s) still open after {SHUTDOWN_TIME_LIMIT:?}; shutting CEF down anyway",
                browsers.live_browser_count()
            );
            break;
        }
        cef::do_message_loop_work();
        std::thread::sleep(PUMP_INTERVAL);
    }
    browsers.release_all();
    shutdown();
    commands.insert_resource(CefShutDown);
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

    #[cfg(feature = "debug")]
    use bevy::prelude::{AppExit, MessageWriter, info};

    #[cfg(feature = "debug")]
    use objc::runtime::Method;

    #[cfg(feature = "debug")]
    unsafe extern "C" {
        fn class_getInstanceMethod(cls: *const Class, sel: Sel) -> *mut Method;
        fn method_exchangeImplementations(m1: *mut Method, m2: *mut Method);
    }

    static IS_HANDLING_SEND_EVENT: AtomicBool = AtomicBool::new(false);

    #[cfg(feature = "debug")]
    static TERMINATE_REQUESTED: AtomicBool = AtomicBool::new(false);

    extern "C" fn is_handling_send_event(_: &Object, _: Sel) -> bool {
        IS_HANDLING_SEND_EVENT.load(Ordering::Relaxed)
    }

    extern "C" fn set_handling_send_event(_: &Object, _: Sel, flag: bool) {
        IS_HANDLING_SEND_EVENT.swap(flag, Ordering::Relaxed);
    }

    #[cfg(feature = "debug")]
    extern "C" fn swizzled_terminate(_: &Object, _: Sel, _sender: *mut Object) {
        // Intentionally does NOT call the original terminate:.
        // Calling it would post NSApplicationWillTerminateNotification and
        // re-trigger the winit `applicationWillTerminate:` panic.
        TERMINATE_REQUESTED.store(true, Ordering::Relaxed);
    }

    #[cfg(feature = "debug")]
    fn install_terminate_swizzle() {
        unsafe {
            let cls = Class::get("NSApplication").expect("NSApplication class not found");

            let placeholder_sel = sel!(cef_swizzled_terminate:);
            let added = class_addMethod(
                cls as *const _,
                placeholder_sel,
                swizzled_terminate as *const c_void,
                c"v@:@".as_ptr() as *const c_char,
            );
            assert!(
                added,
                "Failed to add cef_swizzled_terminate: to NSApplication"
            );

            let terminate_method = class_getInstanceMethod(cls as *const _, sel!(terminate:));
            assert!(
                !terminate_method.is_null(),
                "terminate: method not found on NSApplication"
            );
            let swizzled_method = class_getInstanceMethod(cls as *const _, placeholder_sel);
            assert!(
                !swizzled_method.is_null(),
                "cef_swizzled_terminate: method not found after class_addMethod"
            );

            method_exchangeImplementations(terminate_method, swizzled_method);
        }
    }

    pub fn install_cef_app_protocol() {
        unsafe {
            let cls = Class::get("NSApplication").expect("NSApplication class not found");
            let sel_name = sel!(isHandlingSendEvent);
            let success = class_addMethod(
                cls as *const _,
                sel_name,
                is_handling_send_event as *const c_void,
                c"c@:".as_ptr() as *const c_char,
            );
            assert!(
                success,
                "Failed to add isHandlingSendEvent to NSApplication"
            );

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

            #[cfg(feature = "debug")]
            install_terminate_swizzle();
        }
    }

    /// `swap(false, Relaxed)` atomically reads-and-clears the flag, so AppExit
    /// is emitted exactly once even if this system runs again before shutdown
    /// completes.
    #[cfg(feature = "debug")]
    pub(super) fn observe_terminate_request(mut writer: MessageWriter<AppExit>) {
        if TERMINATE_REQUESTED.swap(false, Ordering::Relaxed) {
            info!("Termination intercepted, requesting AppExit");
            writer.write(AppExit::from_code(130));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::JsEmitEventPlugin;

    /// Asserts that only an `Initialize` failure with exit code 21 or 24 reports the root as in
    /// use, and every other error does not.
    ///
    /// Case: a second process starts against a root cache directory that the first process
    /// already holds, and CEF exits initialization with its profile-in-use or
    /// process-notified code.
    #[test]
    fn root_in_use_covers_codes_21_and_24() {
        assert!(CefStartupError::Initialize { exit_code: 21 }.is_root_in_use());
        assert!(CefStartupError::Initialize { exit_code: 24 }.is_root_in_use());
        assert!(!CefStartupError::Initialize { exit_code: 1 }.is_root_in_use());
        assert!(!CefStartupError::LibraryLoad.is_root_in_use());
        assert!(
            !CefStartupError::RootCacheDir {
                path: "p".into(),
                error: "e".into(),
            }
            .is_root_in_use()
        );
    }

    /// Asserts that a failed startup publishes `CefStatus::Failed`, registers no CEF-backed
    /// state, and lets the app keep updating.
    ///
    /// Case: an app that also added `JsEmitEventPlugin` starts while another process holds the
    /// root cache directory, so `cef_initialize` returned 0.
    #[test]
    fn a_failed_start_adds_no_cef_systems() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        add_failed_startup(
            &mut app,
            CefStartupError::Initialize { exit_code: 21 },
            Some("root".into()),
        );
        app.add_plugins(JsEmitEventPlugin::<serde_json::Value>::default());

        assert!(matches!(
            app.world().resource::<CefStatus>(),
            CefStatus::Failed(_)
        ));
        assert!(!app.world().contains_non_send::<Browsers>());
        app.update();
    }

    /// Asserts that after a failed start, a Startup system written like the examples can still
    /// take the webview material assets and the keyboard filter, so `update` does not panic.
    ///
    /// Case: an app with `AssetPlugin` whose setup system spawns a webview mesh with
    /// `ResMut<Assets<WebviewExtendStandardMaterial>>` runs while another process holds the root
    /// cache directory.
    #[test]
    fn a_failed_start_keeps_example_startup_systems_running() {
        fn setup(
            _materials: ResMut<Assets<WebviewExtendStandardMaterial>>,
            _ui_materials: ResMut<Assets<WebviewUiMaterial>>,
            _filter: Res<CefKeyboardFilter>,
            _interval: Res<BeginFrameInterval>,
            _marker: NonSend<RunOnMainThread>,
        ) {
        }

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        add_failed_startup(
            &mut app,
            CefStartupError::Initialize { exit_code: 21 },
            None,
        );
        app.add_systems(Startup, setup);

        app.update();
    }

    /// Asserts that a system registered with `register_on_app_exit` runs in the same update in
    /// which a `Last` system writes `AppExit`.
    ///
    /// Case: the user closes the last window, so Bevy's `exit_on_all_closed` writes
    /// `AppExit::Success` from `Last`, and the winit runner exits without another frame.
    #[test]
    fn an_app_exit_written_in_last_reaches_the_exit_system() {
        #[derive(Resource, Default)]
        struct ExitSystemRan(bool);

        fn mark_ran(mut ran: ResMut<ExitSystemRan>) {
            ran.0 = true;
        }

        fn write_exit(mut exit: MessageWriter<AppExit>) {
            exit.write(AppExit::Success);
        }

        let mut app = App::new();
        app.init_resource::<ExitSystemRan>()
            .add_systems(Last, write_exit);
        register_on_app_exit(&mut app, mark_ran);

        app.update();

        assert!(app.world().resource::<ExitSystemRan>().0);
    }
}
