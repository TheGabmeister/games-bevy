use bevy::prelude::*;

use crate::{
    components::{
        Facing, Health, Hitbox, Hurtbox, InvulnerabilityTimer, Knockback, MoveSpeed, Player,
        RoomEntity, SolidBody, Velocity,
    },
    constants,
    input::InputActions,
    resources::{DialogueState, PlayerVitals, RoomTransitionState},
    rendering::{circle_mesh, color_material, WorldColor},
    room::RoomLoadedMessage,
    states::AppState,
};

const PLAYER_RADIUS: f32 = 8.0;
const PLAYER_SPEED: f32 = 90.0;
const FACING_INDICATOR_SIZE: f32 = 5.0;

pub struct PlayerPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerSet {
    Input,
}

#[derive(Component)]
struct FacingIndicator;

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
        )
        .add_systems(
            Update,
            sync_facing_indicator.run_if(in_state(AppState::Playing)),
        );
    }
}

fn spawn_player_on_room_load(
    mut commands: Commands,
    mut room_loaded: MessageReader<RoomLoadedMessage>,
    player_vitals: Res<PlayerVitals>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for message in room_loaded.read() {
        let _room = message.room;
        let mut invulnerability = Timer::from_seconds(0.0, TimerMode::Once);
        invulnerability.set_elapsed(invulnerability.duration());

        commands
            .spawn((
                Name::new("Player"),
                Player,
                RoomEntity,
                Velocity::default(),
                MoveSpeed(PLAYER_SPEED),
                Facing::Down,
                Health {
                    current: player_vitals.current_health,
                    max: player_vitals.max_health,
                },
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
                InvulnerabilityTimer(invulnerability),
                circle_mesh(meshes.as_mut(), PLAYER_RADIUS),
                color_material(materials.as_mut(), WorldColor::Player),
                Transform::from_xyz(
                    message.player_spawn.x,
                    message.player_spawn.y,
                    constants::render_layers::ENTITIES,
                ),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Name::new("PlayerFacingIndicator"),
                    FacingIndicator,
                    Mesh2d(meshes.add(Triangle2d::new(
                        Vec2::new(0.0, FACING_INDICATOR_SIZE),
                        Vec2::new(-FACING_INDICATOR_SIZE * 0.7, -FACING_INDICATOR_SIZE),
                        Vec2::new(FACING_INDICATOR_SIZE * 0.7, -FACING_INDICATOR_SIZE),
                    ))),
                    color_material(materials.as_mut(), WorldColor::Accent),
                    Transform {
                        translation: Vec3::new(0.0, 0.0, 1.0),
                        rotation: Quat::from_rotation_z(std::f32::consts::PI),
                        ..default()
                    },
                ));
            });
    }
}

fn update_player_motion_and_facing(
    actions: Res<InputActions>,
    transition: Res<RoomTransitionState>,
    dialogue: Res<DialogueState>,
    mut player: Query<(&mut Velocity, &MoveSpeed, &mut Facing), With<Player>>,
) {
    let Ok((mut velocity, move_speed, mut facing)) = player.single_mut() else {
        return;
    };

    if dialogue.is_active() {
        velocity.0 = Vec2::ZERO;
        return;
    }

    if transition.locked {
        velocity.0 = Vec2::ZERO;
        return;
    }

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

fn sync_facing_indicator(
    players: Query<(&Facing, &Children), (With<Player>, Changed<Facing>)>,
    mut indicators: Query<&mut Transform, With<FacingIndicator>>,
) {
    for (facing, children) in &players {
        for child in children.iter() {
            if let Ok(mut transform) = indicators.get_mut(child) {
                transform.translation = Vec3::new(0.0, 0.0, 1.0);
                transform.rotation = Quat::from_rotation_z(match facing {
                    Facing::Up => 0.0,
                    Facing::Down => std::f32::consts::PI,
                    Facing::Left => std::f32::consts::FRAC_PI_2,
                    Facing::Right => -std::f32::consts::FRAC_PI_2,
                });
            }
        }
    }
}
