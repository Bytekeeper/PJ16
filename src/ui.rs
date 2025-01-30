use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

use crate::actions::{Actions, Health};
use crate::loading::{Fonts, TextureAssets};
use crate::player::Player;
use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cooldown_displays)
            .add_systems(
                Update,
                (update_health_display, update_score_display).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct CooldownDisplay(pub Entity);

#[derive(Component)]
pub struct ScoreDisplay;

#[derive(Component)]
pub struct HealthDisplay;

fn update_cooldown_displays(
    mut display_query: Query<(Entity, &mut Text2d, &mut Visibility, &CooldownDisplay)>,
    actions_query: Query<&Actions>,
    mut commands: Commands,
) {
    for (cooldown_entity, mut display_text, mut display_visibility, CooldownDisplay(target)) in
        display_query.iter_mut()
    {
        let Ok(actions) = actions_query.get(*target) else {
            // Maybe the entity is dead? Remove the cool-down display.
            commands.entity(cooldown_entity).despawn();
            return;
        };
        match actions {
            Actions::Cooldown(trigger_cooldown) => {
                display_text.0 = format!("{}", trigger_cooldown.remaining_secs().ceil() as u32);
                *display_visibility = Visibility::Inherited;
            }
            _ => {
                *display_visibility = Visibility::Hidden;
            }
        }
    }
}

fn update_health_display(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    player_query: Query<&Health, (Changed<Health>, With<Player>)>,
    health_query: Query<Entity, With<HealthDisplay>>,
) {
    let health_entity = health_query.get_single();
    let Ok(player_health) = player_query.get_single() else {
        if let Ok(health_entity) = health_entity {
            commands.entity(health_entity).despawn_recursive();
        }
        return;
    };
    let health_entity = if let Ok(health_entity) = health_entity {
        commands.entity(health_entity).despawn_descendants();
        health_entity
    } else {
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
            .id()
    };
    commands.entity(health_entity).with_children(|parent| {
        for i in 0..player_health.max_health {
            if i < player_health.health {
                parent.spawn((
                    AseUiAnimation {
                        aseprite: textures.player_life.clone(),
                        animation: Animation::tag("beating"),
                    },
                    Node {
                        width: Val::Px(38.0),
                        height: Val::Px(38.0),
                        ..default()
                    },
                ));
            } else {
                parent.spawn((
                    AseUiAnimation {
                        aseprite: textures.player_life.clone(),
                        animation: Animation::tag("depleted"),
                    },
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

fn update_score_display(
    mut commands: Commands,
    fonts: Res<Fonts>,
    player_query: Query<&Player, Changed<Player>>,
    score_query: Query<Entity, With<ScoreDisplay>>,
) {
    let Ok(player) = player_query.get_single() else {
        return;
    };
    let score_entity = score_query.get_single();
    let mut ec = if let Ok(score_entity) = score_entity {
        commands.entity(score_entity)
    } else {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Percent(40.0),
                width: Val::Percent(100.0),
                height: Val::Px(64.0),
                ..default()
            },
            StateScoped(GameState::Playing),
            ScoreDisplay,
        ))
    };
    ec.insert((
        Text::new(player.score.to_string()),
        TextFont {
            font: fonts.font.clone(),
            font_size: 48.0,
            ..default()
        },
    ));
}
