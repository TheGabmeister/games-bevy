use bevy::prelude::*;

use crate::game::{
    components::{
        Ghost, GhostAppearance, GridMover, LivesText, MessageText, PacmanMouth, Pellet, PelletKind,
        Player, ScoreText,
    },
    resources::{GameMaterials, GameSession, RoundState},
};

pub fn sync_ghost_appearance(
    session: Res<GameSession>,
    materials: Res<GameMaterials>,
    mut ghosts: Query<(&Ghost, &GhostAppearance, &mut MeshMaterial2d<ColorMaterial>)>,
) {
    for (ghost, appearance, mut material) in &mut ghosts {
        let next_material = if ghost.returning_home {
            materials.hidden_ghost.clone()
        } else if session.frightened_active() {
            materials.frightened_ghost.clone()
        } else {
            appearance.normal_material.clone()
        };

        *material = MeshMaterial2d(next_material);
    }
}

pub fn animate_pacman(
    time: Res<Time>,
    round_state: Res<State<RoundState>>,
    player: Single<&GridMover, With<Player>>,
    mut mouth: Single<&mut Transform, With<PacmanMouth>>,
) {
    let moving = *round_state.get() == RoundState::Playing && player.current.is_some();
    let openness = if moving {
        0.15 + time.elapsed_secs().sin().abs() * 0.95
    } else {
        0.12
    };

    mouth.scale.y = openness;
}

pub fn animate_power_pellets(time: Res<Time>, mut pellets: Query<(&Pellet, &mut Transform)>) {
    let pulse = 0.85 + time.elapsed_secs().sin().abs() * 0.4;

    for (pellet, mut transform) in &mut pellets {
        if pellet.kind == PelletKind::Power {
            transform.scale = Vec3::splat(pulse);
        }
    }
}

pub fn update_hud(
    session: Res<GameSession>,
    round_state: Res<State<RoundState>>,
    mut score_text: Single<&mut Text, With<ScoreText>>,
    mut lives_text: Single<&mut Text, With<LivesText>>,
    mut message_text: Single<&mut Text, With<MessageText>>,
) {
    score_text.0 = format!("SCORE {:05}", session.score);
    lives_text.0 = format!("LIVES {}", session.lives);
    message_text.0 = match round_state.get() {
        RoundState::Ready => "READY!".into(),
        RoundState::Playing => String::new(),
        RoundState::Won => "YOU WIN  PRESS SPACE".into(),
        RoundState::GameOver => "GAME OVER  PRESS SPACE".into(),
    };
}
