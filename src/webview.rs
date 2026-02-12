use crate::common::localhost::responser::{InlineHtmlId, InlineHtmlStore};
use crate::common::{CefWebviewUri, HostWindow, IpcEventRawSender, WebviewSize};
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
                &initialize_scripts.0,
                host_window,
            );
        }
    });
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
