use avian2d::prelude::*;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

use crate::actions::{Actions, Health};
use crate::loading::{Fonts, TextureAssets};
use crate::physics::CollisionLayer;
use crate::ui::CooldownDisplay;
use crate::GameState;

pub struct PlayerPlugin;

#[derive(Default, Component)]
#[require(RigidBody)]
pub struct Player {
    pub form: PlayerForm,
    pub score: u32,
}

#[derive(Default, Clone, Copy)]
pub enum PlayerForm {
    #[default]
    Sword,
    Bow,
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
                update_direction_arrow.run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>, fonts: Res<Fonts>) {
    commands
        .spawn((
            AseSpriteAnimation {
                aseprite: textures.player_sword.clone(),
                animation: Animation::tag("flaming"),
            },
            Transform::from_translation(Vec3::new(0., 0., 2.)),
            Collider::circle(5.0),
            CollisionLayers::new(
                CollisionLayer::Player,
                [
                    CollisionLayer::Default,
                    CollisionLayer::Enemy,
                    CollisionLayer::EnemyProjectile,
                ],
            ),
            LockedAxes::ROTATION_LOCKED,
            LinearDamping(10.0),
            Player::default(),
            Health {
                owner: 0,
                max_health: 5,
                health: 5,
            },
            Actions::default(),
            StateScoped(GameState::Playing),
        ))
        .with_children(|commands| {
            commands.spawn((
                CooldownDisplay(commands.parent_entity()),
                Text2d::new("1"),
                // Workaround to make Bevy not blur the font
                Transform::from_translation(vec3(12.0, -10.0, 1.0))
                    .with_scale(Vec3::splat(1.0 / 4.0)),
                TextFont {
                    font: fonts.font.clone(),
                    font_size: 64.0,
                    ..default()
                },
            ));
        });
}

fn update_direction_arrow(
    textures: Res<TextureAssets>,
    mut arrow_query: Query<(Entity, &mut Transform), (With<DirectionArrow>, Without<Player>)>,
    player_query: Query<(&Actions, &Transform), With<Player>>,
    mut commands: Commands,
) {
    let Ok((actions, player_transform)) = player_query.get_single() else {
        return;
    };
    let Actions::Charging {
        charge,
        trigger_direction: Some(player_direction),
    } = actions
    else {
        // No direction selected: remove arrow display
        for (entity, _) in arrow_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    let direction = player_direction.normalize().extend(0.0);
    let as_quat = Quat::from_rotation_arc(Vec3::Y, direction);
    let mut target_transform = player_transform.with_rotation(as_quat);
    target_transform.translation += direction * 20.0;
    target_transform.scale = Vec3::splat(1.0 + charge.fraction());

    if let Some((_, mut arrow_transform)) = arrow_query.iter_mut().next() {
        *arrow_transform = target_transform;
        return;
    }
    // No existing arrow
    commands.spawn((
        Sprite::from_image(textures.arrow.clone()),
        target_transform,
        DirectionArrow,
        StateScoped(GameState::Playing),
    ));
}
