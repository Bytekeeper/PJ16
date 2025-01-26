use avian2d::prelude::PhysicsLayer;
use bevy::prelude::*;

#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    //Player,
    //Enemy,
    //Item,
    //Ground,
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, _app: &mut App) {}
}
