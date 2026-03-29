pub const WINDOW_WIDTH: u32 = 960;
pub const WINDOW_HEIGHT: u32 = 720;

pub const ARENA_HALF_WIDTH: f32 = 440.0;
pub const ARENA_HALF_HEIGHT: f32 = 320.0;
pub const ARENA_BORDER_THICKNESS: f32 = 3.0;

pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_RADIUS: f32 = 8.0;
pub const PLAYER_FIRE_COOLDOWN: f32 = 0.08;
pub const PLAYER_INVINCIBILITY_DURATION: f32 = 2.0;
pub const PLAYER_BLINK_INTERVAL: f32 = 0.1;
pub const STARTING_LIVES: u32 = 3;
pub const EXTRA_LIFE_EVERY: u32 = 25_000;

pub const BULLET_SPEED: f32 = 800.0;
pub const BULLET_RADIUS: f32 = 3.0;
pub const MAX_PLAYER_BULLETS: u32 = 15;

pub const GRUNT_BASE_SPEED: f32 = 120.0;
pub const GRUNT_RADIUS: f32 = 10.0;

pub const HULK_SPEED: f32 = 60.0;
pub const HULK_RADIUS: f32 = 16.0;
pub const HULK_KNOCKBACK_STRENGTH: f32 = 200.0;
pub const HULK_KNOCKBACK_DECAY: f32 = 5.0;
pub const HULK_WANDER_INTERVAL: f32 = 2.0;

pub const BRAIN_SPEED: f32 = 100.0;
pub const BRAIN_RADIUS: f32 = 10.0;
pub const BRAIN_MISSILE_COOLDOWN: f32 = 3.0;
pub const MISSILE_SPEED: f32 = 150.0;
pub const MISSILE_TURN_RATE: f32 = 2.5;
pub const MISSILE_RADIUS: f32 = 4.0;
pub const MISSILE_LIFETIME: f32 = 6.0;

pub const PROG_SPEED: f32 = 160.0;
pub const PROG_RADIUS: f32 = 8.0;

pub const SPHEROID_SPEED: f32 = 80.0;
pub const SPHEROID_RADIUS: f32 = 12.0;
pub const SPHEROID_SPAWN_COOLDOWN: f32 = 4.0;
pub const SPHEROID_MAX_CHILDREN: u32 = 3;

pub const ENFORCER_SPEED: f32 = 90.0;
pub const ENFORCER_RADIUS: f32 = 8.0;
pub const ENFORCER_FIRE_COOLDOWN: f32 = 2.5;
pub const SPARK_SPEED: f32 = 250.0;
pub const SPARK_RADIUS: f32 = 3.0;
pub const SPARK_LIFETIME: f32 = 3.0;

pub const QUARK_SPEED: f32 = 70.0;
pub const QUARK_RADIUS: f32 = 12.0;
pub const QUARK_SPAWN_COOLDOWN: f32 = 5.0;
pub const QUARK_MAX_CHILDREN: u32 = 2;

pub const TANK_SPEED: f32 = 50.0;
pub const TANK_RADIUS: f32 = 10.0;
pub const TANK_FIRE_COOLDOWN: f32 = 3.0;
pub const SHELL_SPEED: f32 = 200.0;
pub const SHELL_RADIUS: f32 = 4.0;
pub const SHELL_MAX_BOUNCES: u32 = 3;
pub const SHELL_LIFETIME: f32 = 5.0;

pub const ELECTRODE_RADIUS: f32 = 6.0;

pub const HUMAN_SPEED: f32 = 40.0;
pub const HUMAN_RADIUS: f32 = 6.0;
pub const HUMAN_WANDER_INTERVAL: f32 = 2.0;

pub const SPAWN_EXCLUSION_RADIUS: f32 = 100.0;
pub const SPAWN_MIN_SEPARATION: f32 = 20.0;
pub const MAX_TOTAL_ENEMIES: u32 = 150;

pub const WAVE_INTRO_DURATION: f32 = 1.5;
pub const WAVE_CLEAR_DURATION: f32 = 1.0;
pub const DEATH_PAUSE_DURATION: f32 = 1.5;
pub const GAME_OVER_INPUT_DELAY: f32 = 2.0;

pub const EXPLOSION_PARTICLE_COUNT: u32 = 16;
pub const RESCUE_PARTICLE_COUNT: u32 = 10;
pub const DEATH_PARTICLE_COUNT: u32 = 30;
pub const PARTICLE_LIFETIME: f32 = 0.5;
pub const PARTICLE_SPEED: f32 = 300.0;
pub const PARTICLE_RADIUS: f32 = 2.0;
pub const PARTICLE_DRAG: f32 = 3.0;

pub const SCREEN_SHAKE_MAX_OFFSET: f32 = 8.0;
pub const SCREEN_SHAKE_DECAY: f32 = 3.0;

pub const SCORE_POPUP_LIFETIME: f32 = 0.8;
pub const SCORE_POPUP_RISE_SPEED: f32 = 60.0;

pub const HIGH_SCORE_FILE: &str = "highscore.txt";
