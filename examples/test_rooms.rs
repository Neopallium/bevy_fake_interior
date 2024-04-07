//! A shader and a material that uses it.

use std::fmt;

use bevy::{
  prelude::*, window::close_on_esc,
  diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
  window::PresentMode,
  pbr::PointLightShadowMap,
  input::common_conditions,
  core_pipeline::prepass::{DepthPrepass, NormalPrepass},
  pbr::NotShadowCaster,
  reflect::TypePath,
  render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

use bevy_fake_interior::*;

fn main() {
  let mut app = App::new();

  app.add_plugins((
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Test rooms".into(),
            //present_mode: PresentMode::AutoNoVsync,
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
    .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
    .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin);

  app.insert_resource(PointLightShadowMap { size: 4096 });
  app
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            close_on_esc,
            toggle_prepass_view.run_if(common_conditions::input_just_pressed(KeyCode::KeyP)),
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

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut interiors: ResMut<Assets<StandardFakeInteriorMaterial>>,
    mut depth_materials: ResMut<Assets<PrepassOutputMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // light
    commands.spawn((PointLightBundle {
        point_light: PointLight {
            intensity: 1000000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-5.0, 6.0, 6.5),
        ..default()
    }, Name::new("Spot Light")));

    // circular base
    commands.spawn((PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_xyz(0.0, -5.0, 0.0)
          .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
          .with_scale(Vec3::new(8., 8., 8.)),
        ..default()
    }, Name::new("Ground")));

    let simple_window = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/window_glass_texture.png")),
        reflectance: 1.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
      });
    let room_01 = interiors.add(StandardFakeInteriorMaterial {
      base: StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/room_gltf.png")),
        emissive: Color::WHITE * 15.0,
        emissive_texture: Some(asset_server.load("textures/room_gltf_E.png")),
        reflectance: 0.2,
        normal_map_texture: Some(asset_server.load("textures/room_gltf_normal.png")),
        //depth_map: Some(asset_server.load("textures/room_gltf_depth.png")),
        //max_parallax_layer_count: 0.0,
        ..default()
      },
      extension: FakeInteriorMaterial {
        atlas_rooms: Vec2::new(1.0, 1.0),
        rooms: Vec2::new(1.0, 1.0),
        depth: 0.5,
        room_seed: 1.2,
        ..default()
      }
    });
    let room_02 = interiors.add(StandardFakeInteriorMaterial {
      base: StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/room_gltf_02.png")),
        emissive: Color::WHITE * 15.0,
        emissive_texture: Some(asset_server.load("textures/room_gltf_02_E.png")),
        reflectance: 0.2,
        normal_map_texture: Some(asset_server.load("textures/room_gltf_02_normal.png")),
        //depth_map: Some(asset_server.load("textures/room_gltf_02_depth.png")),
        //max_parallax_layer_count: 0.0,
        ..default()
      },
      extension: FakeInteriorMaterial {
        atlas_rooms: Vec2::new(1.0, 1.0),
        rooms: Vec2::new(1.0, 1.0),
        depth: 0.5,
        room_seed: 1.2,
        ..default()
      }
    });
    let room_03 = interiors.add(StandardFakeInteriorMaterial {
      base: StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/room_gltf_02.png")),
        emissive: Color::WHITE * 15.0,
        emissive_texture: Some(asset_server.load("textures/room_gltf_02_E.png")),
        reflectance: 0.2,
        //normal_map_texture: Some(asset_server.load("textures/room_gltf_02_normal.png")),
        //depth_map: Some(asset_server.load("textures/room_gltf_02_depth.png")),
        //max_parallax_layer_count: 0.0,
        ..default()
      },
      extension: FakeInteriorMaterial {
        atlas_rooms: Vec2::new(1.0, 1.0),
        rooms: Vec2::new(1.0, 1.0),
        depth: 0.5,
        room_seed: 1.2,
        ..default()
      }
    });

    let mesh = meshes.add(Mesh::from(shape::Plane { size: 10.0, subdivisions: 0 })
      .with_generated_tangents().unwrap());
    // wall 1
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(-20.0, 0.0, 5.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: room_01.clone(),
        ..default()
    });
    wall.insert(Name::new("Room 01"));
    // wall 2
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(-10.0, 0.0, 5.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: room_02.clone(),
        ..default()
    });
    wall.insert(Name::new("Room 02"));
    // wall 3
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 5.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: room_03.clone(),
        ..default()
    });
    wall.insert(Name::new("Room 03"));
    /*
    // window 1
    let mut window = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(-20., 0.0, 5.001)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: simple_window.clone(),
        ..default()
    });
    window.insert(Name::new("Window 1"));
    // window 2
    let mut window = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(-10., 0.0, 5.001)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: simple_window.clone(),
        ..default()
    });
    window.insert(Name::new("Window 2"));
    // window 3
    let mut window = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(0., 0.0, 5.001)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: simple_window.clone(),
        ..default()
    });
    window.insert(Name::new("Window 3"));
    // */

    // camera
    let mut cam = commands.spawn(Camera3dBundle {
      transform: Transform::from_xyz(5.0, 0.0, 0.0)
        .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
      ..default()
    });

    /*
    cam.insert(FogSettings {
        color: Color::rgba(0.25, 0.25, 0.25, 1.0),
        falloff: FogFalloff::Linear {
            start: 5.0,
            end: 200.0,
        },
        ..default()
    });
    // */

    //*
    cam.insert(bevy_panorbit_camera::PanOrbitCamera {
        focus: Vec3::new(0.0, 0.0, 0.0),
        radius: Some(25.0),
        yaw: Some(0.00),
        pitch: Some(0.0),
        ..default()
      },
    );
    // */
    cam.insert((
        // To enable the prepass you need to add the components associated with the ones you need
        // This will write the depth buffer to a texture that you can use in the main pass
        DepthPrepass,
        // This will generate a texture containing world normals (with normal maps applied)
        NormalPrepass,
    ));
    cam.insert(Name::new("Camera"));

    cam.with_children(|commands| {
      // A quad that shows the outputs of the prepass
      // To make it easy, we just draw a big quad right in front of the camera.
      // For a real application, this isn't ideal.
      commands.spawn((
          MaterialMeshBundle {
              mesh: meshes.add(shape::Quad::new(Vec2::new(20.0, 20.0))),
              material: depth_materials.add(PrepassOutputMaterial {
                  settings: ShowPrepassSettings::default(),
              }),
              transform: Transform::from_xyz(0., 0., -0.5),
              ..default()
          },
          NotShadowCaster,
      ));
    });
}
