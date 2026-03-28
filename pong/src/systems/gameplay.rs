use bevy::prelude::*;

use crate::{
    components::{
        Ball, Collider, GameplayEntity, MatchConfig, Paddle, PaddleHitEvent, PlayerControl,
        PlayerSide, Score, ScoreText, Velocity, Winner,
    },
    state::Phase,
};

const ARENA_HALF_WIDTH: f32 = 450.0;
const ARENA_HALF_HEIGHT: f32 = 250.0;
const PADDLE_WIDTH: f32 = 18.0;
const PADDLE_HEIGHT: f32 = 110.0;
const PADDLE_X_OFFSET: f32 = ARENA_HALF_WIDTH - 36.0;
const BALL_RADIUS: f32 = 10.0;
const CENTER_LINE_WIDTH: f32 = 6.0;

#[derive(Resource)]
pub(crate) struct GameplayRenderAssets {
    paddle_mesh: Handle<Mesh>,
    ball_mesh: Handle<Mesh>,
    center_line_mesh: Handle<Mesh>,
    white_material: Handle<ColorMaterial>,
    gray_material: Handle<ColorMaterial>,
}

type BallMovementQuery<'w, 's> =
    Query<'w, 's, (&'static mut Transform, &'static Velocity), With<Ball>>;
type BallCollisionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Velocity,
        &'static Collider,
    ),
    (With<Ball>, Without<Paddle>),
>;
type BallStateQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut Velocity,
        &'static Collider,
    ),
    With<Ball>,
>;
type PaddleCollisionQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Collider, &'static Paddle), Without<Ball>>;

pub fn setup_gameplay_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(GameplayRenderAssets {
        paddle_mesh: meshes.add(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
        ball_mesh: meshes.add(Circle::new(BALL_RADIUS)),
        center_line_mesh: meshes.add(Rectangle::new(CENTER_LINE_WIDTH, ARENA_HALF_HEIGHT * 2.0)),
        white_material: materials.add(Color::WHITE),
        gray_material: materials.add(Color::srgb(0.35, 0.35, 0.35)),
    });
}

pub fn spawn_gameplay(
    mut commands: Commands,
    render_assets: Res<GameplayRenderAssets>,
    config: Res<MatchConfig>,
    score: Res<Score>,
) {
    commands.spawn((
        GameplayEntity,
        Mesh2d(render_assets.center_line_mesh.clone()),
        MeshMaterial2d(render_assets.gray_material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands.spawn((
        GameplayEntity,
        Paddle {
            side: PlayerSide::Left,
        },
        PlayerControl {
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
        },
        Collider {
            half_size: Vec2::new(PADDLE_WIDTH * 0.5, PADDLE_HEIGHT * 0.5),
        },
        Mesh2d(render_assets.paddle_mesh.clone()),
        MeshMaterial2d(render_assets.white_material.clone()),
        Transform::from_xyz(-PADDLE_X_OFFSET, 0.0, 1.0),
    ));

    commands.spawn((
        GameplayEntity,
        Paddle {
            side: PlayerSide::Right,
        },
        PlayerControl {
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
        },
        Collider {
            half_size: Vec2::new(PADDLE_WIDTH * 0.5, PADDLE_HEIGHT * 0.5),
        },
        Mesh2d(render_assets.paddle_mesh.clone()),
        MeshMaterial2d(render_assets.white_material.clone()),
        Transform::from_xyz(PADDLE_X_OFFSET, 0.0, 1.0),
    ));

    let (ball_position, ball_velocity) = reset_ball_state(*config, PlayerSide::Left);
    commands.spawn((
        GameplayEntity,
        Ball,
        Velocity(ball_velocity),
        Collider {
            half_size: Vec2::splat(BALL_RADIUS),
        },
        Mesh2d(render_assets.ball_mesh.clone()),
        MeshMaterial2d(render_assets.white_material.clone()),
        Transform::from_translation(ball_position),
    ));

    commands.spawn((
        GameplayEntity,
        ScoreText,
        Text2d::new(score.formatted()),
        TextFont {
            font_size: 52.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, ARENA_HALF_HEIGHT - 14.0, 10.0),
    ));
}

pub fn move_paddles(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    config: Res<MatchConfig>,
    mut paddles: Query<(&mut Transform, &Collider, &PlayerControl), With<Paddle>>,
) {
    for (mut transform, collider, control) in &mut paddles {
        let mut axis = 0.0;
        if input.pressed(control.up) {
            axis += 1.0;
        }
        if input.pressed(control.down) {
            axis -= 1.0;
        }

        transform.translation.y += axis * config.paddle_speed * time.delta_secs();

        let max_y = ARENA_HALF_HEIGHT - collider.half_size.y;
        transform.translation.y = transform.translation.y.clamp(-max_y, max_y);
    }
}

pub fn move_ball(time: Res<Time>, mut ball_query: BallMovementQuery) {
    let Ok((mut transform, velocity)) = ball_query.single_mut() else {
        return;
    };

    transform.translation += velocity.0.extend(0.0) * time.delta_secs();
}

pub fn bounce_from_bounds(mut ball_query: BallStateQuery) {
    let Ok((mut transform, mut velocity, collider)) = ball_query.single_mut() else {
        return;
    };

    let top_limit = ARENA_HALF_HEIGHT - collider.half_size.y;
    let bottom_limit = -ARENA_HALF_HEIGHT + collider.half_size.y;

    if transform.translation.y >= top_limit && velocity.y > 0.0 {
        transform.translation.y = top_limit;
        velocity.y = -velocity.y;
    } else if transform.translation.y <= bottom_limit && velocity.y < 0.0 {
        transform.translation.y = bottom_limit;
        velocity.y = -velocity.y;
    }
}

pub fn bounce_from_paddles(
    mut commands: Commands,
    config: Res<MatchConfig>,
    mut ball_query: BallCollisionQuery,
    paddles: PaddleCollisionQuery,
) {
    let Ok((mut ball_transform, mut ball_velocity, ball_collider)) = ball_query.single_mut() else {
        return;
    };

    let ball_position = ball_transform.translation.truncate();

    for (paddle_transform, paddle_collider, paddle) in &paddles {
        let paddle_position = paddle_transform.translation.truncate();
        if !intersects_aabb(
            ball_position,
            ball_collider.half_size,
            paddle_position,
            paddle_collider.half_size,
        ) {
            continue;
        }

        let current_speed = ball_velocity.length();
        ball_velocity.0 = compute_paddle_bounce(
            ball_position.y,
            paddle_position.y,
            paddle_collider.half_size.y,
            current_speed,
            paddle.side,
            config.speed_gain_per_hit,
        );

        let offset_sign = match paddle.side {
            PlayerSide::Left => 1.0,
            PlayerSide::Right => -1.0,
        };
        ball_transform.translation.x = paddle_position.x
            + offset_sign * (paddle_collider.half_size.x + ball_collider.half_size.x + 0.5);

        commands.trigger(PaddleHitEvent);
        break;
    }
}

pub fn handle_score_and_win(
    mut next_phase: ResMut<NextState<Phase>>,
    config: Res<MatchConfig>,
    mut score: ResMut<Score>,
    mut winner: ResMut<Winner>,
    mut ball_query: BallStateQuery,
) {
    let Ok((mut ball_transform, mut ball_velocity, ball_collider)) = ball_query.single_mut() else {
        return;
    };

    let left_out = -ARENA_HALF_WIDTH - ball_collider.half_size.x;
    let right_out = ARENA_HALF_WIDTH + ball_collider.half_size.x;

    let scorer = if ball_transform.translation.x < left_out {
        Some(PlayerSide::Right)
    } else if ball_transform.translation.x > right_out {
        Some(PlayerSide::Left)
    } else {
        None
    };

    let Some(scorer) = scorer else {
        return;
    };

    if let Some(winning_side) = award_point(&mut score, scorer, config.win_score) {
        winner.side = Some(winning_side);
        next_phase.set(Phase::Winner);
        return;
    }

    let (position, velocity) = reset_ball_state(*config, scorer.opposite());
    ball_transform.translation = position;
    ball_velocity.0 = velocity;
}

pub fn update_score_text(score: Res<Score>, mut text_query: Query<&mut Text2d, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    text.0 = score.formatted();
}

fn intersects_aabb(a_pos: Vec2, a_half: Vec2, b_pos: Vec2, b_half: Vec2) -> bool {
    (a_pos.x - b_pos.x).abs() <= a_half.x + b_half.x
        && (a_pos.y - b_pos.y).abs() <= a_half.y + b_half.y
}

fn compute_paddle_bounce(
    ball_y: f32,
    paddle_y: f32,
    paddle_half_height: f32,
    speed: f32,
    hit_paddle: PlayerSide,
    speed_gain_per_hit: f32,
) -> Vec2 {
    let impact = ((ball_y - paddle_y) / paddle_half_height).clamp(-1.0, 1.0);
    let horizontal = match hit_paddle {
        PlayerSide::Left => 1.0,
        PlayerSide::Right => -1.0,
    };
    let direction = Vec2::new(horizontal, impact * 0.9).normalize_or_zero();
    let direction = if direction == Vec2::ZERO {
        Vec2::new(horizontal, 0.0)
    } else {
        direction
    };

    direction * (speed + speed_gain_per_hit)
}

fn award_point(score: &mut Score, scorer: PlayerSide, win_score: u8) -> Option<PlayerSide> {
    match scorer {
        PlayerSide::Left => score.left = score.left.saturating_add(1),
        PlayerSide::Right => score.right = score.right.saturating_add(1),
    }

    if score.left >= win_score {
        Some(PlayerSide::Left)
    } else if score.right >= win_score {
        Some(PlayerSide::Right)
    } else {
        None
    }
}

fn reset_ball_state(config: MatchConfig, serve_toward: PlayerSide) -> (Vec3, Vec2) {
    let horizontal = match serve_toward {
        PlayerSide::Left => -1.0,
        PlayerSide::Right => 1.0,
    };
    let direction = Vec2::new(horizontal, 0.22).normalize();
    (Vec3::new(0.0, 0.0, 2.0), direction * config.ball_speed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn award_point_returns_winner_when_threshold_is_met() {
        let mut score = Score { left: 4, right: 3 };

        let winner = award_point(&mut score, PlayerSide::Left, 5);

        assert_eq!(winner, Some(PlayerSide::Left));
        assert_eq!(score.left, 5);
        assert_eq!(score.right, 3);
    }

    #[test]
    fn compute_paddle_bounce_reflects_ball_away_from_left_paddle_and_adds_speed() {
        let start_speed = 360.0;
        let bounced = compute_paddle_bounce(30.0, 0.0, 55.0, start_speed, PlayerSide::Left, 26.0);

        assert!(bounced.x > 0.0);
        assert!(bounced.length() > start_speed);
        assert!(bounced.y > 0.0);
    }

    #[test]
    fn reset_ball_state_serves_toward_requested_side() {
        let config = MatchConfig::default();

        let (_, toward_left) = reset_ball_state(config, PlayerSide::Left);
        let (_, toward_right) = reset_ball_state(config, PlayerSide::Right);

        assert!(toward_left.x < 0.0);
        assert!(toward_right.x > 0.0);
        assert_eq!(toward_left.length(), config.ball_speed);
        assert_eq!(toward_right.length(), config.ball_speed);
    }
}
