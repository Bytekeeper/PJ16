use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_enoki::{
    prelude::{OneShot, ParticleEffectHandle},
    ParticleSpawner,
};
use bevy_kira_audio::prelude::*;
use std::collections::VecDeque;

use crate::enemies::{Ai, EnemyForm};
use crate::loading::{AudioAssets, EffectAssets, RangedEnemyAssets, TextureAssets};
use crate::physics::CollisionLayer;
use crate::player::{Player, PlayerForm};
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
            (
                character_actions,
                character_movement,
                despawn_dead,
                hit_detection,
            )
                .chain()
                .after(InputSet)
                .run_if(in_state(GameState::Playing)),
        )
        .add_plugins(InputPlugin);
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Dying(pub Timer);

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
    ChangePlayerForm(PlayerForm),
}

pub struct Step {
    pub timer: Timer,
    pub effect: Effect,
    pub sfx: Option<Handle<AudioSource>>,
    pub forward: f32,
}

pub enum Effect {
    None,
    Circle,
    Splash,
    Spawn(Spawn),
}

pub enum Spawn {
    Arrow,
}

impl Step {
    pub fn from_timer(timer: Timer) -> Self {
        Self {
            timer,
            forward: 0.0,
            effect: Effect::None,
            sfx: default(),
        }
    }

    pub fn with_effect(self, effect: Effect) -> Self {
        Self { effect, ..self }
    }

    pub fn with_forward(self, forward: f32) -> Self {
        Self { forward, ..self }
    }

    pub fn with_sfx(self, sfx: Handle<AudioSource>) -> Self {
        Self {
            sfx: Some(sfx),
            ..self
        }
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
    pub source_owner: u32,
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
    mut player_query: Query<&mut Player>,
    audio: Res<Audio>,
    textures: Res<TextureAssets>,
    ranged_enemy_assets: Res<RangedEnemyAssets>,
    effect_assets: Res<EffectAssets>,
    mut commands: Commands,
) {
    for (character_entity, character_transform, mut actions, Health { owner, .. }) in
        character_query.iter_mut()
    {
        let actions = &mut *actions;
        match actions {
            Actions::ChangePlayerForm(next_player_form) => {
                let mut ec = commands.entity(character_entity);
                match next_player_form {
                    PlayerForm::Sword => {
                        ec.insert(AseSpriteAnimation {
                            aseprite: textures.player_sword.clone(),
                            animation: Animation::tag("flaming"),
                        });
                    }
                    PlayerForm::Bow => {
                        ec.insert(AseSpriteAnimation {
                            aseprite: textures.player_bow.clone(),
                            ..default()
                        });
                    }
                }
                player_query.single_mut().form = *next_player_form;
                *actions = Actions::Idle;
            }
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
                        transform.rotation = Quat::from_rotation_z(trigger_direction.to_angle());
                        let mut ec = commands.spawn((
                            transform,
                            OneShot::Despawn,
                            Collider::circle(15.0),
                            Damage {
                                source_owner: *owner,
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
                            Effect::Spawn(Spawn::Arrow) => {
                                commands.spawn((
                                    AseSpriteAnimation {
                                        aseprite: ranged_enemy_assets.projectile.clone(),
                                        animation: default(),
                                    },
                                    transform,
                                    LinearVelocity(
                                        trigger_direction.clamp_length(100000.0, 100000.0),
                                    ),
                                    Collider::circle(2.0),
                                    Dying(Timer::from_seconds(0.7, TimerMode::Once)),
                                    CollisionLayers::new(
                                        CollisionLayer::EnemyProjectile,
                                        LayerMask::NONE,
                                        //[CollisionLayer::Default, CollisionLayer::Player],
                                    ),
                                    StateScoped(GameState::Playing),
                                ));
                            }
                            Effect::None => (),
                        }
                        if let Some(sfx) = item.sfx {
                            audio.play(sfx.clone()).with_volume(0.2);
                        }

                        if item.forward != 0.0 {
                            let character_direction =
                                trigger_direction.normalize_or_zero() * item.forward * 1000.0;
                            commands
                                .entity(character_entity)
                                .insert(ExternalImpulse::new(character_direction));
                        }
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
    mut player_query: Query<&mut Player>,
    ai_query: Query<&Ai>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
    mut commands: Commands,
    textures: Res<TextureAssets>,
    ranged_enemy_assets: Res<RangedEnemyAssets>,
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
            Ok((
                damage_source_transform,
                Damage {
                    target_owner,
                    source_owner,
                },
            )),
            Ok((target_transform, mut health)),
        ) = (damage_query.get(entity1), health_query.get_mut(entity2))
        {
            if *target_owner == health.owner && health.health > 0 {
                health.health -= 1;
                let delta = target_transform.translation - damage_source_transform.translation;
                let delta = delta.truncate().normalize_or_zero();
                let mut ec = commands.entity(entity2);
                ec.insert(ExternalImpulse::new(delta * 1000.0));
                if health.health == 0 {
                    if *source_owner == 0 {
                        player_query.single_mut().score += 1;
                    }
                    if *target_owner == 0 {
                        audio
                            .play(audio_assets.player_damaged_effected.clone())
                            .with_volume(0.3);
                    }
                    ec.insert(Dying(Timer::from_seconds(1.0, TimerMode::Once)));
                    ec.remove::<(Ai, Health)>();

                    if let Ok(ai) = ai_query.get(entity2) {
                        let animation = Animation::default().with_repeat(0.into());
                        match ai.form {
                            EnemyForm::Melee => {
                                ec.insert(AseSpriteAnimation {
                                    aseprite: textures.enemy_1_death.clone(),
                                    animation,
                                });
                            }
                            EnemyForm::Ranged => {
                                ec.insert(AseSpriteAnimation {
                                    aseprite: ranged_enemy_assets.death.clone(),
                                    animation,
                                });
                            }
                        }
                        ec.insert(AnimationState::default());
                    }
                }
                commands.entity(entity1).remove::<Damage>();
            }
        }
    }
}

fn despawn_dead(
    mut commands: Commands,
    mut dead_query: Query<(Entity, &mut Dying)>,
    time: Res<Time>,
) {
    for (entity, mut dying) in dead_query.iter_mut() {
        dying.tick(time.delta());
        if dying.finished() {
            commands.entity(entity).despawn();
        }
    }
}
