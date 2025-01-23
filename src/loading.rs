use crate::tiled::TiledMap;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_enoki::Particle2dEffect;
use bevy_kira_audio::AudioSource;

use crate::animation::{Animation, AnimationIndices};
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
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>()
                .load_collection::<EffectAssets>()
                .load_collection::<TileMapAssets>()
                .load_collection::<Fonts>()
                .init_resource::<Animations>(),
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
    #[asset(path = "textures/player_sword_1.png")]
    pub player_sword: Handle<Image>,
    #[asset(path = "textures/Player_Mace_1.png")]
    pub player_mace: Handle<Image>,
    #[asset(path = "textures/Player_Spear_1.png")]
    pub player_spear: Handle<Image>,
    #[asset(path = "textures/player_Bow.png")]
    pub player_bow: Handle<Image>,
    #[asset(path = "textures/placeholder/spritesheet.png")]
    pub tiles: Handle<Image>,
    #[asset(path = "textures/Melee_Enemy_1.png")]
    pub enemy_1: Handle<Image>,
    #[asset(path = "textures/Melee_Enemy_2.png")]
    pub enemy_2: Handle<Image>,
    #[asset(path = "textures/Melee_Enemy_3.png")]
    pub enemy_3: Handle<Image>,
    #[asset(path = "textures/Player_Life_Anim.png")]
    pub beating_heart: Handle<Image>,
    #[asset(path = "textures/Player_Life_Depleted.png")]
    pub broken_heart: Handle<Image>,
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
}

#[derive(Resource)]
pub struct Animations {
    pub enemy_1_walk: Animation,
    pub beating_heart: Animation,
}

#[derive(AssetCollection, Resource)]
pub struct Fonts {
    #[asset(path = "Pixelated Elegance.ttf")]
    pub font: Handle<Font>,
}

impl FromWorld for Animations {
    fn from_world(world: &mut World) -> Self {
        let mut texture_atlas_layouts = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .expect("Missing TextureAtlasLayout assets");
        let enemy_1_walk_atlas = TextureAtlas {
            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(16),
                13,
                1,
                None,
                None,
            )),
            index: 0,
        };
        let beating_heart_atlas = TextureAtlas {
            layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(19),
                11,
                1,
                None,
                None,
            )),
            index: 0,
        };
        let texture_assets = world
            .get_resource::<TextureAssets>()
            .expect("Textures not loaded");
        Self {
            enemy_1_walk: Animation {
                image: texture_assets.enemy_1.clone(),
                atlas: enemy_1_walk_atlas,
                indices: AnimationIndices { first: 0, last: 12 },
            },
            beating_heart: Animation {
                image: texture_assets.beating_heart.clone(),
                atlas: beating_heart_atlas,
                indices: AnimationIndices { first: 0, last: 10 },
            },
        }
    }
}
