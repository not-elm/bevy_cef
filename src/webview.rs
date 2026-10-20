use crate::common::localhost::responser::{InlineHtmlId, InlineHtmlStore};
use crate::common::{
    CefShutDown, HostWindow, IpcEventRawSender, ResolvedWebviewUri, WebviewDpr, WebviewSize,
    WebviewSource, ZoomLevel,
};
use crate::cursor_icon::SystemCursorIconSender;
use crate::drag::DraggableRegionSender;
use crate::navigation::{AddressChangedSender, LoadHandlerSender};
use crate::prelude::{PreloadScripts, WebviewProfile};
use crate::title::TitleChangedSender;
use crate::webview::mesh::MeshWebviewPlugin;
use crate::webview::ui::UiWebviewPlugin;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::input::mouse::MouseScrollUnit;
use bevy::prelude::*;
use bevy::shader::load_shader_library;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;
use bevy_cef_core::prelude::*;
use bevy_remote::BrpSender;
#[allow(deprecated)]
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
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
        load_shader_library!(app, "webview/util.wesl");
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
                        .run_if(not(resource_exists::<CefShutDown>))
                        .in_set(WebviewSet::CreateBrowser),
                    // NOTE: Must run before browser creation. A profile webview's browser is
                    // created with the latest of its source and its stored `RequestNavigate`;
                    // navigating it in the same frame would replace a newer navigation with an
                    // older source.
                    navigate_on_source_change.before(WebviewSet::CreateBrowser),
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

/// Query filter for webviews on the mesh input path: entities with a `Mesh3d` or `Mesh2d`,
/// excluding `Sprite` entities, which take the sprite input path even though Bevy renders them
/// through an internal `Mesh2d`.
pub(crate) type MeshWebviewFilter = (Or<(With<Mesh3d>, With<Mesh2d>)>, Without<Sprite>);

/// Queues `AssetEvent::Modified` for `id` without changing the asset, so Bevy rebuilds the
/// asset's GPU-side data (for a material, its bind group). Does nothing when `id` is not loaded.
pub(crate) fn touch_asset<A: Asset>(assets: &mut Assets<A>, id: impl Into<AssetId<A>>) {
    if let Some(asset) = assets.get_mut(id) {
        // NOTE: A bare `assets.get_mut(id)` queues nothing. Bevy's `AssetMut` guard queues the
        // event only when it is mutably dereferenced, which `into_inner` does.
        asset.into_inner();
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

/// The resources a browser is created with, shared by every system that creates browsers.
#[derive(SystemParam)]
pub(crate) struct BrowserCreation<'w, 's> {
    requester: Res<'w, Requester>,
    ipc_event_sender: Res<'w, IpcEventRawSender>,
    brp_sender: Res<'w, BrpSender>,
    cursor_icon_sender: Res<'w, SystemCursorIconSender>,
    drag_regions_sender: Res<'w, DraggableRegionSender>,
    load_handler_sender: Res<'w, LoadHandlerSender>,
    address_changed_sender: Res<'w, AddressChangedSender>,
    title_changed_sender: Res<'w, TitleChangedSender>,
    primary_window: Query<'w, 's, Entity, With<PrimaryWindow>>,
}

/// The webview values a browser is created with.
pub(crate) struct CreateArgs<'a> {
    pub(crate) uri: &'a str,
    pub(crate) size: Vec2,
    pub(crate) dpr: f32,
    pub(crate) preload: &'a [String],
    pub(crate) zoom: f64,
    /// The window to parent the browser to; `None` means the primary window.
    pub(crate) host_window: Option<Entity>,
}

impl BrowserCreation<'_, '_> {
    /// Creates the browser of `webview` in `browsers`, on the context of the profile with cache
    /// path `profile`, or on a fresh in-memory context when `profile` is `None`.
    ///
    /// The host window is resolved to a raw window handle now, so a window closed earlier is
    /// not used.
    pub(crate) fn create(
        &self,
        browsers: &mut Browsers,
        webview: Entity,
        args: CreateArgs,
        profile: Option<&str>,
    ) -> Result<(), CreateBrowserError> {
        browsers.create_browser(
            webview,
            args.uri,
            args.size,
            args.dpr,
            self.requester.clone(),
            self.ipc_event_sender.0.clone(),
            self.brp_sender.clone(),
            self.cursor_icon_sender.clone(),
            self.drag_regions_sender.0.clone(),
            self.load_handler_sender.0.clone(),
            self.address_changed_sender.0.clone(),
            self.title_changed_sender.0.clone(),
            args.preload,
            self.window_handle(args.host_window),
            args.zoom,
            profile,
        )
    }

    fn window_handle(&self, host_window: Option<Entity>) -> Option<RawWindowHandle> {
        WINIT_WINDOWS.with(|winit_windows| {
            let winit_windows = winit_windows.borrow();
            host_window
                .and_then(|window| winit_windows.get_window(window))
                .or_else(|| winit_windows.get_window(self.primary_window.single().ok()?))
                .and_then(|window| {
                    #[allow(deprecated)]
                    window.raw_window_handle().ok()
                })
        })
    }
}

fn any_resized(webviews: Query<Entity, Changed<WebviewSize>>) -> bool {
    !webviews.is_empty()
}

fn added_webview(
    webviews: Query<Entity, (Added<ResolvedWebviewUri>, Without<WebviewProfile>)>,
) -> bool {
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

/// Creates the browser of each newly resolved webview without a [`WebviewProfile`], on its own
/// in-memory context.
fn create_webview(
    mut browsers: NonSendMut<Browsers>,
    creation: BrowserCreation,
    webviews: Query<
        (
            Entity,
            &ResolvedWebviewUri,
            &WebviewSize,
            &WebviewDpr,
            &PreloadScripts,
            Option<&HostWindow>,
            Option<&ZoomLevel>,
        ),
        (Added<ResolvedWebviewUri>, Without<WebviewProfile>),
    >,
) {
    for (webview, uri, size, dpr, preload, host_window, zoom) in webviews.iter() {
        let args = CreateArgs {
            uri: &uri.0,
            size: size.0,
            dpr: dpr.0,
            preload: &preload.0,
            zoom: zoom.map_or(0.0, |zoom| zoom.0),
            host_window: host_window.map(|window| window.0),
        };
        if let Err(error) = creation.create(&mut browsers, webview, args, None) {
            error!("bevy_cef: failed to create the browser of webview {webview}: {error}");
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that a sprite carrying a `Mesh2d` is not matched by the mesh-path webview filter.
    ///
    /// Case: a sprite webview is rendered by Bevy 0.20, which inserts a `Mesh2d` quad on every
    /// `Sprite`.
    #[test]
    fn sprite_with_mesh2d_is_not_a_mesh_webview() {
        let mut world = World::new();
        let sprite = world
            .spawn((Sprite::default(), Mesh2d(Handle::default())))
            .id();
        let mut webviews = world.query_filtered::<Entity, MeshWebviewFilter>();
        assert!(webviews.get(&world, sprite).is_err());
    }

    /// Asserts that `Mesh3d` and plain `Mesh2d` entities match the mesh-path webview filter.
    ///
    /// Case: a user shows a webview on a `Mesh3d` plane or on a hand-built `Mesh2d` quad.
    #[test]
    fn meshes_without_sprite_are_mesh_webviews() {
        let mut world = World::new();
        let mesh3d = world.spawn(Mesh3d(Handle::default())).id();
        let mesh2d = world.spawn(Mesh2d(Handle::default())).id();
        let mut webviews = world.query_filtered::<Entity, MeshWebviewFilter>();
        assert!(webviews.get(&world, mesh3d).is_ok());
        assert!(webviews.get(&world, mesh2d).is_ok());
    }
}
