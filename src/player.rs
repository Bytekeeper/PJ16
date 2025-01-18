use bevy::prelude::*;

use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
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
        Player,
    ));
}

fn move_player(
    time: Res<Time>,
    actions: Res<Actions>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if actions.player_direction.is_none() {
        return;
    }
    let speed = 150.;
    let movement = Vec3::new(
        actions.player_direction.unwrap().x * speed * time.delta_secs(),
        actions.player_direction.unwrap().y * speed * time.delta_secs(),
        0.,
    );
    for mut player_transform in &mut player_query {
        player_transform.translation += movement;
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

    'outer: for player_transform in player_query.iter() {
        let direction = player_direction.normalize().extend(0.0);
        let as_quat = Quat::from_rotation_arc(Vec3::Y, direction);
        let mut target_transform = player_transform.clone().with_rotation(as_quat);
        target_transform.translation += direction * 20.0;

        for (_, mut arrow_transform) in arrow_query.iter_mut() {
            *arrow_transform = target_transform;
            continue 'outer;
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
