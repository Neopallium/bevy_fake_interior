//! A shader and a material that uses it.

use bevy::{
  prelude::*,
};

mod material;
pub use material::*;

fn main() {
  let mut app = App::new();

  app.add_plugins(DefaultPlugins.set(
    AssetPlugin {
        mode: AssetMode::Processed,
        ..default()
    }
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_xyz(0.0, -0.5, 1.0)
          .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // back cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(-1.0, 0.0, -1.0),
        ..default()
    });
    // front cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(1.0, 0.0, 0.8),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let mut mesh = Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 });
    mesh.generate_tangents().unwrap();
    let mesh = meshes.add(mesh);
    // wall 1
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(-1.0, 0.0, 0.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: interiors.add(StandardFakeInteriorMaterial {
          base: StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/rooms_depth.png")),
            emissive: Color::WHITE,
            emissive_texture: Some(asset_server.load("textures/rooms_emit.png")),
            reflectance: 1.0,
            ..default()
          },
          extension: FakeInteriorMaterial {
            atlas_rooms: Vec2::new(3.0, 2.0),
            rooms: Vec2::new(4.0, 6.0),
            depth: 0.5,
            room_seed: 1.2,
            ..default()
          }
        }),
        ..default()
    });
    wall.insert(Name::new("Wall Atlas"));
    // wall 2
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: interiors.add(StandardFakeInteriorMaterial {
          base: StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/room_3.png")),
            emissive: Color::WHITE,
            emissive_texture: Some(asset_server.load("textures/room_3_E.png")),
            reflectance: 1.0,
            ..default()
          },
          extension: FakeInteriorMaterial {
            atlas_rooms: Vec2::new(1.0, 1.0),
            rooms: Vec2::new(4.0, 6.0),
            depth: 0.5,
            room_seed: 1.4,
            ..default()
          }
        }),
        ..default()
    });
    wall.insert(Name::new("Wall"));

    // camera
    let mut cam = commands.spawn((Camera3dBundle {
      transform: Transform::from_xyz(5.0, 0.0, 0.0)
        .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
      ..default()
    },FogSettings {
        color: Color::rgba(0.25, 0.25, 0.25, 1.0),
        falloff: FogFalloff::Linear {
            start: 5.0,
            end: 20.0,
        },
        ..default()
    },));

    //cam.insert(bevy_spectator::Spectator);
    //*
    cam.insert(bevy_panorbit_camera::PanOrbitCamera {
        focus: Vec3::new(0.0, 0.0, 0.0),
        radius: Some(5.0),
        alpha: Some(0.00),
        beta: Some(0.0),
        ..default()
      },
    );
    // */
    cam.insert(Name::new("Camera"));
}
