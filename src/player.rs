use avian2d::prelude::*;
use bevy::prelude::*;

use crate::actions::{Actions, Health};
use crate::loading::TextureAssets;
use crate::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
#[require(RigidBody)]
pub struct Player;

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

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        Sprite::from_image(textures.player_sword.clone()),
        Transform::from_translation(Vec3::new(0., 0., 2.)),
        Collider::circle(5.0),
        LockedAxes::ROTATION_LOCKED,
        LinearDamping(10.0),
        Player,
        Health { owner: 0 },
        Actions::default(),
        StateScoped(GameState::Playing),
    ));
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
    let Some(player_direction) = actions.trigger_direction else {
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
