use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct InputActions {
    pub move_up: bool,
    pub move_down: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub attack: bool,
    pub item_use: bool,
    pub pause: bool,
    pub confirm: bool,
    pub cancel: bool,
}

impl InputActions {
    pub fn movement_axis(&self) -> Vec2 {
        Vec2::new(
            axis_value(self.move_right, self.move_left),
            axis_value(self.move_up, self.move_down),
        )
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputActions>()
            .add_systems(PreUpdate, (read_keyboard, read_gamepad).chain());
    }
}

fn read_keyboard(keys: Res<ButtonInput<KeyCode>>, mut actions: ResMut<InputActions>) {
    actions.move_up = keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW);
    actions.move_down = keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS);
    actions.move_left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    actions.move_right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);

    actions.attack = keys.just_pressed(KeyCode::KeyZ)
        || keys.just_pressed(KeyCode::KeyJ)
        || keys.just_pressed(KeyCode::ControlLeft);
    actions.item_use = keys.just_pressed(KeyCode::KeyX)
        || keys.just_pressed(KeyCode::KeyK)
        || keys.just_pressed(KeyCode::ControlRight);
    actions.pause = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Tab);
    actions.confirm = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space);
    actions.cancel = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Backspace);
}

const STICK_THRESHOLD: f32 = 0.5;

fn read_gamepad(gamepads: Query<&Gamepad>, mut actions: ResMut<InputActions>) {
    let Some(gamepad) = gamepads.iter().next() else {
        return;
    };

    let dpad = gamepad.dpad();
    let stick = gamepad.left_stick();

    actions.move_up = actions.move_up || dpad.y > 0.5 || stick.y > STICK_THRESHOLD;
    actions.move_down = actions.move_down || dpad.y < -0.5 || stick.y < -STICK_THRESHOLD;
    actions.move_left = actions.move_left || dpad.x < -0.5 || stick.x < -STICK_THRESHOLD;
    actions.move_right = actions.move_right || dpad.x > 0.5 || stick.x > STICK_THRESHOLD;

    actions.attack = actions.attack || gamepad.just_pressed(GamepadButton::South);
    actions.item_use = actions.item_use || gamepad.just_pressed(GamepadButton::East);
    actions.pause = actions.pause || gamepad.just_pressed(GamepadButton::Start);
    actions.confirm = actions.confirm || gamepad.just_pressed(GamepadButton::South);
    actions.cancel = actions.cancel || gamepad.just_pressed(GamepadButton::East);
}

fn axis_value(positive: bool, negative: bool) -> f32 {
    match (positive, negative) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    }
}
