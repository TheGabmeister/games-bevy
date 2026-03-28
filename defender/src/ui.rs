use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::scheduling::GameplaySet;
use crate::states::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(
                OnEnter(AppState::StartScreen),
                (reset_game, start_screen_setup).chain(),
            )
            .add_systems(OnExit(AppState::StartScreen), start_screen_cleanup)
            .add_systems(
                Update,
                start_screen_input.run_if(in_state(AppState::StartScreen)),
            )
            .add_systems(Update, ui_update.in_set(GameplaySet::Post))
            .add_systems(OnEnter(AppState::PlayerDeath), player_death_setup)
            .add_systems(
                Update,
                player_death_timer.run_if(in_state(AppState::PlayerDeath)),
            )
            .add_systems(OnEnter(AppState::GameOver), game_over_setup)
            .add_systems(OnExit(AppState::GameOver), game_over_cleanup)
            .add_systems(Update, game_over_input.run_if(in_state(AppState::GameOver)));
    }
}

pub fn setup_ui(mut commands: Commands) {
    // Score text
    commands.spawn((
        ScoreText,
        Text::new("SCORE: 0"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    // Lives text
    commands.spawn((
        LivesText,
        Text::new("LIVES: 3"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.5, 1.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    // Smart bombs text
    commands.spawn((
        SmartBombText,
        Text::new("BOMBS: 3"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(35.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));
}

pub fn ui_update(
    game_state: Res<GameState>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<LivesText>, Without<SmartBombText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<SmartBombText>)>,
    mut bombs_q: Query<&mut Text, (With<SmartBombText>, Without<ScoreText>, Without<LivesText>)>,
) {
    if !game_state.is_changed() {
        return;
    }

    if let Ok(mut text) = score_q.single_mut() {
        *text = Text::new(format!("SCORE: {}", game_state.score));
    }
    if let Ok(mut text) = lives_q.single_mut() {
        *text = Text::new(format!("LIVES: {}", game_state.lives));
    }
    if let Ok(mut text) = bombs_q.single_mut() {
        *text = Text::new(format!("BOMBS: {}", game_state.smart_bombs));
    }
}

pub fn game_over_setup(mut commands: Commands) {
    commands.spawn((
        GameOverScreen,
        Text::new("GAME OVER"),
        TextFont {
            font_size: 72.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.2, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(30.0),
            ..default()
        },
    ));

    commands.spawn((
        GameOverScreen,
        Text::new("Press SPACE to restart"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(32.0),
            ..default()
        },
    ));
}

pub fn game_over_cleanup(mut commands: Commands, screens: Query<Entity, With<GameOverScreen>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::StartScreen);
    }
}

pub fn start_screen_setup(mut commands: Commands) {
    commands.spawn((
        StartScreen,
        Text::new("DEFENDER"),
        TextFont {
            font_size: 80.0,
            ..default()
        },
        TextColor(COLOR_PLAYER),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(25.0),
            left: Val::Percent(30.0),
            ..default()
        },
    ));

    commands.spawn((
        StartScreen,
        Text::new("Press SPACE to start"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(45.0),
            left: Val::Percent(33.0),
            ..default()
        },
    ));

    commands.spawn((
        StartScreen,
        Text::new(
            "ARROWS/WASD: Move   SHIFT: Reverse   SPACE: Fire\nE: Smart Bomb   H: Hyperspace",
        ),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(60.0),
            left: Val::Percent(22.0),
            ..default()
        },
    ));
}

pub fn start_screen_cleanup(mut commands: Commands, screens: Query<Entity, With<StartScreen>>) {
    for entity in &screens {
        commands.entity(entity).despawn();
    }
}

pub fn start_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::WaveIntro);
    }
}

pub fn player_death_setup(
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_q: Query<(&WorldPosition, &Transform), With<Player>>,
) {
    if let Ok((wp, tf)) = player_q.single() {
        crate::spawning::spawn_explosion(
            &mut commands,
            &assets,
            &mut materials,
            wp.0,
            tf.translation.y,
            COLOR_PLAYER,
        );
    }

    commands.insert_resource(DeathTimer(Timer::from_seconds(1.5, TimerMode::Once)));
}

pub fn player_death_timer(
    time: Res<Time>,
    mut timer: ResMut<DeathTimer>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
    player_q: Query<Entity, With<Player>>,
) {
    timer.0.tick(time.delta());
    if timer.0.is_finished() {
        // Despawn player
        for entity in &player_q {
            commands.entity(entity).despawn();
        }

        if game_state.lives > 0 {
            game_state.lives -= 1;
            next_state.set(AppState::WaveIntro);
        } else {
            next_state.set(AppState::GameOver);
        }
    }
}

pub fn respawn_player(
    mut commands: Commands,
    assets: Res<GameplayAssets>,
    player_q: Query<Entity, With<Player>>,
    mut cam_pos: ResMut<CameraWorldPos>,
) {
    // Only spawn if no player exists
    if player_q.iter().next().is_none() {
        let spawn_x = WORLD_WIDTH / 2.0;
        crate::spawning::spawn_player(&mut commands, &assets, spawn_x);
        cam_pos.0 = spawn_x;
    }
}

pub fn reset_game(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut wave_state: ResMut<WaveState>,
    mut rng: ResMut<GameRng>,
    // Despawn everything
    enemies: Query<Entity, With<Enemy>>,
    humans: Query<Entity, With<Human>>,
    projectiles: Query<Entity, With<PlayerProjectile>>,
    enemy_projectiles: Query<Entity, (With<EnemyProjectile>, Without<PlayerProjectile>)>,
    mines: Query<
        Entity,
        (
            With<Mine>,
            Without<PlayerProjectile>,
            Without<EnemyProjectile>,
        ),
    >,
    player: Query<Entity, With<Player>>,
    explosions: Query<Entity, With<Explosion>>,
    scanner_dots: Query<Entity, With<ScannerDot>>,
) {
    for e in enemies
        .iter()
        .chain(humans.iter())
        .chain(projectiles.iter())
        .chain(enemy_projectiles.iter())
        .chain(mines.iter())
        .chain(player.iter())
        .chain(explosions.iter())
        .chain(scanner_dots.iter())
    {
        commands.entity(e).despawn();
    }

    *game_state = GameState::default();
    *wave_state = WaveState::default();
    rng.reset();
}
