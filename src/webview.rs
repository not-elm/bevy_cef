use crate::common::localhost::responser::{InlineHtmlId, InlineHtmlStore};
use crate::common::{
    HostWindow, IpcEventRawSender, ResolvedWebviewUri, WebviewSize, WebviewSource,
};
use crate::cursor_icon::SystemCursorIconSender;
use crate::prelude::PreloadScripts;
use crate::webview::mesh::MeshWebviewPlugin;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use bevy_cef_core::prelude::*;
use bevy_remote::BrpSender;
#[allow(deprecated)]
use raw_window_handle::HasRawWindowHandle;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(target_os = "windows")]
use crate::common::CommandChannelReceiver;
#[cfg(target_os = "windows")]
use crate::common::TextureSenderRes;

mod mesh;
pub(crate) mod webview_sprite;

pub mod prelude {
    pub use crate::webview::{
        BeginFrameInterval, RequestCloseDevtool, RequestShowDevTool, WebviewPlugin, mesh::*,
    };
}

/// A Trigger event to request showing the developer tools in a webview.
///
/// When you want to close the developer tools, use [`RequestCloseDevtool`].
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_cef::prelude::*;
///
/// #[derive(Component)]
/// struct DebugWebview;
///
/// fn show_devtool_system(mut commands: Commands, webviews: Query<Entity, With<DebugWebview>>) {
///     let entity = webviews.single().unwrap();
///     commands.entity(entity).trigger(|webview| RequestShowDevTool { webview });
/// }
/// ```
#[derive(Reflect, Debug, Copy, Clone, Serialize, Deserialize, EntityEvent)]
#[reflect(Serialize, Deserialize)]
pub struct RequestShowDevTool {
    #[event_target]
    pub webview: Entity,
}

/// A Trigger event to request closing the developer tools in a webview.
///
/// When showing the devtool, use [`RequestShowDevTool`] instead.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_cef::prelude::*;
///
/// #[derive(Component)]
/// struct DebugWebview;
///
/// fn close_devtool_system(mut commands: Commands, webviews: Query<Entity, With<DebugWebview>>) {
///     let entity = webviews.single().unwrap();
///     commands.entity(entity).trigger(|webview| RequestCloseDevtool { webview });
/// }
/// ```
#[derive(Reflect, Debug, Copy, Clone, Serialize, Deserialize, EntityEvent)]
#[reflect(Serialize, Deserialize)]
pub struct RequestCloseDevtool {
    #[event_target]
    pub webview: Entity,
}

/// Controls the interval between CEF external begin frame calls.
///
/// Defaults to ~30fps. Users can override by inserting this resource:
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_cef::prelude::*;
///
/// App::new()
///     .add_plugins(CefPlugin::default())
///     .insert_resource(BeginFrameInterval(core::time::Duration::from_millis(1000 / 60)));
/// ```
#[derive(Resource)]
pub struct BeginFrameInterval(pub Duration);

impl Default for BeginFrameInterval {
    fn default() -> Self {
        Self(Duration::from_millis(1000 / 30))
    }
}

/// System ordering for the webview lifecycle.
#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub enum WebviewSet {
    /// Resize drag tracking writes DisplaySize.
    ResizeInteraction,
    /// Derives WebviewSize from pipeline components.
    DerivePipeline,
    /// Creates CEF browser instances.
    CreateBrowser,
    /// Commits WebviewSize changes to CEF via browsers.resize().
    CommitResize,
}

pub struct WebviewPlugin;

impl Plugin for WebviewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RequestShowDevTool>();

        app.configure_sets(
            Update,
            (
                WebviewSet::ResizeInteraction,
                WebviewSet::DerivePipeline,
                WebviewSet::CreateBrowser,
                WebviewSet::CommitResize,
            )
                .chain(),
        );

        // macOS/Linux: direct NonSend<Browsers>
        #[cfg(not(target_os = "windows"))]
        {
            app.init_non_send_resource::<Browsers>()
                .init_resource::<BeginFrameInterval>()
                .add_plugins((MeshWebviewPlugin,))
                .add_systems(Main, send_external_begin_frame)
                .add_systems(
                    Update,
                    (
                        resize.run_if(any_resized).in_set(WebviewSet::CommitResize),
                        create_webview
                            .run_if(added_webview)
                            .in_set(WebviewSet::CreateBrowser),
                        navigate_on_source_change,
                    ),
                )
                .add_observer(apply_request_show_devtool)
                .add_observer(apply_request_close_devtool);
        }

        // Windows: BrowsersProxy already inserted by MessageLoopPlugin.
        // No send_external_begin_frame (CEF drives compositing).
        // Register conditional drain system that posts CefPostTask(TID_UI).
        #[cfg(target_os = "windows")]
        {
            app.add_plugins((MeshWebviewPlugin,));

            // Initialise the thread-local BrowsersCefSide on the CEF UI thread
            // with the texture sender so that created browsers can deliver
            // rendered frames back to Bevy.
            let texture_sender = app.world().resource::<TextureSenderRes>().0.clone();
            {
                use cef::rc::Rc;
                use cef::{ImplTask, Task, WrapTask};

                cef::wrap_task! {
                    struct InitCefBrowsersTask {
                        sender: async_channel::Sender<RenderTextureMessage>,
                    }
                    impl Task {
                        fn execute(&self) {
                            bevy_cef_core::prelude::init_cef_browsers(self.sender.clone());
                        }
                    }
                }
                let mut task = InitCefBrowsersTask::new(texture_sender);
                cef::post_task(cef::ThreadId::UI, Some(&mut task));
            }

            app.add_systems(Main, post_drain_task.run_if(win_commands_pending))
                .add_systems(
                    Update,
                    (
                        resize_win
                            .run_if(any_resized)
                            .in_set(WebviewSet::CommitResize),
                        create_webview_win
                            .run_if(added_webview)
                            .in_set(WebviewSet::CreateBrowser),
                        navigate_on_source_change_win,
                    ),
                )
                .add_observer(apply_request_show_devtool_win)
                .add_observer(apply_request_close_devtool_win);
        }

        // Platform-conditional despawn hook
        app.world_mut()
            .register_component_hooks::<WebviewSource>()
            .on_despawn(|world: DeferredWorld, ctx: HookContext| {
                #[cfg(not(target_os = "windows"))]
                {
                    let mut world = world;
                    world.non_send_resource_mut::<Browsers>().close(&ctx.entity);
                }
                #[cfg(target_os = "windows")]
                world.resource::<BrowsersProxy>().close(&ctx.entity);
            });

        app.world_mut()
            .register_component_hooks::<InlineHtmlId>()
            .on_remove(|mut world: DeferredWorld, ctx: HookContext| {
                let id = world.get::<InlineHtmlId>(ctx.entity).unwrap().0.clone();
                world.resource_mut::<InlineHtmlStore>().remove(&id);
            });
    }
}

fn any_resized(webviews: Query<Entity, Changed<WebviewSize>>) -> bool {
    !webviews.is_empty()
}

fn added_webview(webviews: Query<Entity, Added<ResolvedWebviewUri>>) -> bool {
    !webviews.is_empty()
}

#[cfg(not(target_os = "windows"))]
fn send_external_begin_frame(
    mut hosts: NonSendMut<Browsers>,
    time: Res<Time>,
    interval: Res<BeginFrameInterval>,
    mut timer: Local<Option<Timer>>,
) {
    if interval.is_changed() || timer.is_none() {
        *timer = Some(Timer::new(interval.0, TimerMode::Repeating));
    }
    let timer = timer.as_mut().unwrap();
    timer.tick(time.delta());
    if timer.just_finished() {
        hosts.send_external_begin_frame();
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(clippy::too_many_arguments)]
fn create_webview(
    mut browsers: NonSendMut<Browsers>,
    requester: Res<Requester>,
    ipc_event_sender: Res<IpcEventRawSender>,
    brp_sender: Res<BrpSender>,
    cursor_icon_sender: Res<SystemCursorIconSender>,
    drag_regions_sender: Res<crate::drag::DraggableRegionSender>,
    webviews: Query<
        (
            Entity,
            &ResolvedWebviewUri,
            &WebviewSize,
            &PreloadScripts,
            Option<&HostWindow>,
        ),
        Added<ResolvedWebviewUri>,
    >,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    WINIT_WINDOWS.with(|winit_windows| {
        let winit_windows = winit_windows.borrow();
        for (entity, uri, size, initialize_scripts, host_window) in webviews.iter() {
            let host_window = host_window
                .and_then(|w| winit_windows.get_window(w.0))
                .or_else(|| winit_windows.get_window(primary_window.single().ok()?))
                .and_then(|w| {
                    #[allow(deprecated)]
                    w.raw_window_handle().ok()
                });
            browsers.create_browser(
                entity,
                &uri.0,
                size.0,
                requester.clone(),
                ipc_event_sender.0.clone(),
                brp_sender.clone(),
                cursor_icon_sender.clone(),
                drag_regions_sender.0.clone(),
                &initialize_scripts.0,
                host_window,
            );
        }
    });
}

#[cfg(not(target_os = "windows"))]
fn navigate_on_source_change(
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &ResolvedWebviewUri), Changed<ResolvedWebviewUri>>,
    added: Query<Entity, Added<ResolvedWebviewUri>>,
) {
    for (entity, uri) in webviews.iter() {
        if added.contains(entity) {
            continue;
        }
        browsers.navigate(&entity, &uri.0);
    }
}

#[cfg(not(target_os = "windows"))]
fn resize(
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &WebviewSize), Changed<WebviewSize>>,
) {
    for (webview, size) in webviews.iter() {
        browsers.resize(&webview, size.0);
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_request_show_devtool(trigger: On<RequestShowDevTool>, browsers: NonSend<Browsers>) {
    browsers.show_devtool(&trigger.webview);
}

#[cfg(not(target_os = "windows"))]
fn apply_request_close_devtool(trigger: On<RequestCloseDevtool>, browsers: NonSend<Browsers>) {
    browsers.close_devtools(&trigger.webview);
}

#[cfg(target_os = "windows")]
fn win_commands_pending(proxy: Res<BrowsersProxy>) -> bool {
    !proxy.is_empty()
}

#[cfg(target_os = "windows")]
fn post_drain_task(rx: Res<CommandChannelReceiver>) {
    use cef::rc::Rc;
    use cef::{ImplTask, Task, WrapTask};

    let receiver = rx.0.clone();
    cef::wrap_task! {
        struct DrainTask {
            rx: async_channel::Receiver<CefCommand>,
        }

        impl Task {
            fn execute(&self) {
                bevy_cef_core::prelude::drain_commands(&self.rx);
            }
        }
    }
    let mut task = DrainTask::new(receiver);
    cef::post_task(cef::ThreadId::UI, Some(&mut task));
}

#[cfg(target_os = "windows")]
#[allow(clippy::too_many_arguments)]
fn create_webview_win(
    proxy: Res<BrowsersProxy>,
    requester: Res<Requester>,
    ipc_event_sender: Res<IpcEventRawSender>,
    brp_sender: Res<BrpSender>,
    cursor_icon_sender: Res<SystemCursorIconSender>,
    drag_regions_sender: Res<crate::drag::DraggableRegionSender>,
    webviews: Query<
        (
            Entity,
            &ResolvedWebviewUri,
            &WebviewSize,
            &PreloadScripts,
            Option<&HostWindow>,
        ),
        Added<ResolvedWebviewUri>,
    >,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    WINIT_WINDOWS.with(|winit_windows| {
        let winit_windows = winit_windows.borrow();
        for (entity, uri, size, initialize_scripts, host_window) in webviews.iter() {
            let host_window = host_window
                .and_then(|w| winit_windows.get_window(w.0))
                .or_else(|| winit_windows.get_window(primary_window.single().ok()?))
                .and_then(|w| {
                    #[allow(deprecated)]
                    w.raw_window_handle().ok()
                });
            proxy.create_browser(
                entity,
                &uri.0,
                size.0,
                requester.clone(),
                ipc_event_sender.0.clone(),
                brp_sender.clone(),
                cursor_icon_sender.clone(),
                drag_regions_sender.0.clone(),
                &initialize_scripts.0,
                host_window,
            );
        }
    });
}

#[cfg(target_os = "windows")]
fn navigate_on_source_change_win(
    proxy: Res<BrowsersProxy>,
    webviews: Query<(Entity, &ResolvedWebviewUri), Changed<ResolvedWebviewUri>>,
    added: Query<Entity, Added<ResolvedWebviewUri>>,
) {
    for (entity, uri) in webviews.iter() {
        if added.contains(entity) {
            continue;
        }
        proxy.navigate(&entity, &uri.0);
    }
}

#[cfg(target_os = "windows")]
fn resize_win(
    proxy: Res<BrowsersProxy>,
    webviews: Query<(Entity, &WebviewSize), Changed<WebviewSize>>,
) {
    for (webview, size) in webviews.iter() {
        proxy.resize(&webview, size.0);
    }
}

#[cfg(target_os = "windows")]
fn apply_request_show_devtool_win(trigger: On<RequestShowDevTool>, proxy: Res<BrowsersProxy>) {
    proxy.show_devtool(&trigger.webview);
}

#[cfg(target_os = "windows")]
fn apply_request_close_devtool_win(trigger: On<RequestCloseDevtool>, proxy: Res<BrowsersProxy>) {
    proxy.close_devtools(&trigger.webview);
}
