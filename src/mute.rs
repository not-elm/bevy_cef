use crate::prelude::AudioMuted;
use bevy::prelude::*;

pub(super) struct AudioMutePlugin;

impl Plugin for AudioMutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_audio_mute.run_if(any_changed_audio_mute));
    }
}

fn any_changed_audio_mute(audio_mute: Query<&AudioMuted, Changed<AudioMuted>>) -> bool {
    !audio_mute.is_empty()
}

fn sync_audio_mute(
    browsers: NonSend<bevy_cef_core::prelude::Browsers>,
    audio_mute: Query<(Entity, &AudioMuted), Changed<AudioMuted>>,
) {
    for (entity, mute) in audio_mute.iter() {
        browsers.set_audio_muted(&entity, mute.0);
    }
}
