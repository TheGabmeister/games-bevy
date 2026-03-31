use bevy::prelude::*;

use crate::constants::*;

/// Unified input state gathered from keyboard and gamepad each frame.
#[derive(Resource, Default)]
pub struct ActionInput {
    pub move_x: f32,
    pub running: bool,
    pub jump_pressed: bool,
    pub jump_just_pressed: bool,
    pub jump_just_released: bool,
    pub duck: bool,
    pub shoot_just_pressed: bool,
    pub pause_just_pressed: bool,
    pub confirm_just_pressed: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionInput>()
            .add_systems(PreUpdate, gather_input);
    }
}

fn gather_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut action: ResMut<ActionInput>,
) {
    *action = ActionInput::default();

    // ── Gamepad (any connected pad contributes) ──
    for gamepad in &gamepads {
        // Movement — stick
        let stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

        if stick_x < -STICK_DEADZONE {
            action.move_x = -1.0;
        } else if stick_x > STICK_DEADZONE {
            action.move_x = 1.0;
        }

        // Movement — d-pad (overrides stick)
        if gamepad.pressed(GamepadButton::DPadLeft) {
            action.move_x = -1.0;
        }
        if gamepad.pressed(GamepadButton::DPadRight) {
            action.move_x = 1.0;
        }

        // Duck
        action.duck |= gamepad.pressed(GamepadButton::DPadDown)
            || stick_y < -STICK_DEADZONE;

        // Jump — South (A / Cross)
        action.jump_pressed |= gamepad.pressed(GamepadButton::South);
        action.jump_just_pressed |= gamepad.just_pressed(GamepadButton::South);
        action.jump_just_released |= gamepad.just_released(GamepadButton::South);

        // Run — West (X / Square) or right shoulder
        action.running |= gamepad.pressed(GamepadButton::West)
            || gamepad.pressed(GamepadButton::RightTrigger);

        // Shoot — East (B / Circle)
        action.shoot_just_pressed |= gamepad.just_pressed(GamepadButton::East);

        // Pause — Start
        action.pause_just_pressed |= gamepad.just_pressed(GamepadButton::Start);

        // Confirm — Start or South (menus)
        action.confirm_just_pressed |= gamepad.just_pressed(GamepadButton::Start)
            || gamepad.just_pressed(GamepadButton::South);
    }

    // ── Keyboard (overrides gamepad for directional input) ──
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        action.move_x = -1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        action.move_x = 1.0;
    }

    action.running |= keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    action.jump_pressed |= keyboard.pressed(KeyCode::Space)
        || keyboard.pressed(KeyCode::ArrowUp);
    action.jump_just_pressed |= keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::ArrowUp);
    action.jump_just_released |= keyboard.just_released(KeyCode::Space)
        || keyboard.just_released(KeyCode::ArrowUp);

    action.duck |= keyboard.pressed(KeyCode::ArrowDown)
        || keyboard.pressed(KeyCode::KeyS);

    action.shoot_just_pressed |= keyboard.just_pressed(KeyCode::KeyJ)
        || keyboard.just_pressed(KeyCode::KeyE);

    action.pause_just_pressed |= keyboard.just_pressed(KeyCode::Escape);

    action.confirm_just_pressed |= keyboard.just_pressed(KeyCode::Enter);
}
