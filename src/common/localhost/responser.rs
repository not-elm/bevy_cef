use crate::common::localhost::asset_loader::CefResponseHandle;
use crate::common::{ResolvedWebviewUri, WebviewSource};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_cef_core::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

static INLINE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Prefix for inline HTML URIs within the `cef://localhost/` scheme.
const INLINE_PREFIX: &str = "__inline__/";

/// Cleanup marker that stays on the entity. Removed on despawn to clean up the store.
#[derive(Component)]
pub(crate) struct InlineHtmlId(pub(crate) String);

/// In-memory store for inline HTML content.
#[derive(Resource, Default)]
pub(crate) struct InlineHtmlStore {
    by_id: HashMap<String, Vec<u8>>,
}

impl InlineHtmlStore {
    pub(crate) fn remove(&mut self, id: &str) {
        self.by_id.remove(id);
    }
}

pub struct ResponserPlugin;

impl Plugin for ResponserPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();
        app.insert_resource(Requester(tx))
            .insert_resource(RequesterReceiver(rx))
            .init_resource::<InlineHtmlStore>()
            .add_systems(PreUpdate, resolve_webview_source)
            .add_systems(
                Update,
                (
                    coming_request,
                    responser,
                    hot_reload.run_if(any_changed_assets),
                ),
            );
    }
}

fn any_changed_assets(mut er: MessageReader<AssetEvent<CefResponse>>) -> bool {
    er.read()
        .any(|event| matches!(event, AssetEvent::Modified { .. }))
}

fn resolve_webview_source(
    mut commands: Commands,
    mut store: ResMut<InlineHtmlStore>,
    query: Query<(Entity, &WebviewSource, Option<&InlineHtmlId>), Changed<WebviewSource>>,
) {
    for (entity, source, existing_id) in query.iter() {
        // Clean up old inline entry if switching away or updating
        if let Some(old_id) = existing_id {
            store.by_id.remove(&old_id.0);
        }

        match source {
            WebviewSource::Url(url) => {
                let mut entity_commands = commands.entity(entity);
                entity_commands.insert(ResolvedWebviewUri(url.clone()));
                if existing_id.is_some() {
                    entity_commands.remove::<InlineHtmlId>();
                }
            }
            WebviewSource::InlineHtml(html) => {
                let id = INLINE_ID_COUNTER
                    .fetch_add(1, Ordering::Relaxed)
                    .to_string();
                store.by_id.insert(id.clone(), html.as_bytes().to_vec());

                let url = format!("{SCHEME_CEF}://{HOST_CEF}/{INLINE_PREFIX}{id}");
                commands
                    .entity(entity)
                    .insert((ResolvedWebviewUri(url), InlineHtmlId(id)));
            }
        }
    }
}

fn coming_request(
    mut commands: Commands,
    requester_receiver: Res<RequesterReceiver>,
    asset_server: Res<AssetServer>,
    store: Res<InlineHtmlStore>,
) {
    while let Ok(request) = requester_receiver.0.try_recv() {
        if let Some(id) = extract_inline_id(&request.uri) {
            let response = match store.by_id.get(id) {
                Some(data) => CefResponse {
                    mime_type: "text/html".to_string(),
                    status_code: 200,
                    data: data.clone(),
                },
                None => CefResponse {
                    mime_type: "text/plain".to_string(),
                    status_code: 404,
                    data: b"Not Found".to_vec(),
                },
            };
            let _ = request.responser.0.send_blocking(response);
        } else {
            commands.spawn((
                CefResponseHandle(asset_server.load(request.uri)),
                request.responser,
            ));
        }
    }
}

/// Extracts the inline ID from a URI like `__inline__/123` or `__inline__/123?query#fragment`.
fn extract_inline_id(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix(INLINE_PREFIX)?;
    // Strip query string and fragment
    let id = rest.split(['?', '#']).next().unwrap_or(rest);
    Some(id)
}

fn responser(
    mut commands: Commands,
    mut handle_stores: Local<HashSet<Handle<CefResponse>>>,
    responses: Res<Assets<CefResponse>>,
    handles: Query<(Entity, &CefResponseHandle, &Responser)>,
) {
    for (entity, handle, responser) in handles.iter() {
        if let Some(response) = responses.get(&handle.0) {
            let _ = responser.0.send_blocking(response.clone());
            commands.entity(entity).despawn();
            handle_stores.insert(handle.0.clone());
        }
    }
}

fn hot_reload(browsers: NonSend<Browsers>) {
    browsers.reload();
}
