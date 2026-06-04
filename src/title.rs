//! Page title change notifications for webviews.
//!
//! When a webview's page title changes, CEF's `DisplayHandler::on_title_change`
//! sends a [`TitleChangedMessage`] across a channel. [`drain_title_changed`]
//! receives it on the Bevy side, updates the [`WebviewTitle`] component, and
//! fires a [`TitleChanged`] entity event — but only when the title actually
//! differs from the current value (dedup).

use async_channel::Receiver;
use bevy::ecs::event::EntityTrigger;
use bevy::prelude::*;
use bevy_cef_core::prelude::{TitleChangedMessage, TitleChangedSenderInner};
use serde::{Deserialize, Serialize};

pub(super) struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = async_channel::unbounded();
        app.insert_resource(TitleChangedSender(tx))
            .insert_resource(TitleChangedReceiver(rx))
            .register_type::<TitleChanged>()
            .register_type::<WebviewTitle>()
            .add_systems(PreUpdate, drain_title_changed);
    }
}

/// Fired when a webview's page title changes.
#[derive(Debug, EntityEvent, Clone, Reflect, Serialize, Deserialize)]
pub struct TitleChanged {
    #[event_target]
    pub webview: Entity,
    pub title: String,
}

/// Holds the current page title of a webview. Updated automatically when the
/// title changes. Absent until the first title is reported.
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct WebviewTitle(pub String);

#[derive(Resource, Debug, Deref)]
pub(crate) struct TitleChangedSender(pub(crate) TitleChangedSenderInner);

#[derive(Resource, Debug)]
struct TitleChangedReceiver(Receiver<TitleChangedMessage>);

fn drain_title_changed(
    mut commands: Commands,
    receiver: Res<TitleChangedReceiver>,
    titles: Query<&WebviewTitle>,
) {
    // 同一フレーム内で同じ entity に複数メッセージが来ても正しく dedup するため、
    // この drain 中に適用した最新タイトルを覚えておく(Commands は遅延適用のため、
    // titles クエリはループ内の insert を反映しない)。
    let mut applied: std::collections::HashMap<Entity, String> = Default::default();
    while let Ok(msg) = receiver.0.try_recv() {
        let current = applied
            .get(&msg.webview)
            .map(String::as_str)
            .or_else(|| titles.get(msg.webview).ok().map(|t| t.0.as_str()));
        // dedup: 現在値と同じタイトルなら component 更新も event 発火もしない。
        if current == Some(msg.title.as_str()) {
            continue;
        }
        applied.insert(msg.webview, msg.title.clone());
        // webview が既に despawn されている場合に panic しないよう存在確認する。
        if let Ok(mut entity) = commands.get_entity(msg.webview) {
            entity.insert(WebviewTitle(msg.title.clone()));
        }
        commands.trigger_with(
            TitleChanged {
                webview: msg.webview,
                title: msg.title,
            },
            EntityTrigger,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_cef_core::prelude::TitleChangedMessage;

    #[derive(Resource, Default)]
    struct FiredTitles(Vec<String>);

    fn record_title(on: On<TitleChanged>, mut fired: ResMut<FiredTitles>) {
        fired.0.push(on.title.clone());
    }

    /// World + Schedule を直接使う(App/プラグイン非依存)。Schedule::run は
    /// 各システム後に Commands を flush し、その際 trigger 経由の observer が走る。
    fn setup() -> (World, Schedule, async_channel::Sender<TitleChangedMessage>) {
        let (tx, rx) = async_channel::unbounded::<TitleChangedMessage>();
        let mut world = World::new();
        world.insert_resource(TitleChangedReceiver(rx));
        world.init_resource::<FiredTitles>();
        world.add_observer(record_title);
        let mut schedule = Schedule::default();
        schedule.add_systems(drain_title_changed);
        (world, schedule, tx)
    }

    #[test]
    fn updates_component_and_fires_event() {
        let (mut world, mut schedule, tx) = setup();
        let e = world.spawn_empty().id();
        tx.send_blocking(TitleChangedMessage {
            webview: e,
            title: "Hello".into(),
        })
        .unwrap();
        schedule.run(&mut world);
        assert_eq!(
            world.get::<WebviewTitle>(e).map(|t| t.0.as_str()),
            Some("Hello")
        );
        assert_eq!(world.resource::<FiredTitles>().0, vec!["Hello".to_string()]);
    }

    #[test]
    fn dedups_identical_titles_in_same_frame() {
        let (mut world, mut schedule, tx) = setup();
        let e = world.spawn_empty().id();
        for _ in 0..2 {
            tx.send_blocking(TitleChangedMessage {
                webview: e,
                title: "A".into(),
            })
            .unwrap();
        }
        schedule.run(&mut world);
        assert_eq!(world.resource::<FiredTitles>().0, vec!["A".to_string()]);
    }

    #[test]
    fn fires_for_distinct_titles() {
        let (mut world, mut schedule, tx) = setup();
        let e = world.spawn_empty().id();
        for t in ["A", "B"] {
            tx.send_blocking(TitleChangedMessage {
                webview: e,
                title: t.into(),
            })
            .unwrap();
        }
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<FiredTitles>().0,
            vec!["A".to_string(), "B".to_string()]
        );
        assert_eq!(
            world.get::<WebviewTitle>(e).map(|t| t.0.clone()),
            Some("B".to_string())
        );
    }

    #[test]
    fn dedups_identical_titles_across_frames() {
        let (mut world, mut schedule, tx) = setup();
        let e = world.spawn_empty().id();
        tx.send_blocking(TitleChangedMessage {
            webview: e,
            title: "A".into(),
        })
        .unwrap();
        schedule.run(&mut world);
        tx.send_blocking(TitleChangedMessage {
            webview: e,
            title: "A".into(),
        })
        .unwrap();
        schedule.run(&mut world);
        assert_eq!(world.resource::<FiredTitles>().0, vec!["A".to_string()]);
    }
}
