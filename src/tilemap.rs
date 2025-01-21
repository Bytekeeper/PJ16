use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilemapRenderSettings;

use crate::loading::TileMapAssets;
use crate::tiled::*;
use crate::GameState;

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_ecs_tilemap::TilemapPlugin)
            .add_systems(OnEnter(GameState::Playing), setup_map);
    }
}

fn setup_map(mut commands: Commands, maps: Res<TileMapAssets>) {
    commands.spawn((
        TiledMapBundle {
            tiled_map: TiledMapHandle(maps.level_1.clone()),
            render_settings: TilemapRenderSettings {
                render_chunk_size: UVec2::new(64, 1),
                y_sort: true,
            },
            ..default()
        },
        StateScoped(GameState::Playing),
    ));
}
