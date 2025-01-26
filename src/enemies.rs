use crate::actions::{Actions, Health, MoveMotion, Movement};
use crate::loading::TextureAssets;
use crate::player::Player;
use crate::GameState;

use avian2d::prelude::{Collider, LockedAxes};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

pub struct EnemiesPlugin;

#[derive(Component)]
pub struct Ai;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_enemies)
            .add_systems(PreUpdate, ai_think.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_enemies(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        AseSpriteAnimation {
            aseprite: textures.enemy_1.clone(),
            animation: Animation::tag("walk"),
        },
        Transform::from_translation(vec3(100.0, 100.0, 5.0)),
        Ai,
        Collider::circle(5.0),
        Health {
            owner: 1,
            max_health: 3,
            health: 3,
        },
        LockedAxes::ROTATION_LOCKED,
        MoveMotion::Sliding { speed: 10.0 },
        Movement::default(),
        Actions::default(),
        StateScoped(GameState::Playing),
    ));
    commands.spawn((
        AseSpriteAnimation {
            aseprite: textures.enemy_1.clone(),
            animation: Animation::tag("walk"),
        },
        Transform::from_translation(vec3(-100.0, 100.0, 5.0)),
        Ai,
        Collider::circle(5.0),
        Health {
            owner: 1,
            max_health: 3,
            health: 3,
        },
        LockedAxes::ROTATION_LOCKED,
        MoveMotion::Bouncing {
            speed: 10.0,
            timer: Timer::from_seconds(0.6, TimerMode::Repeating),
        },
        Actions::default(),
        Movement::default(),
        StateScoped(GameState::Playing),
    ));
}

fn ai_think(
    mut ai_query: Query<(&mut Movement, &mut Actions, &Transform), With<Ai>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        debug!("No player found");
        return;
    };
    for (mut movement, mut actions, ai_transform) in ai_query.iter_mut() {
        let actions = &mut *actions;
        let delta = (player_transform.translation - ai_transform.translation).truncate();
        if delta.length() > 10.0 {
            movement.move_direction = Some(delta.clamp_length_min(10.0));
        } else {
            movement.move_direction = None;
            match actions {
                Actions::Idle => {
                    // TODO This is one of the points where we decide on how the monster attacks
                    *actions = Actions::Executing {
                        trigger_direction: delta,
                        pending_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
                        steps: [Timer::from_seconds(0.0, TimerMode::Once)].into(),
                    };
                }
                // When not Idle, were either attacking or on cool-down: Just wait
                _ => (),
            }
        }
    }
}
