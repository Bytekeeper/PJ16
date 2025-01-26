use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enoki::{
    prelude::{OneShot, ParticleEffectHandle},
    ParticleSpawner,
};
use bevy_kira_audio::prelude::*;
use std::collections::VecDeque;

use crate::loading::{AudioAssets, EffectAssets};
use crate::GameState;
use game_control::{InputPlugin, InputSet};

mod game_control;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions.
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (character_actions, character_movement, hit_detection)
                .chain()
                .after(InputSet)
                .run_if(in_state(GameState::Playing)),
        )
        .add_plugins(InputPlugin);
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
        steps: VecDeque<Step>,
    },
}

pub struct Step {
    pub timer: Timer,
    pub effect: Effect,
}

pub enum Effect {
    None,
    Circle,
    Splash,
}

impl Step {
    pub fn from_timer(timer: Timer) -> Self {
        Self {
            timer,
            effect: Effect::None,
        }
    }

    pub fn with_effect(self, effect: Effect) -> Self {
        Self { effect, ..self }
    }
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

fn character_movement(
    time: Res<Time>,
    mut character_query: Query<(Entity, &Movement, &mut MoveMotion)>,
    mut commands: Commands,
) {
    for (character_entity, movement, mut move_motion) in character_query.iter_mut() {
        if let Some(move_direction) = movement.move_direction {
            match *move_motion {
                MoveMotion::Sliding { speed } => {
                    commands.entity(character_entity).insert(
                        ExternalForce::new(move_direction.clamp_length(0.0, speed) * 800.0)
                            .with_persistence(false),
                    );
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
                                move_direction.clamp_length(0.0, speed) * 500.0,
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
                    item.timer.tick(time.delta());
                    if item.timer.finished() {
                        let item = steps.pop_front().expect("No steps found");

                        let mut transform = *character_transform;
                        transform.translation += Vec3::Z;
                        transform.rotation =
                            Quat::from_rotation_arc_2d(Vec2::Y, *trigger_direction);
                        let mut ec = commands.spawn((
                            transform,
                            OneShot::Despawn,
                            Collider::circle(15.0),
                            Damage {
                                target_owner: 1 - *owner,
                            },
                            Sensor,
                        ));
                        match item.effect {
                            Effect::Circle => {
                                ec.insert((
                                    ParticleSpawner::default(),
                                    ParticleEffectHandle(effect_assets.sword_slash.clone()),
                                ));
                            }
                            Effect::Splash => {
                                ec.insert((
                                    ParticleSpawner::default(),
                                    ParticleEffectHandle(effect_assets.enemy_1_attack.clone()),
                                ));
                            }
                            Effect::None => (),
                        }

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
    mut health_query: Query<(&Transform, &mut Health)>,
    damage_query: Query<(&Transform, &Damage)>,
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
        if let (
            Ok((damage_source_transform, Damage { target_owner })),
            Ok((target_transform, mut health)),
        ) = (damage_query.get(entity1), health_query.get_mut(entity2))
        {
            let Health { owner, health, .. } = &mut *health;
            if target_owner == owner {
                *health -= 1;
                let delta = target_transform.translation - damage_source_transform.translation;
                let delta = delta.truncate().normalize_or_zero();
                let mut ec = commands.entity(entity2);
                ec.insert(ExternalImpulse::new(delta * 1000.0));
                if *health == 0 {
                    ec.despawn();
                }
                commands.entity(entity1).remove::<Damage>();
            }
        }
    }
}
