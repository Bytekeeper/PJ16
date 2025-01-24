use avian2d::prelude::*;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_enoki::{
    prelude::{OneShot, ParticleEffectHandle},
    ParticleSpawner,
};
use bevy_kira_audio::prelude::*;
use std::collections::VecDeque;

use crate::actions::game_control::{get_movement, GameControl};
use crate::loading::{AudioAssets, EffectAssets};
use crate::player::{Player, PlayerForm};
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
            (
                player_keyboard_input,
                character_actions,
                character_movement,
                hit_detection,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Default, Component)]
pub struct Movement {
    pub move_direction: Option<Vec2>,
}

#[derive(Default, Component)]
pub enum Actions {
    #[default]
    Idle,
    Cooldown(Timer),
    Charging {
        charge: Charge,
        trigger_direction: Option<Vec2>,
    },
    Executing {
        trigger_direction: Vec2,
        pending_cooldown: Timer,
        // Each step will trigger something based on the weapon
        steps: VecDeque<Timer>,
    },
}

#[derive(DerefMut, Deref)]
pub struct Charge(Timer);

impl Default for Charge {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Once))
    }
}

#[derive(Component)]
#[require(RigidBody, LinearDamping(|| LinearDamping(10.0)))]
pub enum MoveMotion {
    Sliding { speed: f32 },
    Bouncing { speed: f32, timer: Timer },
}

#[derive(Component)]
pub struct Health {
    pub owner: u32,
    pub health: u32,
    pub max_health: u32,
}

#[derive(Component)]
pub struct Damage {
    pub target_owner: u32,
}

pub fn player_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touch_input: Res<Touches>,
    mut player: Query<(&mut Actions, &Transform, &Player)>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok((mut actions, player_transform, player)) = player.get_single_mut() else {
        return;
    };

    let mut player_direction = Vec2::new(
        get_movement(GameControl::Right, &keyboard_input)
            - get_movement(GameControl::Left, &keyboard_input),
        get_movement(GameControl::Up, &keyboard_input)
            - get_movement(GameControl::Down, &keyboard_input),
    );

    if let Some(touch_position) = touch_input.first_pressed_position() {
        let (camera, camera_transform) = camera.single();
        if let Ok(touch_position) = camera.viewport_to_world_2d(camera_transform, touch_position) {
            let diff = touch_position - player_transform.translation.xy();
            if diff.length() > FOLLOW_EPSILON {
                player_direction = diff.normalize();
            }
        }
    }

    let player_direction = if player_direction != Vec2::ZERO {
        Some(player_direction.normalize())
    } else {
        None
    };

    let actions = &mut *actions;
    let triggering = keyboard_input.pressed(KeyCode::Space);
    if !triggering {
        match actions {
            Actions::Charging {
                charge,
                trigger_direction,
            } => {
                if let Some(trigger_direction) = player_direction.or(*trigger_direction) {
                    match player.form {
                        PlayerForm::Sword => {
                            let mut steps =
                                VecDeque::from([Timer::from_seconds(0.0, TimerMode::Once)]);
                            for _ in 0..charge.elapsed_secs() as u32 {
                                steps.push_back(Timer::from_seconds(0.2, TimerMode::Once));
                            }
                            *actions = Actions::Executing {
                                trigger_direction,
                                pending_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
                                steps,
                            };
                        }
                    }
                } else {
                    // No direction was selected, nothing will be done but no cool-down will be
                    // applied
                    *actions = Actions::Idle;
                }
            }
            // Unless charging, stopping releasing the trigger will not do anything
            _ => (),
        }
    } else {
        if matches!(actions, Actions::Idle) {
            *actions = Actions::Charging {
                charge: default(),
                trigger_direction: default(),
            };
        }
        match actions {
            Actions::Charging {
                charge,
                trigger_direction,
            } => {
                charge.tick(time.delta());
                *trigger_direction = player_direction.or(*trigger_direction);
            }
            // No other Action state allows charging currently
            _ => (),
        }
    }
}

fn character_movement(
    time: Res<Time>,
    mut character_query: Query<(Entity, &Movement, &mut MoveMotion)>,
    mut commands: Commands,
) {
    for (character_entity, movement, mut move_motion) in character_query.iter_mut() {
        if let Some(move_direction) = movement.move_direction {
            match *move_motion {
                MoveMotion::Sliding { speed } => {
                    commands
                        .entity(character_entity)
                        .insert(ExternalImpulse::new(
                            (move_direction * time.delta_secs()).clamp_length(0.0, speed) * 200.0,
                        ));
                }
                MoveMotion::Bouncing {
                    speed,
                    ref mut timer,
                } => {
                    timer.tick(time.delta());
                    if timer.just_finished() {
                        commands
                            .entity(character_entity)
                            .insert(ExternalImpulse::new(
                                (move_direction * time.delta_secs()).clamp_length(0.0, speed)
                                    * 2000.0,
                            ));
                    }
                }
            }
        }
    }
}

fn character_actions(
    time: Res<Time>,
    mut character_query: Query<(Entity, &Transform, &mut Actions, &Health)>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    effect_assets: Res<EffectAssets>,
    mut commands: Commands,
) {
    for (character_entity, character_transform, mut actions, Health { owner, .. }) in
        character_query.iter_mut()
    {
        let actions = &mut *actions;
        match actions {
            Actions::Cooldown(timer) => {
                timer.tick(time.delta());
                if timer.finished() {
                    *actions = Actions::Idle;
                }
            }
            Actions::Executing {
                trigger_direction,
                pending_cooldown,
                steps,
            } => {
                if let Some(item) = steps.front_mut() {
                    // TODO the time is 'cut off' here, some portion should be subtracted from the next
                    // timer
                    item.tick(time.delta());
                    if item.finished() {
                        steps.pop_front();

                        let mut transform = *character_transform;
                        transform.translation += Vec3::Z;
                        commands.spawn((
                            ParticleSpawner::default(),
                            ParticleEffectHandle(effect_assets.sword_slash.clone()),
                            transform,
                            OneShot::Despawn,
                            Collider::circle(15.0),
                            Damage {
                                target_owner: 1 - *owner,
                            },
                            Sensor,
                        ));
                        audio.play(audio_assets.woosh.clone()).with_volume(0.3);

                        let character_direction = trigger_direction.normalize_or_zero() * 20000.0;
                        commands
                            .entity(character_entity)
                            .insert(ExternalImpulse::new(character_direction));
                    }
                } else {
                    *actions = Actions::Cooldown(pending_cooldown.clone());
                };
            }
            _ => (),
        }
    }
}

fn hit_detection(
    mut collision_event_reader: EventReader<Collision>,
    health_query: Query<&Health>,
    damage_query: Query<&Damage>,
    mut commands: Commands,
) {
    for Collision(contacts) in collision_event_reader.read() {
        let Contacts {
            mut entity1,
            mut entity2,
            ..
        } = contacts;
        if health_query.contains(entity1) {
            std::mem::swap(&mut entity1, &mut entity2);
        }
        if let (Ok(Damage { target_owner }), Ok(Health { owner, .. })) =
            (damage_query.get(entity1), health_query.get(entity2))
        {
            if target_owner == owner {
                commands.entity(entity2).despawn();
            }
        }
    }
}
