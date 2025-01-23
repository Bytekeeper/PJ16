use bevy::prelude::*;

use crate::actions::{Actions, Health};
use crate::animation::AnimationTimer;
use crate::loading::{Animations, TextureAssets};
use crate::player::Player;
use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cooldown_displays)
            .add_systems(
                Update,
                update_health_display.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct CooldownDisplay(pub Entity);

fn update_cooldown_displays(
    mut display_query: Query<(&mut Text2d, &mut Visibility, &CooldownDisplay)>,
    actions_query: Query<&Actions>,
) {
    for (mut display_text, mut display_visibility, CooldownDisplay(target)) in
        display_query.iter_mut()
    {
        if let Ok(actions) = actions_query.get(*target) {
            if actions.trigger_cooldown.finished() {
                *display_visibility = Visibility::Hidden;
            } else {
                display_text.0 = format!(
                    "{}",
                    actions.trigger_cooldown.remaining_secs().ceil() as u32
                );
                *display_visibility = Visibility::Inherited;
            }
        }
    }
}

#[derive(Component)]
pub struct HealthDisplay;

fn update_health_display(
    mut commands: Commands,
    animations: Res<Animations>,
    textures: Res<TextureAssets>,
    player_query: Query<(&Health, Ref<Player>)>,
) {
    let Ok((player_health, player)) = player_query.get_single() else {
        return;
    };
    if player.is_added() {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(100.0),
                    top: Val::Px(20.0),
                    width: Val::Px(128.0),
                    height: Val::Px(64.0),
                    ..default()
                },
                StateScoped(GameState::Playing),
                HealthDisplay,
            ))
            .with_children(|parent| {
                for i in 0..player_health.max_health {
                    if i < player_health.health {
                        let mut atlas = animations.beating_heart.atlas.clone();
                        atlas.index = i as usize;
                        parent.spawn((
                            ImageNode::from_atlas_image(
                                animations.beating_heart.image.clone(),
                                atlas,
                            ),
                            animations.beating_heart.indices,
                            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                            Node {
                                width: Val::Px(38.0),
                                height: Val::Px(38.0),
                                ..default()
                            },
                        ));
                    } else {
                        parent.spawn((
                            ImageNode::new(textures.broken_heart.clone()),
                            Node {
                                width: Val::Px(38.0),
                                height: Val::Px(38.0),
                                ..default()
                            },
                        ));
                    }
                }
            });
    }
}
