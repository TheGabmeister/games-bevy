use bevy::{color::palettes::css, prelude::*};

use super::gameplay::{
    DespawnOnStateExit, GameConfig, Invader, InvaderRow, Player, Projectile, ProjectileOwner,
    ScreenState, SessionState, ShieldCell, Ufo,
};

pub struct SpaceInvadersPresentationPlugin;

impl Plugin for SpaceInvadersPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ScreenState::Title),
            (spawn_playfield_frame, spawn_title_overlay),
        )
        .add_systems(
            OnEnter(ScreenState::Playing),
            (spawn_playfield_frame, spawn_hud),
        )
        .add_systems(
            OnEnter(ScreenState::WaveTransition),
            (spawn_playfield_frame, spawn_wave_overlay),
        )
        .add_systems(
            OnEnter(ScreenState::GameOver),
            (spawn_playfield_frame, spawn_game_over_overlay),
        )
        .add_systems(
            Update,
            (
                build_player_visuals,
                build_invader_visuals,
                build_projectile_visuals,
                build_shield_visuals,
                build_ufo_visuals,
                update_hud,
            ),
        );
    }
}

#[derive(Component)]
struct HudScore;

#[derive(Component)]
struct HudLives;

#[derive(Component)]
struct HudWave;

fn spawn_playfield_frame(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let left = config.playfield_left();
    let right = config.playfield_right();
    let top = config.playfield_top();
    let bottom = config.playfield_bottom();
    let border = 3.0;
    let frame_color = neon_cyan();
    let invasion_color = neon_magenta();

    commands
        .spawn((Transform::default(), DespawnOnStateExit))
        .with_children(|parent| {
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(border, config.playfield_size.y),
                frame_color,
                Vec3::new(left, 0.0, -10.0),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(border, config.playfield_size.y),
                frame_color,
                Vec3::new(right, 0.0, -10.0),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(config.playfield_size.x, border),
                frame_color,
                Vec3::new(0.0, top, -10.0),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(config.playfield_size.x, border),
                Color::srgba(0.08, 0.7, 0.65, 0.35),
                Vec3::new(0.0, bottom, -10.0),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(config.playfield_size.x - 34.0, 2.0),
                invasion_color,
                Vec3::new(0.0, config.invasion_line_y, -10.0),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(config.playfield_size.x - 40.0, 4.0),
                Color::srgba(0.1, 0.85, 0.9, 0.45),
                Vec3::new(0.0, top - 54.0, -10.0),
            ));
        });
}

fn spawn_title_overlay(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(16),
                ..default()
            },
            DespawnOnStateExit,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SPACE INVADERS"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(neon_cyan()),
            ));
            parent.spawn((
                Text::new(
                    "Cabinet classic rebuilt in Bevy\nA / Left and D / Right to move\nSpace to fire\nPress Enter or Space to start",
                ),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(soft_white()),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                left: px(24),
                right: px(24),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnStateExit,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SCORE 0000"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(neon_cyan()),
                HudScore,
            ));
            parent.spawn((
                Text::new("LIVES 3"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(neon_green()),
                HudLives,
            ));
            parent.spawn((
                Text::new("WAVE 01"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(neon_magenta()),
                HudWave,
            ));
        });
}

fn spawn_wave_overlay(mut commands: Commands, session: Res<SessionState>) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnStateExit,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("WAVE {:02}\nINCOMING", session.wave)),
                TextFont {
                    font_size: 44.0,
                    ..default()
                },
                TextColor(neon_magenta()),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn spawn_game_over_overlay(mut commands: Commands, session: Res<SessionState>) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(14),
                ..default()
            },
            DespawnOnStateExit,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 54.0,
                    ..default()
                },
                TextColor(neon_magenta()),
            ));
            parent.spawn((
                Text::new(format!(
                    "Final Score {:04}\nPress Enter or Space to restart",
                    session.score
                )),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(soft_white()),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn build_player_visuals(
    mut commands: Commands,
    players: Query<Entity, Added<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in &players {
        commands.entity(entity).with_children(|parent| {
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(18.0, 10.0),
                neon_cyan(),
                Vec3::new(0.0, 9.0, 0.1),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(36.0, 10.0),
                neon_green(),
                Vec3::new(0.0, 1.0, 0.1),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(52.0, 6.0),
                neon_green(),
                Vec3::new(0.0, -8.0, 0.1),
            ));
        });
    }
}

fn build_invader_visuals(
    mut commands: Commands,
    invaders: Query<(Entity, &Invader), Added<Invader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, invader) in &invaders {
        let color = invader_color(invader.row_kind);
        commands
            .entity(entity)
            .with_children(|parent| match invader.row_kind {
                InvaderRow::Commander => {
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(6.0, 4.0),
                        color,
                        Vec3::new(-10.0, 12.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(6.0, 4.0),
                        color,
                        Vec3::new(10.0, 12.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(28.0, 8.0),
                        color,
                        Vec3::new(0.0, 6.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(36.0, 10.0),
                        color,
                        Vec3::new(0.0, -2.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(8.0, 6.0),
                        color,
                        Vec3::new(-10.0, -11.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(8.0, 6.0),
                        color,
                        Vec3::new(10.0, -11.0, 0.1),
                    ));
                }
                InvaderRow::Guard => {
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(34.0, 12.0),
                        color,
                        Vec3::new(0.0, 2.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(10.0, 6.0),
                        color,
                        Vec3::new(-16.0, 0.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(10.0, 6.0),
                        color,
                        Vec3::new(16.0, 0.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(8.0, 6.0),
                        color,
                        Vec3::new(-8.0, -11.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(8.0, 6.0),
                        color,
                        Vec3::new(8.0, -11.0, 0.1),
                    ));
                }
                InvaderRow::Drone => {
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(32.0, 14.0),
                        color,
                        Vec3::new(0.0, 0.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(18.0, 4.0),
                        neon_cyan(),
                        Vec3::new(0.0, 5.0, 0.2),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(8.0, 7.0),
                        color,
                        Vec3::new(-14.0, -8.0, 0.1),
                    ));
                    parent.spawn(rectangle_bundle(
                        &mut meshes,
                        &mut materials,
                        Vec2::new(8.0, 7.0),
                        color,
                        Vec3::new(14.0, -8.0, 0.1),
                    ));
                }
            });
    }
}

fn build_projectile_visuals(
    mut commands: Commands,
    projectiles: Query<(Entity, &Projectile), Added<Projectile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, projectile) in &projectiles {
        let (size, color) = match projectile.owner {
            ProjectileOwner::Player => (Vec2::new(6.0, 18.0), neon_magenta()),
            ProjectileOwner::Invader => (Vec2::new(8.0, 18.0), neon_orange()),
        };

        commands.entity(entity).insert((
            Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
            MeshMaterial2d(materials.add(color)),
        ));
    }
}

fn build_shield_visuals(
    mut commands: Commands,
    shields: Query<Entity, Added<ShieldCell>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in &shields {
        commands.entity(entity).insert((
            Mesh2d(meshes.add(Rectangle::new(11.0, 11.0))),
            MeshMaterial2d(materials.add(Color::srgba(0.1, 0.95, 0.75, 0.92))),
        ));
    }
}

fn build_ufo_visuals(
    mut commands: Commands,
    ufos: Query<Entity, Added<Ufo>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for entity in &ufos {
        commands.entity(entity).with_children(|parent| {
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(68.0, 16.0),
                neon_orange(),
                Vec3::new(0.0, -2.0, 0.1),
            ));
            parent.spawn(rectangle_bundle(
                &mut meshes,
                &mut materials,
                Vec2::new(30.0, 8.0),
                neon_magenta(),
                Vec3::new(0.0, 10.0, 0.2),
            ));
            for x in [-18.0, 0.0, 18.0] {
                parent.spawn(rectangle_bundle(
                    &mut meshes,
                    &mut materials,
                    Vec2::new(6.0, 4.0),
                    soft_white(),
                    Vec3::new(x, 0.0, 0.3),
                ));
            }
        });
    }
}

fn update_hud(
    session: Res<SessionState>,
    mut score_text: Query<&mut Text, With<HudScore>>,
    mut lives_text: Query<&mut Text, With<HudLives>>,
    mut wave_text: Query<&mut Text, With<HudWave>>,
) {
    if !session.is_changed() {
        return;
    }

    if let Ok(mut score) = score_text.single_mut() {
        **score = format!("SCORE {:04}", session.score);
    }

    if let Ok(mut lives) = lives_text.single_mut() {
        **lives = format!("LIVES {}", session.lives);
    }

    if let Ok(mut wave) = wave_text.single_mut() {
        **wave = format!("WAVE {:02}", session.wave);
    }
}

fn rectangle_bundle(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    size: Vec2,
    color: Color,
    translation: Vec3,
) -> (Mesh2d, MeshMaterial2d<ColorMaterial>, Transform) {
    (
        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_translation(translation),
    )
}

fn invader_color(row: InvaderRow) -> Color {
    match row {
        InvaderRow::Commander => neon_orange(),
        InvaderRow::Guard => neon_cyan(),
        InvaderRow::Drone => neon_green(),
    }
}

fn neon_cyan() -> Color {
    Color::from(css::AQUA)
}

fn neon_green() -> Color {
    Color::from(css::GREEN_YELLOW)
}

fn neon_magenta() -> Color {
    Color::from(css::DEEP_PINK)
}

fn neon_orange() -> Color {
    Color::from(css::ORANGE_RED)
}

fn soft_white() -> Color {
    Color::from(css::WHITE_SMOKE)
}
