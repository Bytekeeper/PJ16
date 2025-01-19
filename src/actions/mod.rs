use avian2d::prelude::{ExternalImpulse, LinearDamping, RigidBody};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_enoki::{
    prelude::{OneShot, ParticleEffectHandle},
    ParticleSpawner,
};
use bevy_kira_audio::prelude::*;

use crate::actions::game_control::{get_movement, GameControl};
use crate::loading::{AudioAssets, EffectAssets};
use crate::player::Player;
use crate::GameState;

mod game_control;

pub const FOLLOW_EPSILON: f32 = 5.;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions.
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (set_movement_actions, character_triggers, character_movement)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Default, Component)]
pub struct Actions {
    pub move_direction: Option<Vec2>,
    pub trigger_direction: Option<Vec2>,
    pub trigger_action: bool,
    pub trigger_cooldown: Timer,
}

#[derive(Component)]
#[require(RigidBody, LinearDamping(|| LinearDamping(10.0)))]
pub enum MoveMotion {
    Sliding { speed: f32 },
}

pub fn set_movement_actions(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touch_input: Res<Touches>,
    mut player: Query<(&mut Actions, &Transform), With<Player>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let Ok((mut actions, player_transform)) = player.get_single_mut() else {
        return;
    };
    let mut player_direction = Vec2::new(
        get_movement(GameControl::Right, &keyboard_input)
            - get_movement(GameControl::Left, &keyboard_input),
        get_movement(GameControl::Up, &keyboard_input)
            - get_movement(GameControl::Down, &keyboard_input),
    );

    actions.trigger_action = keyboard_input.pressed(KeyCode::Space);

    if let Some(touch_position) = touch_input.first_pressed_position() {
        let (camera, camera_transform) = camera.single();
        if let Ok(touch_position) = camera.viewport_to_world_2d(camera_transform, touch_position) {
            let diff = touch_position - player_transform.translation.xy();
            if diff.length() > FOLLOW_EPSILON {
                player_direction = diff.normalize();
            }
        }
    }

    if player_direction != Vec2::ZERO {
        actions.trigger_direction = Some(player_direction.normalize());
    } else {
        actions.trigger_direction = None;
    }
}

fn character_movement(
    time: Res<Time>,
    mut character_query: Query<(Entity, &Actions, &MoveMotion)>,
    mut commands: Commands,
) {
    for (character_entity, actions, move_motion) in character_query.iter_mut() {
        if let Some(move_direction) = actions.move_direction {
            match move_motion {
                MoveMotion::Sliding { speed } => {
                    commands
                        .entity(character_entity)
                        .insert(ExternalImpulse::new(
                            (move_direction * time.delta_secs()).clamp_length(0.0, *speed) * 200.0,
                        ));
                }
            }
        }
    }
}

fn character_triggers(
    time: Res<Time>,
    mut character_query: Query<(Entity, &Transform, &mut Actions)>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    effect_assets: Res<EffectAssets>,
    mut commands: Commands,
) {
    for (character_entity, character_transform, mut actions) in character_query.iter_mut() {
        let actions = &mut *actions;
        let timer = &mut actions.trigger_cooldown;
        timer.tick(time.delta());
        if actions.trigger_action && timer.finished() {
            timer.set_duration(Duration::from_secs(1));
            timer.reset();
            let mut transform = *character_transform;
            transform.translation += Vec3::Z;
            commands.spawn((
                ParticleSpawner::default(),
                ParticleEffectHandle(effect_assets.sword_slash.clone()),
                transform,
                OneShot::Despawn,
            ));
            audio.play(audio_assets.woosh.clone()).with_volume(0.3);

            if let Some(character_direction) = actions.trigger_direction {
                let character_direction = character_direction.normalize_or_zero() * 20000.0;
                commands
                    .entity(character_entity)
                    .insert(ExternalImpulse::new(character_direction));
            }
        }
    }
}
