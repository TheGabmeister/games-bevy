use bevy::prelude::*;

pub struct PrimitiveRenderingPlugin;

impl Plugin for PrimitiveRenderingPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Clone, Copy, Debug)]
pub enum WorldColor {
    Backdrop,
    RoomFloor,
    HudPanel,
    Doorway,
    Player,
    Enemy,
    Pickup,
    Hazard,
    Accent,
    UiText,
}

impl WorldColor {
    pub fn color(self) -> Color {
        match self {
            Self::Backdrop => Color::srgb(0.04, 0.07, 0.08),
            Self::RoomFloor => Color::srgb(0.17, 0.22, 0.17),
            Self::HudPanel => Color::srgb(0.10, 0.15, 0.17),
            Self::Doorway => Color::srgb(0.89, 0.76, 0.35),
            Self::Player => Color::srgb(0.33, 0.78, 0.38),
            Self::Enemy => Color::srgb(0.77, 0.24, 0.26),
            Self::Pickup => Color::srgb(0.97, 0.89, 0.44),
            Self::Hazard => Color::srgb(0.86, 0.47, 0.16),
            Self::Accent => Color::srgb(0.94, 0.86, 0.51),
            Self::UiText => Color::srgb(0.95, 0.95, 0.90),
        }
    }
}

pub fn rectangle_mesh(meshes: &mut Assets<Mesh>, size: Vec2) -> Mesh2d {
    Mesh2d(meshes.add(Rectangle::new(size.x, size.y)))
}

pub fn circle_mesh(meshes: &mut Assets<Mesh>, radius: f32) -> Mesh2d {
    Mesh2d(meshes.add(Circle::new(radius)))
}

pub fn color_material(
    materials: &mut Assets<ColorMaterial>,
    color: WorldColor,
) -> MeshMaterial2d<ColorMaterial> {
    MeshMaterial2d(materials.add(color.color()))
}
