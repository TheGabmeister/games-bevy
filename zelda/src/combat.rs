use bevy::prelude::*;

use crate::{
    collision::CollisionSet,
    components::{
        Damage, Enemy, Facing, Health, Hitbox, Hurtbox, InvulnerabilityTimer, Knockback, Lifetime,
        Player, RoomEntity, SwordAttack,
    },
    constants,
    input::InputActions,
    player::PlayerSet,
    rendering::{color_material, rectangle_mesh, WorldColor},
    resources::{CurrentRoom, PlayerVitals, RoomId, RoomTransitionState},
    states::AppState,
};

const SWORD_LIFETIME_SECS: f32 = 0.12;
const PLAYER_INVULNERABILITY_SECS: f32 = 0.75;
const PLAYER_KNOCKBACK_SPEED: f32 = 140.0;
const SWORD_DAMAGE: u8 = 1;

pub struct CombatPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CombatSet {
    AttackSpawn,
    AttackResolve,
    Damage,
    Death,
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerVitals>()
            .configure_sets(
                Update,
                (
                    CombatSet::AttackSpawn,
                    CombatSet::AttackResolve,
                    CombatSet::Damage,
                    CombatSet::Death,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnEnter(AppState::Title), reset_player_session)
            .add_systems(
                Update,
                spawn_sword_attack
                    .in_set(CombatSet::AttackSpawn)
                    .after(PlayerSet::Input),
            )
            .add_systems(
                Update,
                (tick_lifetimes, resolve_sword_hits).in_set(CombatSet::AttackResolve),
            )
            .add_systems(
                Update,
                (tick_player_invulnerability, resolve_player_enemy_damage)
                    .in_set(CombatSet::Damage)
                    .after(CollisionSet::Resolve),
            )
            .add_systems(Update, handle_player_death.in_set(CombatSet::Death));
    }
}

fn reset_player_session(
    mut player_vitals: ResMut<PlayerVitals>,
    mut current_room: ResMut<CurrentRoom>,
    mut transition: ResMut<RoomTransitionState>,
) {
    *player_vitals = PlayerVitals::default();
    current_room.id = RoomId::OverworldCenter;
    transition.locked = false;
    transition.direction = None;
    transition.timer.reset();
}

fn spawn_sword_attack(
    mut commands: Commands,
    actions: Res<InputActions>,
    transition: Res<RoomTransitionState>,
    player: Query<(&Transform, &Facing), With<Player>>,
    attacks: Query<Entity, With<SwordAttack>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !actions.attack || transition.locked || !attacks.is_empty() {
        return;
    }

    let Ok((player_transform, facing)) = player.single() else {
        return;
    };

    let (offset, size) = match facing {
        Facing::Up => (Vec2::new(0.0, 14.0), Vec2::new(10.0, 18.0)),
        Facing::Down => (Vec2::new(0.0, -14.0), Vec2::new(10.0, 18.0)),
        Facing::Left => (Vec2::new(-14.0, 0.0), Vec2::new(18.0, 10.0)),
        Facing::Right => (Vec2::new(14.0, 0.0), Vec2::new(18.0, 10.0)),
    };

    commands.spawn((
        Name::new("SwordAttack"),
        RoomEntity,
        SwordAttack,
        Damage(SWORD_DAMAGE),
        Hitbox {
            half_size: size * 0.5,
        },
        Lifetime(Timer::from_seconds(SWORD_LIFETIME_SECS, TimerMode::Once)),
        rectangle_mesh(meshes.as_mut(), size),
        color_material(materials.as_mut(), WorldColor::Accent),
        Transform::from_xyz(
            player_transform.translation.x + offset.x,
            player_transform.translation.y + offset.y,
            constants::render_layers::PROJECTILES,
        ),
    ));
}

fn tick_lifetimes(
    mut commands: Commands,
    time: Res<Time>,
    mut entities: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut entities {
        lifetime.tick(time.delta());
        if lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn resolve_sword_hits(
    mut commands: Commands,
    attacks: Query<(Entity, &Transform, &Hitbox, &Damage), With<SwordAttack>>,
    mut enemies: Query<(Entity, &Transform, &Hurtbox, &mut Health), (With<Enemy>, Without<SwordAttack>)>,
) {
    for (attack_entity, attack_transform, attack_hitbox, attack_damage) in &attacks {
        let attack_pos = attack_transform.translation.truncate();
        let mut hit_anything = false;

        for (enemy_entity, enemy_transform, enemy_hurtbox, mut enemy_health) in &mut enemies {
            let enemy_pos = enemy_transform.translation.truncate();
            if !aabb_overlap(
                attack_pos,
                attack_hitbox.half_size,
                enemy_pos,
                enemy_hurtbox.half_size,
            ) {
                continue;
            }

            enemy_health.current = enemy_health.current.saturating_sub(attack_damage.0);
            if enemy_health.current == 0 {
                commands.entity(enemy_entity).despawn();
            }
            hit_anything = true;
            break;
        }

        if hit_anything {
            commands.entity(attack_entity).despawn();
        }
    }
}

fn tick_player_invulnerability(
    time: Res<Time>,
    mut players: Query<&mut InvulnerabilityTimer, With<Player>>,
) {
    let Ok(mut timer) = players.single_mut() else {
        return;
    };

    if !timer.is_finished() {
        timer.tick(time.delta());
    }
}

fn resolve_player_enemy_damage(
    mut players: Query<
        (
            &Transform,
            &Hurtbox,
            &mut Health,
            &mut Knockback,
            &mut InvulnerabilityTimer,
        ),
        With<Player>,
    >,
    enemies: Query<(&Transform, &Hitbox, &Damage), (With<Enemy>, Without<Player>)>,
    mut player_vitals: ResMut<PlayerVitals>,
) {
    let Ok((player_transform, player_hurtbox, mut player_health, mut knockback, mut invulnerability)) =
        players.single_mut()
    else {
        return;
    };

    if !invulnerability.is_finished() {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (enemy_transform, enemy_hitbox, enemy_damage) in &enemies {
        let enemy_pos = enemy_transform.translation.truncate();
        if !aabb_overlap(
            player_pos,
            player_hurtbox.half_size,
            enemy_pos,
            enemy_hitbox.half_size,
        ) {
            continue;
        }

        player_health.current = player_health.current.saturating_sub(enemy_damage.0);
        player_vitals.current_health = player_health.current;

        *invulnerability = InvulnerabilityTimer(Timer::from_seconds(
            PLAYER_INVULNERABILITY_SECS,
            TimerMode::Once,
        ));

        let knockback_direction = (player_pos - enemy_pos)
            .try_normalize()
            .unwrap_or(Vec2::Y);
        knockback.velocity = knockback_direction * PLAYER_KNOCKBACK_SPEED;
        break;
    }
}

fn handle_player_death(
    player: Query<&Health, With<Player>>,
    mut player_vitals: ResMut<PlayerVitals>,
    mut current_room: ResMut<CurrentRoom>,
    mut transition: ResMut<RoomTransitionState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(player_health) = player.single() else {
        return;
    };

    if player_health.current > 0 {
        return;
    }

    player_vitals.current_health = player_vitals.continue_health();
    current_room.id = RoomId::OverworldCenter;
    transition.locked = false;
    transition.direction = None;
    transition.timer.reset();
    next_state.set(AppState::GameOver);
}

fn aabb_overlap(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    (a_pos.x - b_pos.x).abs() < (a_half.x + b_half.x)
        && (a_pos.y - b_pos.y).abs() < (a_half.y + b_half.y)
}
