use std::{array::from_fn, collections::HashSet, time::Duration};

use bevy::{ecs::system::SystemParam, prelude::*};

pub const INVADER_COLUMNS: usize = 11;
pub const INVADER_ROWS: usize = 5;
pub const SHIELD_COUNT: usize = 4;

const SHIELD_PATTERN: [&str; 4] = ["  ####  ", " ###### ", "########", "###  ###"];
const PLAYER_RESPAWN_SECONDS: f32 = 0.9;
const WAVE_TRANSITION_SECONDS: f32 = 1.35;

pub struct SpaceInvadersGameplayPlugin;

impl Plugin for SpaceInvadersGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(60.0))
            .init_state::<ScreenState>()
            .insert_resource(GameConfig::default())
            .insert_resource(SessionState::default())
            .insert_resource(FormationState::default())
            .insert_resource(Cooldowns::default())
            .insert_resource(PlayerIntent::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<ScoreEvent>()
            .add_message::<PlayerHitEvent>()
            .add_systems(
                OnEnter(ScreenState::Playing),
                (reset_runtime_state, spawn_playfield_entities).chain(),
            )
            .add_systems(
                Update,
                (
                    handle_title_input.run_if(in_state(ScreenState::Title)),
                    handle_game_over_input.run_if(in_state(ScreenState::GameOver)),
                    cache_player_input.run_if(in_state(ScreenState::Playing)),
                    advance_wave_transition.run_if(in_state(ScreenState::WaveTransition)),
                    detect_wave_clear.run_if(in_state(ScreenState::Playing)),
                ),
            )
            .configure_sets(
                FixedUpdate,
                (
                    GameplayFixedSet::Timers,
                    GameplayFixedSet::Player,
                    GameplayFixedSet::Formation,
                    GameplayFixedSet::Movement,
                    GameplayFixedSet::Collision,
                    GameplayFixedSet::Resolve,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    tick_cooldowns.in_set(GameplayFixedSet::Timers),
                    (respawn_player_when_ready, move_player, player_fire)
                        .chain()
                        .in_set(GameplayFixedSet::Player),
                    (advance_formation, invader_fire, spawn_ufo_when_ready)
                        .chain()
                        .in_set(GameplayFixedSet::Formation),
                    move_dynamic_entities.in_set(GameplayFixedSet::Movement),
                    (handle_projectile_collisions, cleanup_out_of_bounds_entities)
                        .chain()
                        .in_set(GameplayFixedSet::Collision),
                    (handle_score_messages, handle_player_hit_messages)
                        .chain()
                        .in_set(GameplayFixedSet::Resolve),
                )
                    .run_if(in_state(ScreenState::Playing)),
            );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum GameplayFixedSet {
    Timers,
    Player,
    Formation,
    Movement,
    Collision,
    Resolve,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum ScreenState {
    #[default]
    Title,
    Playing,
    WaveTransition,
    GameOver,
}

#[derive(Resource, Clone, Debug)]
pub struct GameConfig {
    pub playfield_size: Vec2,
    pub side_padding: f32,
    pub player_speed: f32,
    pub player_size: Vec2,
    pub player_spawn_y: f32,
    pub player_fire_cooldown: f32,
    pub player_projectile_size: Vec2,
    pub enemy_projectile_size: Vec2,
    pub projectile_speed: f32,
    pub enemy_projectile_speed: f32,
    pub invader_spacing: Vec2,
    pub formation_top_y: f32,
    pub formation_step_x: f32,
    pub formation_step_y: f32,
    pub invasion_line_y: f32,
    pub invader_size: Vec2,
    pub shield_y: f32,
    pub shield_spacing: f32,
    pub shield_cell_size: Vec2,
    pub ufo_size: Vec2,
    pub ufo_speed: f32,
    pub enemy_projectile_limit: usize,
    pub lives: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            playfield_size: Vec2::new(520.0, 700.0),
            side_padding: 18.0,
            player_speed: 360.0,
            player_size: Vec2::new(52.0, 28.0),
            player_spawn_y: -305.0,
            player_fire_cooldown: 0.42,
            player_projectile_size: Vec2::new(6.0, 18.0),
            enemy_projectile_size: Vec2::new(8.0, 18.0),
            projectile_speed: 620.0,
            enemy_projectile_speed: 340.0,
            invader_spacing: Vec2::new(38.0, 34.0),
            formation_top_y: 235.0,
            formation_step_x: 14.0,
            formation_step_y: 18.0,
            invasion_line_y: -245.0,
            invader_size: Vec2::new(32.0, 24.0),
            shield_y: -205.0,
            shield_spacing: 118.0,
            shield_cell_size: Vec2::new(12.0, 12.0),
            ufo_size: Vec2::new(68.0, 24.0),
            ufo_speed: 165.0,
            enemy_projectile_limit: 3,
            lives: 3,
        }
    }
}

impl GameConfig {
    pub fn playfield_left(&self) -> f32 {
        -self.playfield_size.x * 0.5
    }

    pub fn playfield_right(&self) -> f32 {
        self.playfield_size.x * 0.5
    }

    pub fn playfield_top(&self) -> f32 {
        self.playfield_size.y * 0.5
    }

    pub fn playfield_bottom(&self) -> f32 {
        -self.playfield_size.y * 0.5
    }

    pub fn player_spawn(&self) -> Vec2 {
        Vec2::new(0.0, self.player_spawn_y)
    }

    pub fn shield_centers(&self) -> [Vec2; SHIELD_COUNT] {
        let offset = self.shield_spacing * 1.5;
        from_fn(|index| Vec2::new(index as f32 * self.shield_spacing - offset, self.shield_y))
    }
}

#[derive(Resource, Debug)]
pub struct SessionState {
    pub score: u32,
    pub lives: u32,
    pub wave: u32,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: GameConfig::default().lives,
            wave: 1,
        }
    }
}

impl SessionState {
    fn reset(&mut self, config: &GameConfig) {
        self.score = 0;
        self.lives = config.lives;
        self.wave = 1;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WaveScaling {
    pub step_seconds: f32,
    pub fire_seconds: f32,
    pub ufo_spawn_seconds: f32,
}

#[derive(Resource, Debug)]
pub struct FormationState {
    pub direction: f32,
    pub step_timer: Timer,
    pub fire_column_cursor: usize,
}

impl Default for FormationState {
    fn default() -> Self {
        Self {
            direction: 1.0,
            step_timer: armed_timer(0.6),
            fire_column_cursor: 0,
        }
    }
}

impl FormationState {
    fn reset_for_wave(&mut self, wave: u32) {
        let total = INVADER_COLUMNS * INVADER_ROWS;
        let scaling = wave_scaling(wave, total, total);
        self.direction = 1.0;
        self.fire_column_cursor = wave.saturating_sub(1) as usize % INVADER_COLUMNS;
        self.step_timer = armed_timer(scaling.step_seconds);
    }
}

#[derive(Resource, Debug)]
pub struct Cooldowns {
    pub player_fire: Timer,
    pub invader_fire: Timer,
    pub ufo_spawn: Timer,
    pub respawn: Timer,
    pub wave_transition: Timer,
    pub ufo_direction: f32,
}

impl Default for Cooldowns {
    fn default() -> Self {
        Self {
            player_fire: ready_timer(0.4),
            invader_fire: armed_timer(1.0),
            ufo_spawn: armed_timer(9.0),
            respawn: ready_timer(PLAYER_RESPAWN_SECONDS),
            wave_transition: ready_timer(WAVE_TRANSITION_SECONDS),
            ufo_direction: 1.0,
        }
    }
}

impl Cooldowns {
    fn reset_for_wave(&mut self, session: &SessionState, config: &GameConfig) {
        let total = INVADER_COLUMNS * INVADER_ROWS;
        let scaling = wave_scaling(session.wave, total, total);
        self.player_fire = ready_timer(config.player_fire_cooldown);
        self.invader_fire = armed_timer(scaling.fire_seconds);
        self.ufo_spawn = armed_timer(scaling.ufo_spawn_seconds);
        self.respawn = ready_timer(PLAYER_RESPAWN_SECONDS);
        self.wave_transition = ready_timer(WAVE_TRANSITION_SECONDS);
    }

    fn arm_player_fire(&mut self, seconds: f32) {
        self.player_fire = armed_timer(seconds);
    }

    fn arm_respawn(&mut self) {
        self.respawn = armed_timer(PLAYER_RESPAWN_SECONDS);
    }

    fn arm_wave_transition(&mut self) {
        self.wave_transition = armed_timer(WAVE_TRANSITION_SECONDS);
    }
}

#[derive(Resource, Default, Debug)]
pub struct PlayerIntent {
    pub move_axis: f32,
    pub fire_requested: bool,
}

impl PlayerIntent {
    fn clear(&mut self) {
        self.move_axis = 0.0;
        self.fire_requested = false;
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Invader {
    pub row_kind: InvaderRow,
    pub row: usize,
    pub column: usize,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Projectile {
    pub owner: ProjectileOwner,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShieldCell {
    pub shield: usize,
    pub row: usize,
    pub column: usize,
}

#[derive(Component)]
pub struct Ufo;

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component, Clone, Copy, Debug)]
pub struct Collider {
    pub size: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvaderRow {
    Commander,
    Guard,
    Drone,
}

impl InvaderRow {
    pub fn from_index(row: usize) -> Self {
        match row {
            0 => Self::Commander,
            1 | 2 => Self::Guard,
            _ => Self::Drone,
        }
    }

    pub fn score(self) -> u32 {
        score_for_row(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileOwner {
    Player,
    Invader,
}

#[derive(Message)]
pub struct ScoreEvent {
    pub points: u32,
}

#[derive(Message)]
pub struct PlayerHitEvent;

type InvaderSnapshotQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform, &'static Collider), With<Invader>>;
type InvaderTransformQuery<'w, 's> = Query<'w, 's, &'static mut Transform, With<Invader>>;
type InvaderParamSet<'w, 's> =
    ParamSet<'w, 's, (InvaderSnapshotQuery<'w, 's>, InvaderTransformQuery<'w, 's>)>;
type DynamicMoverQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static Projectile>,
        Option<&'static Ufo>,
        &'static Transform,
        &'static Collider,
    ),
>;

#[derive(SystemParam)]
struct InvaderFormationQueries<'w, 's> {
    set: InvaderParamSet<'w, 's>,
}

#[derive(SystemParam)]
struct CollisionMessages<'w> {
    score_messages: MessageWriter<'w, ScoreEvent>,
    player_hit_messages: MessageWriter<'w, PlayerHitEvent>,
}

#[derive(SystemParam)]
struct CollisionQueries<'w, 's> {
    projectiles: Query<
        'w,
        's,
        (
            Entity,
            &'static Projectile,
            &'static Transform,
            &'static Collider,
        ),
    >,
    invaders: Query<
        'w,
        's,
        (
            Entity,
            &'static Invader,
            &'static Transform,
            &'static Collider,
        ),
    >,
    shields: Query<
        'w,
        's,
        (
            Entity,
            &'static ShieldCell,
            &'static Transform,
            &'static Collider,
        ),
    >,
    players: Query<'w, 's, (Entity, &'static Transform, &'static Collider), With<Player>>,
    ufos: Query<'w, 's, (Entity, &'static Transform, &'static Collider), With<Ufo>>,
}

#[derive(SystemParam)]
struct DynamicEntities<'w, 's> {
    movers: DynamicMoverQuery<'w, 's>,
}

fn handle_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut session: ResMut<SessionState>,
    mut intent: ResMut<PlayerIntent>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    if start_pressed(&keys) {
        session.reset(&config);
        intent.clear();
        next_state.set(ScreenState::Playing);
    }
}

fn handle_game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<GameConfig>,
    mut session: ResMut<SessionState>,
    mut intent: ResMut<PlayerIntent>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    if start_pressed(&keys) {
        session.reset(&config);
        intent.clear();
        next_state.set(ScreenState::Playing);
    }
}

fn cache_player_input(keys: Res<ButtonInput<KeyCode>>, mut intent: ResMut<PlayerIntent>) {
    let left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);
    intent.move_axis = right as i8 as f32 - left as i8 as f32;
    intent.fire_requested |= keys.just_pressed(KeyCode::Space);
}

fn reset_runtime_state(
    config: Res<GameConfig>,
    session: Res<SessionState>,
    mut formation: ResMut<FormationState>,
    mut cooldowns: ResMut<Cooldowns>,
    mut intent: ResMut<PlayerIntent>,
) {
    formation.reset_for_wave(session.wave);
    cooldowns.reset_for_wave(&session, &config);
    cooldowns.ufo_direction = if session.wave.is_multiple_of(2) {
        -1.0
    } else {
        1.0
    };
    intent.clear();
}

fn spawn_playfield_entities(mut commands: Commands, config: Res<GameConfig>) {
    spawn_player(&mut commands, &config);
    spawn_invader_formation(&mut commands, &config);
    spawn_shields(&mut commands, &config);
}

fn advance_wave_transition(
    time: Res<Time>,
    mut cooldowns: ResMut<Cooldowns>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    if cooldowns.wave_transition.is_finished() {
        next_state.set(ScreenState::Playing);
        return;
    }

    cooldowns.wave_transition.tick(time.delta());
    if cooldowns.wave_transition.just_finished() {
        next_state.set(ScreenState::Playing);
    }
}

fn tick_cooldowns(time: Res<Time<Fixed>>, mut cooldowns: ResMut<Cooldowns>) {
    let delta = time.delta();
    cooldowns.player_fire.tick(delta);
    cooldowns.invader_fire.tick(delta);
    cooldowns.ufo_spawn.tick(delta);
    cooldowns.respawn.tick(delta);
}

fn respawn_player_when_ready(
    mut commands: Commands,
    config: Res<GameConfig>,
    cooldowns: Res<Cooldowns>,
    players: Query<Entity, With<Player>>,
) {
    if players.is_empty() && cooldowns.respawn.just_finished() {
        spawn_player(&mut commands, &config);
    }
}

fn move_player(
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
    intent: Res<PlayerIntent>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = players.single_mut() else {
        return;
    };

    let next_x =
        transform.translation.x + intent.move_axis * config.player_speed * time.delta_secs();
    transform.translation.x = clamp_player_x(next_x, &config);
}

fn player_fire(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut cooldowns: ResMut<Cooldowns>,
    mut intent: ResMut<PlayerIntent>,
    players: Query<&Transform, With<Player>>,
    projectiles: Query<&Projectile>,
) {
    if !intent.fire_requested {
        return;
    }

    intent.fire_requested = false;

    if !cooldowns.player_fire.is_finished() {
        return;
    }

    if projectiles
        .iter()
        .any(|projectile| projectile.owner == ProjectileOwner::Player)
    {
        return;
    }

    let Ok(player_transform) = players.single() else {
        return;
    };

    let spawn_position = Vec2::new(
        player_transform.translation.x,
        player_transform.translation.y
            + config.player_size.y * 0.5
            + config.player_projectile_size.y * 0.5
            + 2.0,
    );

    spawn_projectile(
        &mut commands,
        spawn_position,
        ProjectileOwner::Player,
        &config,
    );
    cooldowns.arm_player_fire(config.player_fire_cooldown);
}

fn advance_formation(
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
    session: Res<SessionState>,
    mut formation: ResMut<FormationState>,
    mut next_state: ResMut<NextState<ScreenState>>,
    mut invaders: InvaderFormationQueries,
) {
    let invader_data: Vec<(Entity, Vec3, Vec2)> = invaders
        .set
        .p0()
        .iter()
        .map(|(entity, transform, collider)| (entity, transform.translation, collider.size))
        .collect();

    if invader_data.is_empty() {
        return;
    }

    formation.step_timer.tick(time.delta());
    if !formation.step_timer.just_finished() {
        return;
    }

    let mut left = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    let mut bottom = f32::INFINITY;

    for (_, translation, size) in &invader_data {
        left = left.min(translation.x - size.x * 0.5);
        right = right.max(translation.x + size.x * 0.5);
        bottom = bottom.min(translation.y - size.y * 0.5);
    }

    let descending = formation_hits_edge(left, right, formation.direction, &config);
    let delta = if descending {
        Vec3::new(0.0, -config.formation_step_y, 0.0)
    } else {
        Vec3::new(formation.direction * config.formation_step_x, 0.0, 0.0)
    };

    for (entity, _, _) in &invader_data {
        if let Ok(mut transform) = invaders.set.p1().get_mut(*entity) {
            transform.translation += delta;
        }
    }

    if descending {
        formation.direction *= -1.0;
        if bottom - config.formation_step_y <= config.invasion_line_y {
            next_state.set(ScreenState::GameOver);
        }
    }

    let scaling = wave_scaling(
        session.wave,
        invader_data.len(),
        INVADER_COLUMNS * INVADER_ROWS,
    );
    formation.step_timer = armed_timer(scaling.step_seconds);
}

fn invader_fire(
    mut commands: Commands,
    config: Res<GameConfig>,
    session: Res<SessionState>,
    mut formation: ResMut<FormationState>,
    mut cooldowns: ResMut<Cooldowns>,
    invaders: Query<(&Invader, &Transform, &Collider)>,
    projectiles: Query<&Projectile>,
) {
    if !cooldowns.invader_fire.is_finished() {
        return;
    }

    let enemy_projectiles = projectiles
        .iter()
        .filter(|projectile| projectile.owner == ProjectileOwner::Invader)
        .count();

    if enemy_projectiles >= config.enemy_projectile_limit {
        return;
    }

    #[derive(Clone, Copy)]
    struct Shooter {
        row: usize,
        position: Vec3,
        collider: Vec2,
    }

    let mut living_invaders = 0;
    let mut shooters: [Option<Shooter>; INVADER_COLUMNS] = from_fn(|_| None);
    for (invader, transform, collider) in &invaders {
        living_invaders += 1;
        let slot = &mut shooters[invader.column];
        if slot.is_none_or(|candidate| invader.row > candidate.row) {
            *slot = Some(Shooter {
                row: invader.row,
                position: transform.translation,
                collider: collider.size,
            });
        }
    }

    for offset in 0..INVADER_COLUMNS {
        let column = (formation.fire_column_cursor + offset) % INVADER_COLUMNS;
        if let Some(shooter) = shooters[column] {
            let spawn_position = Vec2::new(
                shooter.position.x,
                shooter.position.y - shooter.collider.y * 0.5 - config.enemy_projectile_size.y,
            );
            spawn_projectile(
                &mut commands,
                spawn_position,
                ProjectileOwner::Invader,
                &config,
            );
            formation.fire_column_cursor = (column + 1) % INVADER_COLUMNS;
            let scaling = wave_scaling(
                session.wave,
                living_invaders,
                INVADER_COLUMNS * INVADER_ROWS,
            );
            cooldowns.invader_fire = armed_timer(scaling.fire_seconds);
            break;
        }
    }
}

fn spawn_ufo_when_ready(
    mut commands: Commands,
    config: Res<GameConfig>,
    session: Res<SessionState>,
    mut cooldowns: ResMut<Cooldowns>,
    ufo_query: Query<Entity, With<Ufo>>,
) {
    if !cooldowns.ufo_spawn.is_finished() || !ufo_query.is_empty() {
        return;
    }

    let direction = cooldowns.ufo_direction;
    cooldowns.ufo_direction *= -1.0;

    let x = if direction > 0.0 {
        config.playfield_left() - config.ufo_size.x
    } else {
        config.playfield_right() + config.ufo_size.x
    };

    commands.spawn((
        Name::new("Ufo"),
        Ufo,
        Collider {
            size: config.ufo_size,
        },
        Velocity(Vec2::new(direction * config.ufo_speed, 0.0)),
        Transform::from_xyz(x, config.playfield_top() - 46.0, 4.0),
        DespawnOnExit(ScreenState::Playing),
    ));

    let scaling = wave_scaling(
        session.wave,
        INVADER_COLUMNS * INVADER_ROWS,
        INVADER_COLUMNS * INVADER_ROWS,
    );
    cooldowns.ufo_spawn = armed_timer(scaling.ufo_spawn_seconds);
}

fn move_dynamic_entities(time: Res<Time<Fixed>>, mut movers: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in &mut movers {
        transform.translation += velocity.extend(0.0) * time.delta_secs();
    }
}

fn handle_projectile_collisions(
    mut commands: Commands,
    mut messages: CollisionMessages,
    queries: CollisionQueries,
) {
    let mut despawn_projectiles = HashSet::new();
    let mut despawn_invaders = HashSet::new();
    let mut despawn_shields = HashSet::new();
    let mut despawn_ufos = HashSet::new();
    let mut score_gain = 0;
    let mut player_hit = false;

    let player_data = queries.players.single().ok();
    let ufo_data = queries.ufos.single().ok();

    for (projectile_entity, projectile, projectile_transform, projectile_collider) in
        &queries.projectiles
    {
        if despawn_projectiles.contains(&projectile_entity) {
            continue;
        }

        match projectile.owner {
            ProjectileOwner::Player => {
                let mut hit_target = false;

                for (invader_entity, invader, transform, collider) in &queries.invaders {
                    if despawn_invaders.contains(&invader_entity) {
                        continue;
                    }

                    if intersects(
                        projectile_transform.translation,
                        projectile_collider.size,
                        transform.translation,
                        collider.size,
                    ) {
                        despawn_projectiles.insert(projectile_entity);
                        despawn_invaders.insert(invader_entity);
                        score_gain += invader.row_kind.score();
                        hit_target = true;
                        break;
                    }
                }

                if hit_target {
                    continue;
                }

                if let Some((ufo_entity, transform, collider)) = ufo_data
                    && !despawn_ufos.contains(&ufo_entity)
                    && intersects(
                        projectile_transform.translation,
                        projectile_collider.size,
                        transform.translation,
                        collider.size,
                    )
                {
                    despawn_projectiles.insert(projectile_entity);
                    despawn_ufos.insert(ufo_entity);
                    score_gain += 100;
                    continue;
                }

                for (shield_entity, _, transform, collider) in &queries.shields {
                    if despawn_shields.contains(&shield_entity) {
                        continue;
                    }

                    if intersects(
                        projectile_transform.translation,
                        projectile_collider.size,
                        transform.translation,
                        collider.size,
                    ) {
                        despawn_projectiles.insert(projectile_entity);
                        despawn_shields.insert(shield_entity);
                        break;
                    }
                }
            }
            ProjectileOwner::Invader => {
                if let Some((_, transform, collider)) = player_data
                    && intersects(
                        projectile_transform.translation,
                        projectile_collider.size,
                        transform.translation,
                        collider.size,
                    )
                {
                    despawn_projectiles.insert(projectile_entity);
                    player_hit = true;
                    continue;
                }

                for (shield_entity, _, transform, collider) in &queries.shields {
                    if despawn_shields.contains(&shield_entity) {
                        continue;
                    }

                    if intersects(
                        projectile_transform.translation,
                        projectile_collider.size,
                        transform.translation,
                        collider.size,
                    ) {
                        despawn_projectiles.insert(projectile_entity);
                        despawn_shields.insert(shield_entity);
                        break;
                    }
                }
            }
        }
    }

    for entity in despawn_projectiles {
        despawn_entity(&mut commands, entity);
    }

    for entity in despawn_invaders {
        despawn_entity(&mut commands, entity);
    }

    for entity in despawn_shields {
        despawn_entity(&mut commands, entity);
    }

    for entity in despawn_ufos {
        despawn_entity(&mut commands, entity);
    }

    if score_gain > 0 {
        messages
            .score_messages
            .write(ScoreEvent { points: score_gain });
    }

    if player_hit {
        messages.player_hit_messages.write(PlayerHitEvent);
    }
}

fn cleanup_out_of_bounds_entities(
    mut commands: Commands,
    config: Res<GameConfig>,
    entities: DynamicEntities,
) {
    let horizontal_margin = 60.0;
    let vertical_margin = 30.0;

    for (entity, projectile, ufo, transform, collider) in &entities.movers {
        let x = transform.translation.x;
        let y = transform.translation.y;
        let half = collider.size * 0.5;

        let outside = projectile.is_some_and(|_| {
            x + half.x < config.playfield_left() - horizontal_margin
                || x - half.x > config.playfield_right() + horizontal_margin
                || y - half.y > config.playfield_top() + vertical_margin
                || y + half.y < config.playfield_bottom() - vertical_margin
        }) || ufo.is_some_and(|_| {
            x + half.x < config.playfield_left() - horizontal_margin
                || x - half.x > config.playfield_right() + horizontal_margin
        });

        if outside {
            despawn_entity(&mut commands, entity);
        }
    }
}

fn handle_score_messages(
    mut score_messages: MessageReader<ScoreEvent>,
    mut session: ResMut<SessionState>,
) {
    for message in score_messages.read() {
        session.score += message.points;
    }
}

fn handle_player_hit_messages(
    mut commands: Commands,
    mut player_hit_messages: MessageReader<PlayerHitEvent>,
    mut session: ResMut<SessionState>,
    mut cooldowns: ResMut<Cooldowns>,
    mut next_state: ResMut<NextState<ScreenState>>,
    players: Query<Entity, With<Player>>,
    projectiles: Query<(Entity, &Projectile)>,
) {
    if player_hit_messages.read().next().is_none() {
        return;
    }

    let Ok(player_entity) = players.single() else {
        return;
    };

    despawn_entity(&mut commands, player_entity);
    for (entity, projectile) in &projectiles {
        if projectile.owner == ProjectileOwner::Invader {
            despawn_entity(&mut commands, entity);
        }
    }

    if session.lives > 0 {
        session.lives -= 1;
    }

    if session.lives == 0 {
        next_state.set(ScreenState::GameOver);
    } else {
        cooldowns.arm_respawn();
    }
}

fn detect_wave_clear(
    invaders: Query<Entity, With<Invader>>,
    mut session: ResMut<SessionState>,
    mut cooldowns: ResMut<Cooldowns>,
    mut next_state: ResMut<NextState<ScreenState>>,
) {
    if invaders.is_empty() && cooldowns.wave_transition.is_finished() {
        session.wave += 1;
        cooldowns.arm_wave_transition();
        next_state.set(ScreenState::WaveTransition);
    }
}

fn spawn_player(commands: &mut Commands, config: &GameConfig) {
    commands.spawn((
        Name::new("Player"),
        Player,
        Collider {
            size: config.player_size,
        },
        Transform::from_xyz(config.player_spawn().x, config.player_spawn().y, 5.0),
        DespawnOnExit(ScreenState::Playing),
    ));
}

fn spawn_invader_formation(commands: &mut Commands, config: &GameConfig) {
    let start_x = -((INVADER_COLUMNS - 1) as f32 * config.invader_spacing.x) * 0.5;
    for row in 0..INVADER_ROWS {
        for column in 0..INVADER_COLUMNS {
            let position = Vec3::new(
                start_x + column as f32 * config.invader_spacing.x,
                config.formation_top_y - row as f32 * config.invader_spacing.y,
                3.0,
            );

            commands.spawn((
                Name::new(format!("Invader {row}-{column}")),
                Invader {
                    row_kind: InvaderRow::from_index(row),
                    row,
                    column,
                },
                Collider {
                    size: config.invader_size,
                },
                Transform::from_translation(position),
                DespawnOnExit(ScreenState::Playing),
            ));
        }
    }
}

fn spawn_shields(commands: &mut Commands, config: &GameConfig) {
    let rows = SHIELD_PATTERN.len();
    let columns = SHIELD_PATTERN[0].len();

    for (shield_index, center) in config.shield_centers().into_iter().enumerate() {
        for (row, pattern_row) in SHIELD_PATTERN.iter().enumerate() {
            for (column, tile) in pattern_row.as_bytes().iter().enumerate() {
                if *tile != b'#' {
                    continue;
                }

                let position = shield_cell_world_position(
                    center,
                    row,
                    column,
                    rows,
                    columns,
                    config.shield_cell_size,
                );

                commands.spawn((
                    Name::new(format!("ShieldCell {shield_index}-{row}-{column}")),
                    ShieldCell {
                        shield: shield_index,
                        row,
                        column,
                    },
                    Collider {
                        size: config.shield_cell_size * 0.92,
                    },
                    Transform::from_xyz(position.x, position.y, 2.0),
                    DespawnOnExit(ScreenState::Playing),
                ));
            }
        }
    }
}

fn spawn_projectile(
    commands: &mut Commands,
    position: Vec2,
    owner: ProjectileOwner,
    config: &GameConfig,
) {
    let (size, velocity, z) = match owner {
        ProjectileOwner::Player => (
            config.player_projectile_size,
            Vec2::new(0.0, config.projectile_speed),
            6.0,
        ),
        ProjectileOwner::Invader => (
            config.enemy_projectile_size,
            Vec2::new(0.0, -config.enemy_projectile_speed),
            4.0,
        ),
    };

    commands.spawn((
        Name::new(match owner {
            ProjectileOwner::Player => "PlayerProjectile",
            ProjectileOwner::Invader => "InvaderProjectile",
        }),
        Projectile { owner },
        Collider { size },
        Velocity(velocity),
        Transform::from_xyz(position.x, position.y, z),
        DespawnOnExit(ScreenState::Playing),
    ));
}

fn despawn_entity(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).despawn();
}

fn start_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space)
}

fn intersects(a_translation: Vec3, a_size: Vec2, b_translation: Vec3, b_size: Vec2) -> bool {
    let a_min = a_translation.truncate() - a_size * 0.5;
    let a_max = a_translation.truncate() + a_size * 0.5;
    let b_min = b_translation.truncate() - b_size * 0.5;
    let b_max = b_translation.truncate() + b_size * 0.5;

    a_min.x <= b_max.x && a_max.x >= b_min.x && a_min.y <= b_max.y && a_max.y >= b_min.y
}

pub fn clamp_player_x(x: f32, config: &GameConfig) -> f32 {
    let min = config.playfield_left() + config.side_padding + config.player_size.x * 0.5;
    let max = config.playfield_right() - config.side_padding - config.player_size.x * 0.5;
    x.clamp(min, max)
}

pub fn formation_hits_edge(left: f32, right: f32, direction: f32, config: &GameConfig) -> bool {
    let next_left = left + direction * config.formation_step_x;
    let next_right = right + direction * config.formation_step_x;
    next_left <= config.playfield_left() + config.side_padding
        || next_right >= config.playfield_right() - config.side_padding
}

pub fn score_for_row(row: InvaderRow) -> u32 {
    match row {
        InvaderRow::Commander => 30,
        InvaderRow::Guard => 20,
        InvaderRow::Drone => 10,
    }
}

pub fn wave_scaling(wave: u32, living_invaders: usize, total_invaders: usize) -> WaveScaling {
    let wave_index = wave.saturating_sub(1) as f32;
    let total_invaders = total_invaders.max(1) as f32;
    let living_ratio = (living_invaders as f32 / total_invaders).clamp(0.0, 1.0);
    let wave_factor = (wave_index * 0.05).min(0.24);

    let step_seconds = ((0.64 - wave_factor) * (0.4 + living_ratio * 0.6)).clamp(0.08, 0.64);
    let fire_seconds = ((1.02 - wave_factor) * (0.7 + living_ratio * 0.3)).clamp(0.26, 1.02);
    let ufo_spawn_seconds = (9.2 - wave_index * 0.35).clamp(5.0, 9.2);

    WaveScaling {
        step_seconds,
        fire_seconds,
        ufo_spawn_seconds,
    }
}

pub fn shield_cell_world_position(
    shield_center: Vec2,
    row: usize,
    column: usize,
    rows: usize,
    columns: usize,
    cell_size: Vec2,
) -> Vec2 {
    let width = columns as f32 * cell_size.x;
    let height = rows as f32 * cell_size.y;
    let start = shield_center
        - Vec2::new(
            width * 0.5 - cell_size.x * 0.5,
            -height * 0.5 + cell_size.y * 0.5,
        );

    Vec2::new(
        start.x + column as f32 * cell_size.x,
        start.y - row as f32 * cell_size.y,
    )
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn shield_cell_index(
    local_position: Vec2,
    cell_size: Vec2,
    rows: usize,
    columns: usize,
) -> Option<(usize, usize)> {
    let width = columns as f32 * cell_size.x;
    let height = rows as f32 * cell_size.y;
    let x = local_position.x + width * 0.5;
    let y = height * 0.5 - local_position.y;

    if x < 0.0 || x >= width || y < 0.0 || y >= height {
        return None;
    }

    let column = (x / cell_size.x).floor() as usize;
    let row = (y / cell_size.y).floor() as usize;
    Some((row, column))
}

fn ready_timer(seconds: f32) -> Timer {
    let duration = Duration::from_secs_f32(seconds.max(0.001));
    let mut timer = Timer::new(duration, TimerMode::Once);
    timer.tick(duration);
    timer
}

fn armed_timer(seconds: f32) -> Timer {
    let duration = Duration::from_secs_f32(seconds.max(0.001));
    let mut timer = Timer::new(duration, TimerMode::Once);
    timer.reset();
    timer
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{app::App, prelude::*, state::app::StatesPlugin, time::TimeUpdateStrategy};

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));
        app.add_plugins(SpaceInvadersGameplayPlugin);
        app.update();
        app
    }

    fn press_key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }

    fn clear_input(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }

    fn enter_playing(app: &mut App) {
        press_key(app, KeyCode::Space);
        app.update();
        clear_input(app);
        app.update();
        assert_eq!(
            app.world().resource::<State<ScreenState>>().get(),
            &ScreenState::Playing
        );
    }

    #[test]
    fn score_table_matches_classic_rows() {
        assert_eq!(score_for_row(InvaderRow::Commander), 30);
        assert_eq!(score_for_row(InvaderRow::Guard), 20);
        assert_eq!(score_for_row(InvaderRow::Drone), 10);
    }

    #[test]
    fn player_clamp_respects_playfield() {
        let config = GameConfig::default();
        let min = clamp_player_x(-10_000.0, &config);
        let max = clamp_player_x(10_000.0, &config);

        assert!(min >= config.playfield_left());
        assert!(max <= config.playfield_right());
    }

    #[test]
    fn formation_edge_detection_flips_near_boundaries() {
        let config = GameConfig::default();
        let left = config.playfield_left() + config.side_padding;
        let right = config.playfield_right() - config.side_padding;

        assert!(formation_hits_edge(left, right, 1.0, &config));
        assert!(formation_hits_edge(left, right, -1.0, &config));
    }

    #[test]
    fn wave_scaling_accelerates_over_time() {
        let early = wave_scaling(
            1,
            INVADER_COLUMNS * INVADER_ROWS,
            INVADER_COLUMNS * INVADER_ROWS,
        );
        let later = wave_scaling(4, 6, INVADER_COLUMNS * INVADER_ROWS);

        assert!(later.step_seconds < early.step_seconds);
        assert!(later.fire_seconds < early.fire_seconds);
        assert!(later.ufo_spawn_seconds <= early.ufo_spawn_seconds);
    }

    #[test]
    fn shield_cell_mapping_matches_grid() {
        let index = shield_cell_index(Vec2::new(0.0, 0.0), Vec2::splat(12.0), 4, 8).unwrap();
        assert_eq!(index, (2, 4));
        assert!(shield_cell_index(Vec2::new(100.0, 0.0), Vec2::splat(12.0), 4, 8).is_none());
    }

    #[test]
    fn title_transitions_to_playing() {
        let mut app = test_app();
        enter_playing(&mut app);
    }

    #[test]
    fn player_shot_cap_allows_only_one_active_projectile() {
        let mut app = test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<PlayerIntent>()
            .fire_requested = true;
        app.world_mut().run_schedule(FixedUpdate);

        app.world_mut()
            .resource_mut::<PlayerIntent>()
            .fire_requested = true;
        app.world_mut().run_schedule(FixedUpdate);

        let projectile_count = app
            .world_mut()
            .query::<&Projectile>()
            .iter(app.world())
            .filter(|projectile| projectile.owner == ProjectileOwner::Player)
            .count();

        assert_eq!(projectile_count, 1);
    }

    #[test]
    fn destroying_invader_adds_score_and_despawns_entity() {
        let mut app = test_app();
        enter_playing(&mut app);

        let (target_entity, target_translation) = app
            .world_mut()
            .query::<(Entity, &Invader, &Transform)>()
            .iter(app.world())
            .find(|(_, invader, _)| invader.row_kind == InvaderRow::Commander)
            .map(|(entity, _, transform)| (entity, transform.translation))
            .unwrap();

        app.world_mut().spawn((
            Projectile {
                owner: ProjectileOwner::Player,
            },
            Collider {
                size: GameConfig::default().player_projectile_size,
            },
            Transform::from_translation(target_translation),
            Velocity(Vec2::ZERO),
            DespawnOnExit(ScreenState::Playing),
        ));

        app.update();

        assert!(app.world().get_entity(target_entity).is_err());
        assert_eq!(app.world().resource::<SessionState>().score, 30);
    }

    #[test]
    fn empty_formation_enters_wave_transition() {
        let mut app = test_app();
        enter_playing(&mut app);

        let invaders: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<Invader>>()
            .iter(app.world())
            .collect();
        for entity in invaders {
            app.world_mut().despawn(entity);
        }

        let remaining = app
            .world_mut()
            .query_filtered::<Entity, With<Invader>>()
            .iter(app.world())
            .count();
        assert_eq!(remaining, 0);
        assert!(
            app.world()
                .resource::<Cooldowns>()
                .wave_transition
                .is_finished()
        );

        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<State<ScreenState>>().get(),
            &ScreenState::WaveTransition
        );
        assert_eq!(app.world().resource::<SessionState>().wave, 2);
    }

    #[test]
    fn last_life_enters_game_over() {
        let mut app = test_app();
        enter_playing(&mut app);

        app.world_mut().resource_mut::<SessionState>().lives = 1;
        app.world_mut()
            .resource_mut::<Messages<PlayerHitEvent>>()
            .write(PlayerHitEvent);

        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<State<ScreenState>>().get(),
            &ScreenState::GameOver
        );
    }
}
