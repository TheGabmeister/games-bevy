use bevy::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// One level's layout as authored in `assets/levels/round-NN.ron`.
#[derive(Deserialize)]
struct LevelData {
    /// Rows of 9-cell layout strings; codes map via `BrickKind::from_code`.
    rows: Vec<String>,
}

/// Every brick layout loaded from `assets/levels/round-*.ron`, in filename (round) order.
/// `bricks.rs` indexes this per round (wrapping past the end). Loaded once at startup.
///
/// This reads the RON files synchronously via `std::fs` (so the data is guaranteed present
/// before the first round spawns); the fuller Phase 7 set can move to Bevy's async asset
/// pipeline if hot-reloading is wanted.
#[derive(Resource)]
pub struct Levels(pub Vec<Vec<String>>);

impl FromWorld for Levels {
    fn from_world(_world: &mut World) -> Self {
        // CARGO_MANIFEST_DIR keeps this independent of the process working directory.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/levels");

        let mut files: Vec<_> = fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("reading level dir {}: {e}", dir.display()))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "ron"))
            .collect();
        files.sort();

        let levels: Vec<Vec<String>> = files
            .iter()
            .map(|path| {
                let text = fs::read_to_string(path)
                    .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
                let data: LevelData = ron::from_str(&text)
                    .unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
                data.rows
            })
            .collect();

        assert!(
            !levels.is_empty(),
            "no level .ron files found in {}",
            dir.display()
        );
        Levels(levels)
    }
}
