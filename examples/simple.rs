//! A shader and a material that uses it.

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::mesh::*,
};

use bevy_fake_interior::*;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Simple".into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                mode: AssetMode::Processed,
                ..default()
            }),
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin,
    ));

    app.add_plugins(FakeInteriorMaterialPlugin)
        .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
        .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
        .add_plugins(bevy_spectator::SpectatorPlugin)
        .add_systems(Startup, setup);

    app.run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut interiors: ResMut<Assets<StandardFakeInteriorMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, -0.5, 1.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    let _interior1 = MeshMaterial3d(interiors.add(StandardFakeInteriorMaterial {
        base: StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/rooms_depth.png")),
            emissive: LinearRgba::WHITE * 10.0,
            emissive_texture: Some(asset_server.load("textures/rooms_emit.png")),
            reflectance: 1.0,
            ..default()
        },
        extension: FakeInteriorMaterial {
            atlas_rooms: Vec2::new(3.0, 2.0),
            rooms: Vec2::new(6.0, 6.0),
            depth: 0.5,
            room_seed: 1.2,
            ..default()
        },
    }));
    let _test_room = MeshMaterial3d(interiors.add(StandardFakeInteriorMaterial {
        base: StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/test_room.png")),
            emissive: LinearRgba::WHITE * 10.0,
            emissive_texture: Some(asset_server.load("textures/test_room_E.png")),
            reflectance: 1.0,
            ..default()
        },
        extension: FakeInteriorMaterial {
            atlas_rooms: Vec2::new(1.0, 1.0),
            rooms: Vec2::new(6.0, 6.0),
            depth: 0.5,
            room_seed: 1.2,
            ..default()
        },
    }));
    let interior2 = MeshMaterial3d(interiors.add(StandardFakeInteriorMaterial {
        base: StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/room_3.png")),
            emissive: LinearRgba::WHITE * 10.0,
            emissive_texture: Some(asset_server.load("textures/room_3_E.png")),
            reflectance: 1.0,
            ..default()
        },
        extension: FakeInteriorMaterial {
            atlas_rooms: Vec2::new(1.0, 1.0),
            rooms: Vec2::new(6.0, 6.0),
            depth: 0.5,
            room_seed: 1.4,
            ..default()
        },
    }));
    let cube = Mesh3d(
        meshes.add(
            Mesh::from(Cuboid::from_length(1.0))
                .with_generated_tangents()
                .unwrap(),
        ),
    );
    // back cube
    commands.spawn((
        cube.clone(),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        //interior1.clone(),
        Transform::from_xyz(-1.0, 0.0, -1.0),
    ));
    // front cube
    commands.spawn((
        cube.clone(),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        //interior2.clone(),
        Transform::from_xyz(1.0, 0.0, 0.8),
    ));
    // light
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    let mesh = Mesh3d(
        meshes.add(
            PlaneMeshBuilder::from_length(1.0)
                .subdivisions(0)
                .build()
                .with_generated_tangents()
                .unwrap(),
        ),
    );
    // wall 1
    let mut wall = commands.spawn((
        mesh.clone(),
        Transform::from_xyz(-1.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(1.570796)),
        _test_room,
    ));
    wall.insert(Name::new("Wall 1"));
    // wall 2
    let mut wall = commands.spawn((
        mesh.clone(),
        Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_x(1.570796)),
        interior2,
    ));
    wall.insert(Name::new("Wall 2"));

    // camera
    let mut cam = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 0.0, 0.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        DistanceFog {
            color: Color::srgba(0.25, 0.25, 0.25, 1.0),
            falloff: FogFalloff::Linear {
                start: 5.0,
                end: 20.0,
            },
            ..default()
        },
    ));

    //cam.insert(bevy_spectator::Spectator);
    //*
    cam.insert(bevy_panorbit_camera::PanOrbitCamera {
        focus: Vec3::new(0.0, 0.0, 0.0),
        radius: Some(5.0),
        yaw: Some(0.00),
        pitch: Some(0.0),
        ..default()
    });
    // */
    cam.insert(Name::new("Camera"));
}
