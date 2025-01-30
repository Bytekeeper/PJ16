use bevy::math::vec2;
use bevy::prelude::*;
use std::collections::VecDeque;

use crate::actions::{Actions, Effect, Step};
use crate::loading::AudioAssets;
use crate::player::{Player, PlayerForm};
use crate::GameState;

pub const FOLLOW_EPSILON: f32 = 5.;

#[derive(SystemSet, Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct InputSet;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            keyboard_input
                .in_set(InputSet)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Mush all gamepads together...
    gamepad_input: Query<&Gamepad>,
    touch_input: Res<Touches>,
    mut player: Query<(&mut Actions, &Transform, &Player)>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    audio_assets: Res<AudioAssets>,
    time: Res<Time>,
) {
    let Ok((mut actions, player_transform, player)) = player.get_single_mut() else {
        return;
    };

    let mut player_direction = Vec2::new(
        get_movement(GameControl::Right, &keyboard_input)
            - get_movement(GameControl::Left, &keyboard_input),
        get_movement(GameControl::Up, &keyboard_input)
            - get_movement(GameControl::Down, &keyboard_input),
    );

    let left_stick = gamepad_input
        .iter()
        .flat_map(|g| {
            let (Some(x), Some(y)) = (
                g.get(GamepadAxis::LeftStickX),
                g.get(GamepadAxis::LeftStickY),
            ) else {
                return None;
            };
            let dir = vec2(x, y);
            const DEAD_ZONE: f32 = 0.1;
            (dir.length() > DEAD_ZONE).then_some(dir)
        })
        .next();
    if let Some(left_stick) = left_stick {
        player_direction = left_stick;
    }

    if let Some(touch_position) = touch_input.first_pressed_position() {
        let (camera, camera_transform) = camera.single();
        if let Ok(touch_position) = camera.viewport_to_world_2d(camera_transform, touch_position) {
            let diff = touch_position - player_transform.translation.xy();
            if diff.length() > FOLLOW_EPSILON {
                player_direction = diff.normalize();
            }
        }
    }

    let player_direction = if player_direction != Vec2::ZERO {
        Some(player_direction.normalize())
    } else {
        None
    };

    let actions = &mut *actions;
    let triggering = keyboard_input.pressed(KeyCode::Space)
        || gamepad_input
            .iter()
            .any(|g| g.pressed(GamepadButton::South));
    if !triggering {
        if let Actions::Charging {
            charge,
            trigger_direction,
        } = actions
        {
            if let Some(trigger_direction) = player_direction.or(*trigger_direction) {
                match player.form {
                    PlayerForm::Sword => {
                        let mut steps = VecDeque::from([Step::from_timer(Timer::from_seconds(
                            0.0,
                            TimerMode::Once,
                        ))
                        .with_effect(Effect::Circle)
                        .with_forward(20.0)
                        .with_sfx(audio_assets.woosh.clone())]);
                        for _ in 0..charge.elapsed_secs() as u32 {
                            steps.push_back(
                                Step::from_timer(Timer::from_seconds(0.2, TimerMode::Once))
                                    .with_effect(Effect::Circle)
                                    .with_forward(20.0)
                                    .with_sfx(audio_assets.woosh.clone()),
                            );
                        }
                        *actions = Actions::Executing {
                            trigger_direction,
                            pending_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
                            steps,
                        };
                    }
                    PlayerForm::Bow => todo!(),
                }
            } else {
                // No direction was selected, nothing will be done but no cool-down will be
                // applied
                *actions = Actions::Idle;
            }
            // Unless charging, stopping releasing the trigger will not do anything
        } else if let Actions::Idle = actions {
            if keyboard_input.pressed(KeyCode::Digit1) {
                *actions = Actions::ChangePlayerForm(PlayerForm::Sword);
            } else if keyboard_input.pressed(KeyCode::Digit2) {
                *actions = Actions::ChangePlayerForm(PlayerForm::Bow);
            }
        }
    } else {
        if matches!(actions, Actions::Idle) {
            *actions = Actions::Charging {
                charge: default(),
                trigger_direction: default(),
            };
        }
        if let Actions::Charging {
            charge,
            trigger_direction,
        } = actions
        {
            charge.tick(time.delta());
            *trigger_direction = player_direction.or(*trigger_direction);
        }
        // No other Action state allows charging currently
    }
}

pub enum GameControl {
    Up,
    Down,
    Left,
    Right,
}

impl GameControl {
    pub fn pressed(&self, keyboard_input: &Res<ButtonInput<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp)
            }
            GameControl::Down => {
                keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown)
            }
            GameControl::Left => {
                keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft)
            }
            GameControl::Right => {
                keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight)
            }
        }
    }
}

pub fn get_movement(control: GameControl, input: &Res<ButtonInput<KeyCode>>) -> f32 {
    if control.pressed(input) {
        1.0
    } else {
        0.0
    }
}
