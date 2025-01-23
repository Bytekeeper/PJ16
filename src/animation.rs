use bevy::prelude::*;

pub struct AnimationPlugin;

pub struct Animation {
    pub image: Handle<Image>,
    pub atlas: TextureAtlas,
    pub indices: AnimationIndices,
}

#[derive(Component, Copy, Clone, Debug)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, animate_sprite);
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        Option<&mut Sprite>,
        Option<&mut ImageNode>,
    )>,
) {
    for (indices, mut timer, mut sprite, mut image_node) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(Some(atlas)) = sprite
                .as_deref_mut()
                .map(|s| &mut s.texture_atlas)
                .or_else(|| image_node.as_deref_mut().map(|i| &mut i.texture_atlas))
            {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}
