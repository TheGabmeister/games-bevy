use bevy::prelude::*;

use crate::components::Label;

const LABEL_FONT_SIZE: f32 = 7.0;
const LABEL_Y_OFFSET: f32 = -12.0;

pub struct PrimitiveRenderingPlugin;

impl Plugin for PrimitiveRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, spawn_label_text);
    }
}

fn spawn_label_text(
    mut commands: Commands,
    new_labels: Query<(Entity, &Label), Added<Label>>,
) {
    for (entity, label) in &new_labels {
        commands.entity(entity).with_children(|child| {
            child.spawn((
                Text2d::new(label.0.clone()),
                TextFont {
                    font_size: LABEL_FONT_SIZE,
                    ..default()
                },
                TextColor(WorldColor::UiText.color()),
                Transform::from_xyz(0.0, LABEL_Y_OFFSET, 1.0),
            ));
        });
    }
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
