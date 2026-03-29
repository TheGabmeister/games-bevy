use bevy::prelude::*;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;
use crate::states::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Start Screen
            .add_systems(
                OnEnter(AppState::StartScreen),
                (reset_title_state, spawn_start_screen).chain(),
            )
            .add_systems(OnExit(AppState::StartScreen), cleanup::<StartScreenUI>)
            .add_systems(Update, start_input.run_if(in_state(AppState::StartScreen)))
            // Playing HUD (spawned from the level lifecycle to keep state entry ordered)
            .add_systems(
                Update,
                update_hud.run_if(
                    in_state(AppState::Playing)
                        .or(in_state(AppState::WaveTally))
                        .or(in_state(AppState::Dying)),
                ),
            )
            // Dying
            .add_systems(OnEnter(AppState::Dying), spawn_death_overlay)
            .add_systems(Update, death_sequence_system.run_if(in_state(AppState::Dying)))
            .add_systems(OnExit(AppState::Dying), cleanup::<DeathOverlayUI>)
            // Wave Tally
            .add_systems(OnEnter(AppState::WaveTally), spawn_wave_tally)
            .add_systems(Update, wave_tally_system.run_if(in_state(AppState::WaveTally)))
            .add_systems(OnExit(AppState::WaveTally), cleanup::<WaveTallyUI>)
            // Game Over
            .add_systems(OnEnter(AppState::GameOver), (cleanup_stage, spawn_game_over).chain())
            .add_systems(OnExit(AppState::GameOver), cleanup::<GameOverUI>)
            .add_systems(Update, restart_input.run_if(in_state(AppState::GameOver)))
            // Win Screen
            .add_systems(OnEnter(AppState::WinScreen), (cleanup_stage, spawn_win_screen).chain())
            .add_systems(OnExit(AppState::WinScreen), cleanup::<WinScreenUI>)
            .add_systems(Update, restart_input.run_if(in_state(AppState::WinScreen)));
    }
}

// --- Generic Cleanup ---

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_stage(
    mut commands: Commands,
    stage_q: Query<Entity, With<StageEntity>>,
    attempt_q: Query<Entity, With<AttemptEntity>>,
    hud_q: Query<Entity, With<GameHudUI>>,
) {
    for entity in stage_q.iter().chain(attempt_q.iter()).chain(hud_q.iter()) {
        commands.entity(entity).despawn();
    }
}

// --- Start Screen ---

fn reset_title_state(
    mut commands: Commands,
    mut run_data: ResMut<RunData>,
    mut wave_rt: ResMut<WaveRuntime>,
) {
    *run_data = RunData::default();
    *wave_rt = WaveRuntime::default();
    commands.insert_resource(WaveConfig::from_wave(1));
}

fn spawn_start_screen(mut commands: Commands, session: Res<SessionData>) {
    commands
        .spawn((
            StartScreenUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("DONKEY KONG"),
                TextFont { font_size: FONT_SIZE_TITLE, ..default() },
                TextColor(TEXT_COLOR),
            ));
            if session.high_score > 0 {
                parent.spawn((
                    Text::new(format!("HIGH SCORE: {}", session.high_score)),
                    TextFont { font_size: FONT_SIZE_BODY, ..default() },
                    TextColor(TEXT_COLOR),
                ));
            }
            parent.spawn((
                Text::new("Press SPACE to Play"),
                TextFont { font_size: FONT_SIZE_BODY, ..default() },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn start_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut run_data: ResMut<RunData>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        run_data.next_entry = PlayingEntry::NewRun;
        next_state.set(AppState::Playing);
    }
}

// --- Game HUD ---

pub fn spawn_hud(mut commands: Commands, existing: Query<Entity, With<GameHudUI>>) {
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            GameHudUI,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ScoreText,
                Text::new("0"),
                TextFont { font_size: FONT_SIZE_HUD, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                HighScoreText,
                Text::new("HI: 0"),
                TextFont { font_size: FONT_SIZE_HUD, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                LivesText,
                Text::new("L:3"),
                TextFont { font_size: FONT_SIZE_HUD, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                WaveText,
                Text::new("W:1"),
                TextFont { font_size: FONT_SIZE_HUD, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                BonusTimerText,
                Text::new("5000"),
                TextFont { font_size: FONT_SIZE_HUD, ..default() },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn update_hud(
    run_data: Res<RunData>,
    session: Res<SessionData>,
    wave_rt: Res<WaveRuntime>,
    mut score_q: Query<&mut Text, (With<ScoreText>, Without<HighScoreText>, Without<LivesText>, Without<WaveText>, Without<BonusTimerText>)>,
    mut hi_q: Query<&mut Text, (With<HighScoreText>, Without<ScoreText>, Without<LivesText>, Without<WaveText>, Without<BonusTimerText>)>,
    mut lives_q: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<HighScoreText>, Without<WaveText>, Without<BonusTimerText>)>,
    mut wave_q: Query<&mut Text, (With<WaveText>, Without<ScoreText>, Without<HighScoreText>, Without<LivesText>, Without<BonusTimerText>)>,
    mut bonus_q: Query<&mut Text, (With<BonusTimerText>, Without<ScoreText>, Without<HighScoreText>, Without<LivesText>, Without<WaveText>)>,
) {
    if let Ok(mut t) = score_q.single_mut() {
        **t = format!("{}", run_data.score);
    }
    if let Ok(mut t) = hi_q.single_mut() {
        let hi = run_data.score.max(session.high_score);
        **t = format!("HI:{}", hi);
    }
    if let Ok(mut t) = lives_q.single_mut() {
        **t = format!("L:{}", run_data.lives);
    }
    if let Ok(mut t) = wave_q.single_mut() {
        **t = format!("W:{}", run_data.wave);
    }
    if let Ok(mut t) = bonus_q.single_mut() {
        **t = format!("{}", wave_rt.bonus_timer);
    }
}

// --- Death Sequence ---

fn spawn_death_overlay(mut commands: Commands, death: Res<DeathSequence>) {
    commands
        .spawn((
            DeathOverlayUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(death_message(death.cause)),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn death_message(cause: DeathCause) -> &'static str {
    match cause {
        DeathCause::Barrel => "Barrel hit!",
        DeathCause::Fireball => "Fireball hit!",
        DeathCause::Fall => "Bad fall!",
        DeathCause::Timer => "Time up!",
    }
}

fn death_sequence_system(
    time: Res<Time>,
    mut death: ResMut<DeathSequence>,
    mut run_data: ResMut<RunData>,
    mut session: ResMut<SessionData>,
    mut next_state: ResMut<NextState<AppState>>,
    mut player_q: Query<&mut Visibility, With<Player>>,
) {
    death.elapsed += time.delta_secs();
    let total = DEATH_FLASH_DURATION + DEATH_HOLD_DURATION;

    // Flash player during first phase
    if death.elapsed < DEATH_FLASH_DURATION {
        if let Ok(mut vis) = player_q.single_mut() {
            let flash = (death.elapsed * 10.0).sin() > 0.0;
            *vis = if flash { Visibility::Visible } else { Visibility::Hidden };
        }
    } else if let Ok(mut vis) = player_q.single_mut() {
        *vis = Visibility::Hidden;
    }

    if death.elapsed >= total {
        // Restore visibility
        if let Ok(mut vis) = player_q.single_mut() {
            *vis = Visibility::Visible;
        }

        run_data.lives = run_data.lives.saturating_sub(1);
        session.high_score = session.high_score.max(run_data.score);

        if run_data.lives > 0 {
            run_data.next_entry = PlayingEntry::RetryAfterDeath;
            next_state.set(AppState::Playing);
        } else {
            next_state.set(AppState::GameOver);
        }
    }
}

// --- Wave Tally ---

fn spawn_wave_tally(mut commands: Commands) {
    commands.spawn((
        WaveTallyUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("WAVE CLEAR!"),
            TextFont { font_size: FONT_SIZE_TITLE, ..default() },
            TextColor(TEXT_COLOR),
        ));
    });
}

fn wave_tally_system(
    time: Res<Time>,
    mut run_data: ResMut<RunData>,
    mut wave_rt: ResMut<WaveRuntime>,
    mut session: ResMut<SessionData>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if wave_rt.bonus_timer > 0 {
        let drain = (2000.0 * time.delta_secs()) as i32;
        let actual = drain.min(wave_rt.bonus_timer);
        wave_rt.bonus_timer -= actual;
        run_data.score += actual as u32;
    } else {
        session.high_score = session.high_score.max(run_data.score);
        if run_data.wave >= 5 {
            next_state.set(AppState::WinScreen);
        } else {
            run_data.wave += 1;
            run_data.next_entry = PlayingEntry::NextWave;
            next_state.set(AppState::Playing);
        }
    }
}

// --- Game Over ---

fn spawn_game_over(mut commands: Commands, run_data: Res<RunData>, session: Res<SessionData>) {
    commands
        .spawn((
            GameOverUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont { font_size: FONT_SIZE_TITLE, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new(format!("Score: {}", run_data.score)),
                TextFont { font_size: FONT_SIZE_BODY, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new(format!("High Score: {}", session.high_score)),
                TextFont { font_size: FONT_SIZE_BODY, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new("Press SPACE to Restart"),
                TextFont { font_size: FONT_SIZE_BODY, ..default() },
                TextColor(TEXT_COLOR),
            ));
        });
}

// --- Win Screen ---

fn spawn_win_screen(mut commands: Commands, run_data: Res<RunData>, session: Res<SessionData>) {
    commands
        .spawn((
            WinScreenUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("YOU WIN!"),
                TextFont { font_size: FONT_SIZE_TITLE, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", run_data.score)),
                TextFont { font_size: FONT_SIZE_BODY, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new(format!("High Score: {}", session.high_score)),
                TextFont { font_size: FONT_SIZE_BODY, ..default() },
                TextColor(TEXT_COLOR),
            ));
            parent.spawn((
                Text::new("Press SPACE to Continue"),
                TextFont { font_size: FONT_SIZE_BODY, ..default() },
                TextColor(TEXT_COLOR),
            ));
        });
}

fn restart_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(AppState::StartScreen);
    }
}
