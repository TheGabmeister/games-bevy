use bevy::prelude::*;
use crate::components::*;
use crate::world::*;

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    world: Res<WorldMap>,
) {
    // Background color (will be updated by rooms system)
    commands.insert_resource(ClearColor(world.room(1).color));

    // ---- Player ----
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(12.0, 12.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.0))),
        Transform::from_xyz(0.0, 0.0, 3.0),
        Player,
        InRoom(1),
    ));

    // ---- Items ----
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::GoldKey,  5, 50.0,  50.0);
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::RedKey,  11, -100.0, 0.0);
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::BlueKey,  8,   0.0, 0.0);
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::Sword,   11, 100.0, 0.0);
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::Bridge,  12,   0.0, 0.0);
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::Chalice, 10,   0.0, 0.0);
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::Magnet,   3, 100.0, -50.0);
    spawn_item(&mut commands, &mut meshes, &mut materials, ItemKind::Dot,      8, 180.0,  52.0);

    // ---- Dragons ----
    spawn_dragon(&mut commands, &mut meshes, &mut materials, DragonKind::Yorgle,  5,  80.0,  80.0);
    spawn_dragon(&mut commands, &mut meshes, &mut materials, DragonKind::Grundle, 7, -80.0, -80.0);
    spawn_dragon(&mut commands, &mut meshes, &mut materials, DragonKind::Rhindle, 10,  0.0,  60.0);

    // ---- Bat ----
    let bat_mesh = meshes.add(RegularPolygon::new(10.0, 4));
    let bat_mat = materials.add(Color::srgb(0.2, 0.2, 0.2));
    commands.spawn((
        Mesh2d(bat_mesh),
        MeshMaterial2d(bat_mat),
        Transform::from_xyz(-100.0, 100.0, 4.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 4.0)),
        Bat,
        InRoom(11),
        BatData {
            held_item: None,
            wander_timer: Timer::from_seconds(0.4, TimerMode::Repeating),
            grab_timer: Timer::from_seconds(4.0, TimerMode::Repeating),
        },
    ));

    // ---- Gates ----
    // Room 0 south exit: gold gate
    spawn_gate(&mut commands, &mut meshes, &mut materials, 0, ExitDir::South, KeyColor::Gold);
    // Room 4 east exit: red gate
    spawn_gate(&mut commands, &mut meshes, &mut materials, 4, ExitDir::East, KeyColor::Red);
    // Room 9 south exit: blue gate
    spawn_gate(&mut commands, &mut meshes, &mut materials, 9, ExitDir::South, KeyColor::Blue);
}

fn spawn_item(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    kind: ItemKind,
    room: u8,
    x: f32,
    y: f32,
) {
    let (mesh, color) = item_visual(meshes, kind);
    commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(x, y, 2.0),
        Item,
        kind,
        InRoom(room),
    ));
}

fn item_visual(meshes: &mut ResMut<Assets<Mesh>>, kind: ItemKind) -> (Handle<Mesh>, Color) {
    match kind {
        ItemKind::GoldKey => (meshes.add(Rectangle::new(8.0, 16.0)), Color::srgb(1.0, 0.85, 0.0)),
        ItemKind::RedKey  => (meshes.add(Rectangle::new(8.0, 16.0)), Color::srgb(0.9, 0.1, 0.1)),
        ItemKind::BlueKey => (meshes.add(Rectangle::new(8.0, 16.0)), Color::srgb(0.2, 0.4, 1.0)),
        ItemKind::Sword   => (meshes.add(Rectangle::new(4.0, 28.0)), Color::srgb(0.9, 0.9, 0.9)),
        ItemKind::Bridge  => (meshes.add(Rectangle::new(48.0, 8.0)), Color::srgb(0.5, 0.35, 0.1)),
        ItemKind::Chalice => (meshes.add(Rectangle::new(12.0, 18.0)), Color::srgb(1.0, 0.9, 0.2)),
        ItemKind::Magnet  => (meshes.add(Rectangle::new(10.0, 14.0)), Color::srgb(0.6, 0.6, 0.7)),
        ItemKind::Dot     => (meshes.add(Rectangle::new(3.0, 3.0)),   Color::srgb(0.18, 0.18, 0.18)),
    }
}

fn spawn_dragon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    kind: DragonKind,
    room: u8,
    x: f32,
    y: f32,
) {
    let color = kind.color();
    let body_mesh = meshes.add(Rectangle::new(24.0, 20.0));
    let head_mesh = meshes.add(Triangle2d::new(
        Vec2::new(0.0, 10.0),
        Vec2::new(-7.0, -4.0),
        Vec2::new(7.0, -4.0),
    ));
    let body_mat = materials.add(color);
    let head_mat = materials.add(color);

    commands.spawn((
        Transform::from_xyz(x, y, 3.0),
        Visibility::Inherited,
        GlobalTransform::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Dragon,
        DragonData { kind, alive: true },
        DragonAlive,
        InRoom(room),
    )).with_children(|parent| {
        parent.spawn((
            Mesh2d(body_mesh),
            MeshMaterial2d(body_mat),
            Transform::default(),
            DragonBody,
        ));
        parent.spawn((
            Mesh2d(head_mesh),
            MeshMaterial2d(head_mat),
            Transform::from_xyz(0.0, 14.0, 0.1),
            DragonHead,
        ));
    });
}

fn gate_color(kc: KeyColor) -> Color {
    match kc {
        KeyColor::Gold => Color::srgb(0.9, 0.75, 0.0),
        KeyColor::Red  => Color::srgb(0.85, 0.1, 0.1),
        KeyColor::Blue => Color::srgb(0.1, 0.3, 0.9),
    }
}

fn spawn_gate(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    room: u8,
    exit_dir: ExitDir,
    key_color: KeyColor,
) {
    let half_w = ROOM_W / 2.0;
    let half_h = ROOM_H / 2.0;
    let half_t = WALL_T / 2.0;

    let (gx, gy, gw, gh) = match exit_dir {
        ExitDir::North => (0.0,  half_h - half_t, PASSAGE_W, WALL_T),
        ExitDir::South => (0.0, -(half_h - half_t), PASSAGE_W, WALL_T),
        ExitDir::East  => ( half_w - half_t, 0.0, WALL_T, PASSAGE_W),
        ExitDir::West  => (-(half_w - half_t), 0.0, WALL_T, PASSAGE_W),
    };

    let color = gate_color(key_color);
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(gw, gh))),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(gx, gy, 1.5),
        Gate,
        GateData { key_color, exit_dir, open: false },
        InRoom(room),
    ));
}
