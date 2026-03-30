use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct InputActions {
    pub move_left: bool,
    pub move_right: bool,
    pub rotate_cw: bool,
    pub rotate_ccw: bool,
    pub soft_drop: bool,
    pub hard_drop: bool,
    pub hold: bool,
    pub pause: bool,
    pub start_restart: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputActions>()
            .add_systems(PreUpdate, (clear_input, read_keyboard, read_gamepad).chain());
    }
}

fn clear_input(mut actions: ResMut<InputActions>) {
    *actions = InputActions::default();
}

fn read_keyboard(keys: Res<ButtonInput<KeyCode>>, mut actions: ResMut<InputActions>) {
    actions.move_left |= keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    actions.move_right |= keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);
    actions.rotate_cw |= keys.just_pressed(KeyCode::KeyX) || keys.just_pressed(KeyCode::KeyE);
    actions.rotate_ccw |= keys.just_pressed(KeyCode::KeyZ) || keys.just_pressed(KeyCode::KeyQ);
    actions.soft_drop |= keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS);
    actions.hard_drop |=
        keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::Space);
    actions.hold |= keys.just_pressed(KeyCode::KeyC)
        || keys.just_pressed(KeyCode::ShiftLeft)
        || keys.just_pressed(KeyCode::ShiftRight);
    actions.pause |= keys.just_pressed(KeyCode::Escape);
    actions.start_restart |= keys.just_pressed(KeyCode::Enter);
}

const STICK_THRESHOLD: f32 = 0.5;

fn read_gamepad(gamepads: Query<&Gamepad>, mut actions: ResMut<InputActions>) {
    let Some(gamepad) = gamepads.iter().next() else {
        return;
    };

    let dpad = gamepad.dpad();
    let stick = gamepad.left_stick();

    // Held actions — merge with keyboard (OR).
    actions.move_left = actions.move_left || dpad.x < -0.5 || stick.x < -STICK_THRESHOLD;
    actions.move_right = actions.move_right || dpad.x > 0.5 || stick.x > STICK_THRESHOLD;
    actions.soft_drop = actions.soft_drop || dpad.y < -0.5 || stick.y < -STICK_THRESHOLD;

    // Single-fire actions — merge with keyboard (OR).
    actions.rotate_cw = actions.rotate_cw
        || gamepad.just_pressed(GamepadButton::East)
        || gamepad.just_pressed(GamepadButton::North);
    actions.rotate_ccw = actions.rotate_ccw
        || gamepad.just_pressed(GamepadButton::West)
        || gamepad.just_pressed(GamepadButton::South);
    actions.hard_drop = actions.hard_drop
        || gamepad.just_pressed(GamepadButton::DPadUp)
        || gamepad.just_pressed(GamepadButton::RightTrigger);
    actions.hold = actions.hold || gamepad.just_pressed(GamepadButton::LeftTrigger);
    actions.pause = actions.pause || gamepad.just_pressed(GamepadButton::Start);
    actions.start_restart = actions.start_restart || gamepad.just_pressed(GamepadButton::Select);
}
