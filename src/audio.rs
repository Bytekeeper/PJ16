use crate::actions::set_movement_actions;
use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin);
    }
}

fn control_flying_sound(
    //actions: Res<Actions>,
    //audio: Res<FlyingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    //if let Some(instance) = audio_instances.get_mut(&audio.0) {
    //    match instance.state() {
    //        PlaybackState::Paused { .. } => {
    //            //if actions.player_movement.is_some() {
    //            //    instance.resume(AudioTween::default());
    //            //}
    //        }
    //        PlaybackState::Playing { .. } => {
    //            //if actions.player_movement.is_none() {
    //            //    instance.pause(AudioTween::default());
    //            //}
    //        }
    //        _ => {}
    //    }
    //}
}
