use crate::common::{CefWebviewUri, HostWindow, IpcEventRawSender, WebviewSize};
use crate::cursor_icon::SystemCursorIconSender;
use crate::prelude::PreloadScripts;
use crate::webview::mesh::MeshWebviewPlugin;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy_cef_core::prelude::*;
use bevy_remote::BrpSender;
#[allow(deprecated)]
use raw_window_handle::HasRawWindowHandle;
use serde::{Deserialize, Serialize};

mod mesh;
mod webview_sprite;

pub mod prelude {
    pub use crate::webview::{RequestCloseDevtool, RequestShowDevTool, WebviewPlugin, mesh::*};
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
///     commands.entity(webviews.single().unwrap()).trigger(RequestShowDevTool);
/// }
/// ```
#[derive(Reflect, Debug, Default, Copy, Clone, Serialize, Deserialize, Event)]
#[reflect(Default, Serialize, Deserialize)]
pub struct RequestShowDevTool;

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
///    commands.entity(webviews.single().unwrap()).trigger(RequestCloseDevtool);
/// }
/// ```
#[derive(Reflect, Debug, Default, Copy, Clone, Serialize, Deserialize, Event)]
#[reflect(Default, Serialize, Deserialize)]
pub struct RequestCloseDevtool;

pub struct WebviewPlugin;

impl Plugin for WebviewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RequestShowDevTool>()
            .init_non_send_resource::<Browsers>()
            .add_plugins((MeshWebviewPlugin,))
            .add_systems(Main, send_external_begin_frame)
            .add_systems(
                Update,
                (
                    resize.run_if(any_resized),
                    create_webview.run_if(added_webview),
                ),
            )
            .add_observer(apply_request_show_devtool)
            .add_observer(apply_request_close_devtool);

        app.world_mut()
            .register_component_hooks::<CefWebviewUri>()
            .on_despawn(|mut world: DeferredWorld, ctx: HookContext| {
                world.non_send_resource_mut::<Browsers>().close(&ctx.entity);
            });
    }
}

fn any_resized(webviews: Query<Entity, Changed<WebviewSize>>) -> bool {
    !webviews.is_empty()
}

fn added_webview(webviews: Query<Entity, Added<CefWebviewUri>>) -> bool {
    !webviews.is_empty()
}

fn send_external_begin_frame(mut hosts: NonSendMut<Browsers>) {
    hosts.send_external_begin_frame();
}

#[allow(clippy::too_many_arguments)]
fn create_webview(
    mut browsers: NonSendMut<Browsers>,
    requester: Res<Requester>,
    ipc_event_sender: Res<IpcEventRawSender>,
    brp_sender: Res<BrpSender>,
    cursor_icon_sender: Res<SystemCursorIconSender>,
    winit_windows: NonSend<WinitWindows>,
    webviews: Query<
        (
            Entity,
            &CefWebviewUri,
            &WebviewSize,
            &PreloadScripts,
            Option<&HostWindow>,
        ),
        Added<CefWebviewUri>,
    >,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
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
            &initialize_scripts.0,
            host_window,
        );
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

fn apply_request_show_devtool(trigger: Trigger<RequestShowDevTool>, browsers: NonSend<Browsers>) {
    browsers.show_devtool(&trigger.target());
}

fn apply_request_close_devtool(trigger: Trigger<RequestCloseDevtool>, browsers: NonSend<Browsers>) {
    browsers.close_devtools(&trigger.target());
}
