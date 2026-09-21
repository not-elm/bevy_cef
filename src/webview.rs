use crate::common::localhost::responser::{InlineHtmlId, InlineHtmlStore};
use crate::common::{
    HostWindow, IpcEventRawSender, ResolvedWebviewUri, WebviewDpr, WebviewSize, WebviewSource,
};
use crate::cursor_icon::SystemCursorIconSender;
use crate::prelude::PreloadScripts;
use crate::webview::mesh::MeshWebviewPlugin;
use crate::webview::ui::UiWebviewPlugin;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::mouse::MouseScrollUnit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use bevy_cef_core::prelude::*;
use bevy_remote::BrpSender;
#[allow(deprecated)]
use raw_window_handle::HasRawWindowHandle;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub(crate) mod alpha;
// [macos-gpu-osr] Injects the owned CEF webview GPU texture into RenderAssets<GpuImage>.
#[cfg(target_os = "macos")]
pub(crate) mod gpu_surface;
mod mesh;
pub mod texture_target;
mod ui;
pub(crate) mod webview_sprite;

pub mod prelude {
    pub use crate::webview::{
        BeginFrameInterval, RequestCloseDevtool, RequestShowDevTool, WebviewPlugin, mesh::*,
        texture_target::*, ui::WebviewUiMaterial,
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
///
/// Has no effect on Windows: there CEF drives compositing itself at 60 Hz, so no
/// external begin frames are sent.
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
    /// Seeds and refreshes WebviewDpr from host window scale factors.
    DpiSeed,
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
                WebviewSet::DpiSeed,
                WebviewSet::DerivePipeline,
                WebviewSet::CreateBrowser,
                WebviewSet::CommitResize,
            )
                .chain(),
        );

        app.init_non_send::<Browsers>()
            .init_resource::<BeginFrameInterval>()
            .add_plugins((MeshWebviewPlugin, UiWebviewPlugin))
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

        // Windows lets CEF drive compositing; see `EXTERNAL_BEGIN_FRAME`.
        if EXTERNAL_BEGIN_FRAME {
            app.add_systems(Main, send_external_begin_frame);
        }

        #[cfg(target_os = "macos")]
        app.add_plugins(crate::webview::gpu_surface::WebviewGpuInjectPlugin);

        app.world_mut()
            .register_component_hooks::<WebviewSource>()
            .on_despawn(|mut world: DeferredWorld, ctx: HookContext| {
                world.non_send_mut::<Browsers>().close(&ctx.entity);
            });

        app.world_mut()
            .register_component_hooks::<InlineHtmlId>()
            .on_remove(|mut world: DeferredWorld, ctx: HookContext| {
                let id = world.get::<InlineHtmlId>(ctx.entity).unwrap().0.clone();
                world.resource_mut::<InlineHtmlStore>().remove(&id);
            });
    }
}

/// Converts a mouse-wheel delta into the pixel deltas CEF expects.
/// Chromium's default line height is 3 lines × 40px = 120px per notch.
pub(crate) fn scroll_delta(unit: MouseScrollUnit, x: f32, y: f32) -> Vec2 {
    match unit {
        MouseScrollUnit::Line => Vec2::new(x * 120.0, y * 120.0),
        MouseScrollUnit::Pixel => Vec2::new(x, y),
    }
}

fn any_resized(webviews: Query<Entity, Changed<WebviewSize>>) -> bool {
    !webviews.is_empty()
}

fn added_webview(webviews: Query<Entity, Added<ResolvedWebviewUri>>) -> bool {
    !webviews.is_empty()
}

fn send_external_begin_frame(
    browsers: NonSend<Browsers>,
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
        browsers.send_external_begin_frame();
    }
}

#[allow(clippy::too_many_arguments)]
fn create_webview(
    mut browsers: NonSendMut<Browsers>,
    requester: Res<Requester>,
    ipc_event_sender: Res<IpcEventRawSender>,
    brp_sender: Res<BrpSender>,
    cursor_icon_sender: Res<SystemCursorIconSender>,
    drag_regions_sender: Res<crate::drag::DraggableRegionSender>,
    load_handler_sender: Res<crate::navigation::LoadHandlerSender>,
    address_changed_sender: Res<crate::navigation::AddressChangedSender>,
    title_changed_sender: Res<crate::title::TitleChangedSender>,
    webviews: Query<
        (
            Entity,
            &ResolvedWebviewUri,
            &WebviewSize,
            &WebviewDpr,
            &PreloadScripts,
            Option<&HostWindow>,
        ),
        Added<ResolvedWebviewUri>,
    >,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    WINIT_WINDOWS.with(|winit_windows| {
        let winit_windows = winit_windows.borrow();
        for (entity, uri, size, dpr, initialize_scripts, host_window) in webviews.iter() {
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
                dpr.0,
                requester.clone(),
                ipc_event_sender.0.clone(),
                brp_sender.clone(),
                cursor_icon_sender.clone(),
                drag_regions_sender.0.clone(),
                load_handler_sender.0.clone(),
                address_changed_sender.0.clone(),
                title_changed_sender.0.clone(),
                &initialize_scripts.0,
                host_window,
            );
        }
    });
}

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

fn resize(
    browsers: NonSend<Browsers>,
    webviews: Query<(Entity, &WebviewSize), Changed<WebviewSize>>,
) {
    for (webview, size) in webviews.iter() {
        browsers.resize(&webview, size.0);
    }
}

fn apply_request_show_devtool(trigger: On<RequestShowDevTool>, browsers: NonSend<Browsers>) {
    browsers.show_devtool(&trigger.webview);
}

fn apply_request_close_devtool(trigger: On<RequestCloseDevtool>, browsers: NonSend<Browsers>) {
    browsers.close_devtools(&trigger.webview);
}
