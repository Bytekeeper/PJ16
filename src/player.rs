use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_enoki::{
    prelude::{OneShot, ParticleEffectHandle},
    ParticleSpawner,
};
use bevy_kira_audio::prelude::*;

use crate::actions::Actions;
use crate::loading::{AudioAssets, EffectAssets, TextureAssets};
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
        LinearDamping(10.0),
        Player::default(),
        StateScoped(GameState::Playing),
    ));
}

fn player_action(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<(Entity, &Transform, &mut Player)>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    effect_assets: Res<EffectAssets>,
    mut commands: Commands,
) {
    for (player_entity, player_transform, mut player) in player_query.iter_mut() {
        let timer = &mut player.action_cooldown;
        timer.tick(time.delta());
        if actions.trigger_action && timer.finished() {
            timer.set_duration(Duration::from_secs(1));
            timer.reset();
            let mut transform = *player_transform;
            transform.translation += Vec3::Z;
            commands.spawn((
                ParticleSpawner::default(),
                ParticleEffectHandle(effect_assets.sword_slash.clone()),
                transform,
                OneShot::Despawn,
            ));
            audio.play(audio_assets.woosh.clone()).with_volume(0.3);

            if let Some(player_direction) = actions.player_direction {
                let player_direction = player_direction.normalize_or_zero() * 200000.0;
                commands
                    .entity(player_entity)
                    .insert(ExternalImpulse::new(player_direction));
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

    for player_transform in player_query.iter() {
        let direction = player_direction.normalize().extend(0.0);
        let as_quat = Quat::from_rotation_arc(Vec3::Y, direction);
        let mut target_transform = player_transform.with_rotation(as_quat);
        target_transform.translation += direction * 20.0;

        if let Some((_, mut arrow_transform)) = arrow_query.iter_mut().next() {
            *arrow_transform = target_transform;
            continue;
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
