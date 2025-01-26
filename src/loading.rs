use crate::tiled::TiledMap;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_enoki::Particle2dEffect;
use bevy_kira_audio::AudioSource;

use crate::GameState;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "dynamic_asset.assets.ron",
                )
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>()
                .load_collection::<EffectAssets>()
                .load_collection::<TileMapAssets>()
                .load_collection::<Fonts>(),
        );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/woosh.ogg")]
    pub woosh: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/arrow.png")]
    pub arrow: Handle<Image>,
    #[asset(path = "textures/player_sword.aseprite")]
    pub player_sword: Handle<Aseprite>,
    //#[asset(path = "textures/Player_Mace_1.png")]
    //pub player_mace: Handle<Image>,
    //#[asset(path = "textures/Player_Spear_1.png")]
    //pub player_spear: Handle<Image>,
    //#[asset(path = "textures/player_Bow.png")]
    //pub player_bow: Handle<Image>,
    #[asset(path = "textures/Melee_Enemy_1.aseprite")]
    pub enemy_1: Handle<Aseprite>,
    #[asset(path = "textures/Melee_Enemy_1_Attack.aseprite")]
    pub enemy_1_attack: Handle<Aseprite>,
    //#[asset(path = "textures/Melee_Enemy_2.aseprite")]
    //pub enemy_2: Handle<Aseprite>,
    //#[asset(path = "textures/Melee_Enemy_3.aseprite")]
    //pub enemy_3: Handle<Aseprite>,
    #[asset(path = "textures/Player_Life.aseprite")]
    pub player_life: Handle<Aseprite>,
    #[asset(path = "textures/title.png")]
    pub title: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct TileMapAssets {
    #[asset(path = "level1.tmx")]
    pub level_1: Handle<TiledMap>,
}

#[derive(AssetCollection, Resource)]
pub struct EffectAssets {
    #[asset(path = "effects/sword-slash.ron")]
    pub sword_slash: Handle<Particle2dEffect>,
    #[asset(path = "effects/enemy_1-attack.ron")]
    pub enemy_1_attack: Handle<Particle2dEffect>,
}

#[derive(AssetCollection, Resource)]
pub struct Fonts {
    #[asset(path = "Pixelated Elegance.ttf")]
    pub font: Handle<Font>,
}
