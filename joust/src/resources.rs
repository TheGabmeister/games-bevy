use bevy::prelude::*;

use crate::constants::*;

#[derive(Resource)]
pub struct GameState {
    pub scores: [u32; 2],
    pub lives: [u32; 2],
    pub wave: u32,
    pub high_score: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            scores: [0, 0],
            lives: [3, 3],
            wave: 1,
            high_score: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct RespawnTimers {
    pub timers: Vec<(u8, Timer)>,
}

#[derive(Resource)]
pub struct WaveTimer(pub Timer);

#[derive(Resource)]
pub struct PlayerCount(pub u8);

impl Default for PlayerCount {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Resource)]
pub struct SharedMeshes {
    pub rect_body: Handle<Mesh>,
    pub circle_head: Handle<Mesh>,
    pub rect_lance: Handle<Mesh>,
    pub rect_wing: Handle<Mesh>,
    pub circle_egg: Handle<Mesh>,
    pub circle_particle: Handle<Mesh>,
    pub rect_platform: [Handle<Mesh>; 6],
    pub rect_lava: Handle<Mesh>,
}

#[derive(Resource)]
pub struct SharedMaterials {
    pub player1_body: Handle<ColorMaterial>,
    pub player2_body: Handle<ColorMaterial>,
    pub bounder_body: Handle<ColorMaterial>,
    pub hunter_body: Handle<ColorMaterial>,
    pub shadow_lord_body: Handle<ColorMaterial>,
    pub lance: Handle<ColorMaterial>,
    pub head: Handle<ColorMaterial>,
    pub egg: Handle<ColorMaterial>,
    pub egg_hatch: Handle<ColorMaterial>,
    pub platform: Handle<ColorMaterial>,
    pub lava_front: Handle<ColorMaterial>,
    pub lava_back: Handle<ColorMaterial>,
}

pub struct WaveDef {
    pub bounders: u32,
    pub hunters: u32,
    pub shadow_lords: u32,
    pub egg_hatch_time: f32,
}

pub fn get_wave_def(wave: u32) -> WaveDef {
    match wave {
        1 => WaveDef { bounders: 3, hunters: 0, shadow_lords: 0, egg_hatch_time: EGG_HATCH_TIME_BASE },
        2 => WaveDef { bounders: 4, hunters: 1, shadow_lords: 0, egg_hatch_time: 14.0 },
        3 => WaveDef { bounders: 3, hunters: 2, shadow_lords: 0, egg_hatch_time: 13.0 },
        4 => WaveDef { bounders: 2, hunters: 3, shadow_lords: 0, egg_hatch_time: 12.0 },
        5 => WaveDef { bounders: 1, hunters: 3, shadow_lords: 1, egg_hatch_time: 11.0 },
        6 => WaveDef { bounders: 0, hunters: 3, shadow_lords: 2, egg_hatch_time: 10.0 },
        _ => {
            let sl = (wave - 5).min(5);
            let h = 5u32.saturating_sub(sl / 2);
            WaveDef {
                bounders: 0,
                hunters: h,
                shadow_lords: sl,
                egg_hatch_time: (10.0 - wave as f32 * 0.3).max(5.0),
            }
        }
    }
}
