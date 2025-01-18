use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_enemies);
    }
}

fn spawn_enemies(mut commands: Commands, textures: Res<TextureAssets>) {}
