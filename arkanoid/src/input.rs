use bevy::prelude::*;

/// Snapshot of player intent for the current frame, populated from keyboard,
/// mouse, and gamepad. Gameplay systems read this instead of raw input devices.
#[derive(Resource, Default)]
pub struct InputActions {
    pub move_left: bool,
    pub move_right: bool,
    pub launch: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputActions>()
            .add_systems(PreUpdate, (read_keyboard_mouse, read_gamepad).chain());
    }
}

fn read_keyboard_mouse(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut actions: ResMut<InputActions>,
) {
    actions.move_left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    actions.move_right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);
    actions.launch =
        keys.just_pressed(KeyCode::Space) || mouse_buttons.just_pressed(MouseButton::Left);
}

const STICK_THRESHOLD: f32 = 0.5;

fn read_gamepad(gamepads: Query<&Gamepad>, mut actions: ResMut<InputActions>) {
    let Some(gamepad) = gamepads.iter().next() else {
        return;
    };

    let dpad = gamepad.dpad();
    let stick = gamepad.left_stick();

    // Merge with keyboard/mouse (OR) so either device can drive the Vaus.
    actions.move_left = actions.move_left || dpad.x < -0.5 || stick.x < -STICK_THRESHOLD;
    actions.move_right = actions.move_right || dpad.x > 0.5 || stick.x > STICK_THRESHOLD;
    actions.launch = actions.launch || gamepad.just_pressed(GamepadButton::South);
}
