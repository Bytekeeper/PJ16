use crate::actions::{Actions, Effect, Health, MoveMotion, Movement, Step};
use crate::loading::TextureAssets;
use crate::player::Player;
use crate::GameState;

use avian2d::prelude::{Collider, LockedAxes};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_rand::prelude::{GlobalEntropy, WyRand};
use rand::Rng;

pub struct EnemiesPlugin;

#[derive(Component, Default)]
pub struct Ai {
    pub form: EnemyForm,
}

#[derive(Default)]
pub enum EnemyForm {
    #[default]
    Melee,
}

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (enemy_spawner, ai_think)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            PostUpdate,
            update_sprite.run_if(in_state(GameState::Playing)),
        );
    }
}

fn enemy_spawner(
    mut commands: Commands,
    mut rng: GlobalEntropy<WyRand>,
    mut enemy_1_counter: Local<(f32, f32)>,
    textures: Res<TextureAssets>,
    time: Res<Time>,
) {
    enemy_1_counter.0 += rng.gen::<f32>() * time.delta_secs();

    if enemy_1_counter.0 > enemy_1_counter.1 {
        enemy_1_counter.1 += 0.1;
        enemy_1_counter.0 -= enemy_1_counter.0;

        commands.spawn((
            AseSpriteAnimation {
                aseprite: textures.enemy_1_left.clone(),
                animation: Animation::default(),
            },
            Transform::from_translation(vec3(
                rng.gen_range(-200.0..200.0),
                rng.gen_range(-200.0..200.0),
                5.0,
            )),
            Ai::default(),
            Collider::circle(5.0),
            Health {
                owner: 1,
                max_health: 1,
                health: 1,
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
}

fn ai_think(
    mut ai_query: Query<(&mut Movement, &mut Actions, &Transform, &Ai)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        debug!("No player found");
        return;
    };
    for (mut movement, mut actions, ai_transform, ai) in ai_query.iter_mut() {
        let actions = &mut *actions;
        let delta = (player_transform.translation - ai_transform.translation).truncate();
        let range = match ai.form {
            EnemyForm::Melee => 15.0,
        };
        if delta.length() > range {
            movement.move_direction = Some(delta.clamp_length_min(10.0));
        } else {
            movement.move_direction = None;
            if matches!(actions, Actions::Idle) {
                // TODO This is one of the points where we decide on how the monster attacks
                *actions = Actions::Executing {
                    trigger_direction: delta,
                    pending_cooldown: Timer::from_seconds(0.5, TimerMode::Once),
                    steps: [Step::from_timer(Timer::from_seconds(1.2, TimerMode::Once))
                        .with_effect(Effect::Splash)]
                    .into(),
                };
            }
            // else we're either attacking or on cool-down: Just wait
        }
    }
}

/// Update the enemy sprite animation based on the action it performs
fn update_sprite(
    mut animation_query: Query<
        (
            &mut AseSpriteAnimation,
            &mut AnimationState,
            &Actions,
            &Movement,
        ),
        With<Ai>,
    >,
    textures: Res<TextureAssets>,
) {
    for (mut animation, mut animation_state, actions, movement) in animation_query.iter_mut() {
        let (anim_handle, anim_name) = match actions {
            Actions::Idle => (
                if movement.move_direction.is_some_and(|dir| dir.x <= 0.0) {
                    &textures.enemy_1_left
                } else {
                    &textures.enemy_1_right
                },
                Animation::default(),
            ),
            Actions::Executing {
                trigger_direction, ..
            } => (
                if trigger_direction.x <= 0.0 {
                    &textures.enemy_1_attack_left
                } else {
                    &textures.enemy_1_attack_right
                },
                Animation::default(),
            ),
            Actions::ChangePlayerForm(_) | Actions::Cooldown(_) | Actions::Charging { .. } => {
                continue
            }
        };
        if animation.aseprite != *anim_handle {
            animation.aseprite = anim_handle.clone();
            *animation_state = AnimationState::default();
        }
        if animation.animation.tag != anim_name.tag {
            animation.animation = anim_name;
            *animation_state = AnimationState::default();
        }
    }
}
