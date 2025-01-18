use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_enoki::ParticleSpawner;
use bevy_kira_audio::prelude::*;

use crate::actions::Actions;
use crate::loading::{AudioAssets, TextureAssets};
use crate::GameState;

pub struct PlayerPlugin;

#[derive(Component, Default)]
pub struct Player {
    action_cooldown: Timer,
}

#[derive(Component)]
pub struct DirectionArrow;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (update_direction_arrow, player_action).run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        Sprite::from_image(textures.player_sword.clone()),
        Transform::from_translation(Vec3::new(0., 0., 2.)),
        RigidBody::Dynamic,
        Collider::circle(16.0),
        Player::default(),
        StateScoped(GameState::Playing),
    ));
}

fn player_action(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<(&Transform, &mut Player)>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    mut commands: Commands,
) {
    for (player_transform, mut player) in player_query.iter_mut() {
        let timer = &mut player.action_cooldown;
        timer.tick(time.delta());
        if actions.trigger_action {
            if timer.finished() {
                timer.set_duration(Duration::from_secs(1));
                timer.reset();
                let mut transform = player_transform.clone();
                transform.translation += Vec3::Z;
                commands.spawn((
                    // the main component.
                    // holds a material handle.
                    // defaults to a simple white color quad.
                    // has required components
                    ParticleSpawner::default(),
                    transform,
                ));
                audio.play(audio_assets.woosh.clone()).with_volume(0.3);
            }
        }
    }
}

fn update_direction_arrow(
    actions: Res<Actions>,
    textures: Res<TextureAssets>,
    mut arrow_query: Query<(Entity, &mut Transform), (With<DirectionArrow>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    let Some(player_direction) = actions.player_direction else {
        // No direction selected: remove arrow display
        for (entity, _) in arrow_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    'outer: for player_transform in player_query.iter() {
        let direction = player_direction.normalize().extend(0.0);
        let as_quat = Quat::from_rotation_arc(Vec3::Y, direction);
        let mut target_transform = player_transform.clone().with_rotation(as_quat);
        target_transform.translation += direction * 20.0;

        for (_, mut arrow_transform) in arrow_query.iter_mut() {
            *arrow_transform = target_transform;
            continue 'outer;
        }
        // No existing arrow
        commands.spawn((
            Sprite::from_image(textures.arrow.clone()),
            target_transform,
            DirectionArrow,
            StateScoped(GameState::Playing),
        ));
    }
}
