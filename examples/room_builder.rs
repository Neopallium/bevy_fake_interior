//! Room texture builder.

use bevy::{
  prelude::*, window::close_on_esc,
  diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
  window::PrimaryWindow,
  render::view::screenshot::ScreenshotManager,
  input::common_conditions,
  core_pipeline::prepass::{DepthPrepass, NormalPrepass},
  pbr::NotShadowCaster,
  reflect::TypePath,
  render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

use bevy_fake_interior::*;

const ROOM_SIZE: Vec3 = Vec3::new(10., 10., 60.);
const WALL_BACK: Vec3 = Vec3::new(0., 0., -5.);
const WALL_LEFT: Vec3 = Vec3::new(-5., 0., 25.);
const WALL_RIGHT: Vec3 = Vec3::new(5., 0., 25.);
const WALL_FLOOR: Vec3 = Vec3::new(0., -5., 25.);
const WALL_CEILING: Vec3 = Vec3::new(0., 5., 25.);
const WALL_THICKNESS: f32 = 0.01;

fn main() {
  let mut app = App::new();

  app.add_plugins((
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Room builder".into(),
            resolution: (512., 512.).into(),
            ..default()
        }),
        ..default()
    }).set(
      AssetPlugin {
        mode: AssetMode::Processed,
        ..default()
      }
    ),
    LogDiagnosticsPlugin::default(),
    FrameTimeDiagnosticsPlugin,
  ));

  app.add_plugins(
      MaterialPlugin::<PrepassOutputMaterial> {
          // This material only needs to read the prepass textures,
          // but the meshes using it should not contribute to the prepass render, so we can disable it.
          prepass_enabled: false,
          ..default()
      },
    );
  app.add_plugins(FakeInteriorMaterialPlugin)
    .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new()
      .run_if(common_conditions::input_toggle_active(false, KeyCode::KeyE))
    )
    .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            close_on_esc,
            toggle_prepass_view.run_if(common_conditions::input_just_pressed(KeyCode::KeyP)),
            screenshot_on_spacebar.run_if(common_conditions::input_just_pressed(KeyCode::Space)),
        ),
    );

  app.run();
}

#[derive(Debug, Clone, Default, ShaderType)]
struct ShowPrepassSettings {
    show_depth: u32,
    show_normals: u32,
}

// This shader simply loads the prepass texture and outputs it directly
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PrepassOutputMaterial {
    #[uniform(0)]
    settings: ShowPrepassSettings,
}

impl Material for PrepassOutputMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/show_prepass.wgsl".into()
    }

    // This needs to be transparent in order to show the scene behind the mesh
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Every time you press space, it will cycle between transparent, depth and normals view
fn toggle_prepass_view(
    mut prepass_view: Local<u32>,
    material_handle: Query<&Handle<PrepassOutputMaterial>>,
    mut materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    *prepass_view = (*prepass_view + 1) % 3;

    let label = match *prepass_view {
        0 => "transparent",
        1 => "depth",
        2 => "normals",
        _ => unreachable!(),
    };
    eprintln!("Prepass Output: {label}");

    let handle = material_handle.single();
    let mat = materials.get_mut(handle).unwrap();
    mat.settings.show_depth = (*prepass_view == 1) as u32;
    mat.settings.show_normals = (*prepass_view == 2) as u32;
}

fn screenshot_on_spacebar(
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut counter: Local<u32>,
) {
    let path = format!("./assets/textures/screenshot-{}.png", *counter);
    *counter += 1;
    screenshot_manager
        .save_screenshot_to_disk(main_window.single(), path)
        .unwrap();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut depth_materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    // wall back
    let mut mesh = Mesh::from(shape::Box::new(ROOM_SIZE.x, ROOM_SIZE.y, WALL_THICKNESS));
    mesh.generate_tangents().unwrap();
    let mesh = meshes.add(mesh);
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh,
        transform: Transform::from_translation(WALL_BACK),
        material: materials.add(Color::BLUE),
        ..default()
    });
    wall.insert(Name::new("Wall back"));

    // back wall door.
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::SILVER),
        transform: Transform::from_translation(Vec3::new(-0.8, -1.5, -5.0))
          .with_scale(Vec3::new(3.0, 7.0, 0.5)),
        ..default()
    }, Name::new("Back door")));

    // back wall left corner table
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::GOLD),
        transform: Transform::from_translation(Vec3::new(-3.9, -4.0, -3.2))
          .with_scale(Vec3::new(1.7, 1.7, 1.8)),
        ..default()
    }, Name::new("Back left corner table")));

    // back wall bookcase
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::OLIVE),
        transform: Transform::from_translation(Vec3::new(2.7, -3.0, -4.3))
          .with_scale(Vec3::new(3.1, 4.2, 2.0)),
        ..default()
    }, Name::new("Back bookcase")));

    // left side sofa.
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::DARK_GREEN),
        transform: Transform::from_translation(Vec3::new(-4.0, -4.6, 1.1))
          .with_scale(Vec3::new(1.7, 1.2, 6.0)),
        ..default()
    }, Name::new("Left Sofa")))
      .with_children(|commands| {
        commands.spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::DARK_GREEN),
            transform: Transform::from_translation(Vec3::new(-0.4, 1.0, 0.0))
              .with_scale(Vec3::new(0.2, 1.0, 1.0)),
            ..default()
        }, Name::new("Sofa Back")));
      });

    // Coffee table.
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::WHITE),
        transform: Transform::from_translation(Vec3::new(-1.3, -4.5, 1.1))
          .with_scale(Vec3::new(1.5, 0.8, 5.0)),
        ..default()
    }, Name::new("Coffee table")));

    // TV.
    commands.spawn((PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::BLACK),
        transform: Transform::from_translation(Vec3::new(4.8, -1.5, 1.1))
          .with_scale(Vec3::new(0.2, 3.0, 5.0)),
        ..default()
    }, Name::new("TV")));

    // wall Left
    let mut mesh = Mesh::from(shape::Box::new(WALL_THICKNESS, ROOM_SIZE.y, ROOM_SIZE.z));
    mesh.generate_tangents().unwrap();
    let mesh = meshes.add(mesh);
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_translation(WALL_LEFT),
        material: materials.add(Color::RED),
        ..default()
    });
    wall.insert(Name::new("Wall left"));

    // wall Right
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh,
        transform: Transform::from_translation(WALL_RIGHT),
        material: materials.add(Color::RED),
        ..default()
    });
    wall.insert(Name::new("Wall right"));

    // wall Floor
    let mut mesh = Mesh::from(shape::Box::new(ROOM_SIZE.x, WALL_THICKNESS, ROOM_SIZE.z));
    mesh.generate_tangents().unwrap();
    let mesh = meshes.add(mesh);
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_translation(WALL_FLOOR),
        material: materials.add(Color::GREEN),
        ..default()
    });
    wall.insert(Name::new("Wall floor"));

    // wall Ceiling
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh,
        transform: Transform::from_translation(WALL_CEILING),
        material: materials.add(Color::GREEN),
        ..default()
    });
    wall.insert(Name::new("Wall ceiling"));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, (ROOM_SIZE.y/2.0) - 2.0, 0.0),
        ..default()
    });
    // camera
    let mut cam = commands.spawn(Camera3dBundle {
      transform: Transform::from_xyz(0.0, 0.0, 10.0),
      projection: Projection::Perspective(PerspectiveProjection {
        fov: 53.13_f32.to_radians(),
        ..default()
      }),
      ..default()
    });
    cam.insert((
        // To enable the prepass you need to add the components associated with the ones you need
        // This will write the depth buffer to a texture that you can use in the main pass
        DepthPrepass,
        // This will generate a texture containing world normals (with normal maps applied)
        NormalPrepass,
    ));
    //*
    cam.insert(bevy_panorbit_camera::PanOrbitCamera {
        focus: Vec3::new(0.0, 0.0, 0.0),
        radius: Some(15.0),
        yaw: Some(0.00),
        pitch: Some(0.0),
        ..default()
      },
    );
    // */

    cam.insert(Name::new("Camera"));

    // A quad that shows the outputs of the prepass
    // To make it easy, we just draw a big quad right in front of the camera.
    // For a real application, this isn't ideal.
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(shape::Quad::new(Vec2::new(20.0, 20.0))),
            material: depth_materials.add(PrepassOutputMaterial {
                settings: ShowPrepassSettings::default(),
            }),
            transform: Transform::from_xyz(0., 0., 9.0),
            ..default()
        },
        NotShadowCaster,
    ));
}
