use bevy::prelude::*;

use crate::{
    components::{
        Facing, Health, Hitbox, Hurtbox, Knockback, MoveSpeed, Player, RoomEntity, SolidBody,
        Velocity,
    },
    constants,
    input::InputActions,
    rendering::{circle_mesh, color_material, WorldColor},
    room::RoomLoadedMessage,
    states::AppState,
};

const PLAYER_RADIUS: f32 = 8.0;
const PLAYER_SPEED: f32 = 90.0;

pub struct PlayerPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerSet {
    Input,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_player_on_room_load.run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            update_player_motion_and_facing
                .in_set(PlayerSet::Input)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn spawn_player_on_room_load(
    mut commands: Commands,
    mut room_loaded: MessageReader<RoomLoadedMessage>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for message in room_loaded.read() {
        let _room = message.room;

        commands.spawn((
            Name::new("Player"),
            Player,
            RoomEntity,
            Velocity::default(),
            MoveSpeed(PLAYER_SPEED),
            Facing::Down,
            Health::new(6),
            Hitbox {
                half_size: Vec2::splat(PLAYER_RADIUS),
            },
            Hurtbox {
                half_size: Vec2::splat(PLAYER_RADIUS),
            },
            SolidBody {
                half_size: Vec2::splat(PLAYER_RADIUS),
            },
            Knockback::default(),
            circle_mesh(meshes.as_mut(), PLAYER_RADIUS),
            color_material(materials.as_mut(), WorldColor::Player),
            Transform::from_xyz(
                message.player_spawn.x,
                message.player_spawn.y,
                constants::render_layers::ENTITIES,
            ),
        ));
    }
}

fn update_player_motion_and_facing(
    actions: Res<InputActions>,
    mut player: Query<(&mut Velocity, &MoveSpeed, &mut Facing), With<Player>>,
) {
    let Ok((mut velocity, move_speed, mut facing)) = player.single_mut() else {
        return;
    };

    let mut axis = actions.movement_axis();
    if axis.x != 0.0 && axis.y != 0.0 {
        axis.y = 0.0;
    }

    velocity.0 = axis * move_speed.0;

    *facing = if axis.x < 0.0 {
        Facing::Left
    } else if axis.x > 0.0 {
        Facing::Right
    } else if axis.y > 0.0 {
        Facing::Up
    } else if axis.y < 0.0 {
        Facing::Down
    } else {
        *facing
    };
}
