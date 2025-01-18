#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod enemies;
mod loading;
mod menu;
mod player;
mod tilemap;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::enemies::EnemiesPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::tilemap::TilemapPlugin;

use avian2d::{prelude::Gravity, PhysicsPlugins};
use bevy_enoki::EnokiPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
                TilemapPlugin,
                EnemiesPlugin,
                PhysicsPlugins::default(),
                EnokiPlugin,
            ))
            .enable_state_scoped_entities::<GameState>()
            .insert_resource(Gravity(Vec2::ZERO));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
