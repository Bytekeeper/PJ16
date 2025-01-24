use crate::actions::{Actions, Health, MoveMotion, Movement};
use crate::animation::*;
use crate::loading::Animations;
use crate::player::Player;
use crate::GameState;

use avian2d::prelude::{Collider, LockedAxes};
use bevy::math::vec3;
use bevy::prelude::*;

pub struct EnemiesPlugin;

#[derive(Component)]
pub struct Ai;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_enemies)
            .add_systems(PreUpdate, ai_think.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_enemies(mut commands: Commands, animations: Res<Animations>) {
    commands.spawn((
        Sprite::from_atlas_image(
            animations.enemy_1_walk.image.clone(),
            animations.enemy_1_walk.atlas.clone(),
        ),
        Transform::from_translation(vec3(100.0, 100.0, 5.0)),
        animations.enemy_1_walk.indices,
        Ai,
        Collider::circle(5.0),
        Health {
            owner: 1,
            max_health: 3,
            health: 3,
        },
        LockedAxes::ROTATION_LOCKED,
        MoveMotion::Sliding { speed: 10.0 },
        Actions::default(),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        StateScoped(GameState::Playing),
    ));
    commands.spawn((
        Sprite::from_atlas_image(
            animations.enemy_1_walk.image.clone(),
            animations.enemy_1_walk.atlas.clone(),
        ),
        Transform::from_translation(vec3(-100.0, 100.0, 5.0)),
        animations.enemy_1_walk.indices,
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
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        StateScoped(GameState::Playing),
    ));
}

fn ai_think(
    mut ai_query: Query<(&mut Movement, &Transform), With<Ai>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        warn!("No player found");
        return;
    };
    for (mut movement, ai_transform) in ai_query.iter_mut() {
        movement.move_direction = Some(
            (player_transform.translation - ai_transform.translation)
                .truncate()
                .clamp_length_min(100.0),
        );
    }
}
