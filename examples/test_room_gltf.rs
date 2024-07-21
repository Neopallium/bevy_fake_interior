//! A shader and a material that uses it.

use std::fmt;

use bevy::{
  prelude::*,
  diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
  window::PresentMode,
  pbr::PointLightShadowMap,
  input::common_conditions,
  core_pipeline::prepass::{DepthPrepass, NormalPrepass},
  pbr::NotShadowCaster,
  reflect::TypePath,
  render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
  render::mesh::*,
};

use bevy_fake_interior::*;

const ROOM_SIZE: Vec3 = Vec3::new(10., 10., 10.);
const WALL_BACK: Vec3 = Vec3::new(0., 0., -5.);
const WALL_LEFT: Vec3 = Vec3::new(-5., 0., 0.);
const WALL_RIGHT: Vec3 = Vec3::new(5., 0., 0.);
const WALL_FLOOR: Vec3 = Vec3::new(0., -5., 0.);
const WALL_CEILING: Vec3 = Vec3::new(0., 5., 0.);
const WALL_THICKNESS: f32 = 0.01;

fn main() {
  let mut app = App::new();

  app.add_plugins((
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Test room".into(),
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
    .add_systems(Startup, (setup, setup_room))
    .add_systems(
        Update,
        (
            handle_quit,
            toggle_prepass_view.run_if(common_conditions::input_just_pressed(KeyCode::KeyP)),
        ),
    );

  app.run();
}

fn handle_quit(input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
  if input.pressed(KeyCode::KeyQ) {
    exit.send(AppExit::Success);
  }
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
fn setup_room(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
  // top-level room entity.
  commands
    .spawn((SpatialBundle {
      transform: Transform::from_xyz(0., 0., 0.),
      ..default()
    }, Name::new("Room")))
    .with_children(|commands| {
      // Dirty carpet.
      let carpet_material = materials.add(StandardMaterial {
          base_color_texture: Some(asset_server.load("polyhaven.com/dirty_carpet/textures/dirty_carpet_diff_1k.jpg")),
          normal_map_texture: Some(asset_server.load("polyhaven.com/dirty_carpet/textures/dirty_carpet_nor_gl_1k.jpg")),
          metallic: 0.,
          metallic_roughness_texture: Some(asset_server.load("polyhaven.com/dirty_carpet/textures/dirty_carpet_arm_1k.jpg")),
          ..default()
      });
      // Plaster carpet.
      let plaster_material = materials.add(StandardMaterial {
          base_color_texture: Some(asset_server.load("polyhaven.com/plastered_wall/textures/plastered_wall_diff_1k.jpg")),
          normal_map_texture: Some(asset_server.load("polyhaven.com/plastered_wall/textures/plastered_wall_nor_gl_1k.jpg")),
          metallic: 0.,
          metallic_roughness_texture: Some(asset_server.load("polyhaven.com/plastered_wall/textures/plastered_wall_arm_1k.jpg")),
          //depth_map: Some(asset_server.load("polyhaven.com/plastered_wall/textures/plastered_wall_disp_1k.jpg")),
          ..default()
      });
      // wall back
      let mut mesh = Mesh::from(Cuboid::new(ROOM_SIZE.x, ROOM_SIZE.y, WALL_THICKNESS));
      mesh.generate_tangents().unwrap();
      let mesh = meshes.add(mesh);
      let mut wall = commands.spawn(MaterialMeshBundle {
          mesh,
          transform: Transform::from_translation(WALL_BACK),
          //material: materials.add(Color::BLUE),
          material: plaster_material.clone(),
          ..default()
      });
      wall.insert(Name::new("Wall back"));

      // wall Left
      let mut mesh = Mesh::from(Cuboid::new(WALL_THICKNESS, ROOM_SIZE.y, ROOM_SIZE.z));
      mesh.generate_tangents().unwrap();
      let mesh = meshes.add(mesh);
      let mut wall = commands.spawn(MaterialMeshBundle {
          mesh: mesh.clone(),
          transform: Transform::from_translation(WALL_LEFT),
          //material: materials.add(Color::RED),
          material: plaster_material.clone(),
          ..default()
      });
      wall.insert(Name::new("Wall left"));

      // wall Right
      let mut wall = commands.spawn(MaterialMeshBundle {
          mesh,
          transform: Transform::from_translation(WALL_RIGHT),
          //material: materials.add(Color::RED),
          material: plaster_material.clone(),
          ..default()
      });
      wall.insert(Name::new("Wall right"));

      // wall Floor
      let mut mesh = Mesh::from(Cuboid::new(ROOM_SIZE.x, WALL_THICKNESS, ROOM_SIZE.z));
      mesh.generate_tangents().unwrap();
      let mesh = meshes.add(mesh);
      let mut wall = commands.spawn(MaterialMeshBundle {
          mesh: mesh.clone(),
          transform: Transform::from_translation(WALL_FLOOR),
          material: carpet_material.clone(),
          ..default()
      });
      wall.insert(Name::new("Wall floor"));

      // wall Ceiling
      let mut wall = commands.spawn(MaterialMeshBundle {
          mesh,
          transform: Transform::from_translation(WALL_CEILING),
          //material: materials.add(Color::GREEN),
          material: plaster_material.clone(),
          ..default()
      });
      wall.insert(Name::new("Wall ceiling"));

      // GLTF objects.
      commands.spawn((SceneBundle {
          scene: asset_server.load("polyhaven.com/modern_arm_chair/modern_arm_chair_01_1k.gltf#Scene0"),
          transform: Transform::from_translation(Vec3::new(-3.3, -5.0, -3.0))
            .with_rotation(Quat::from_rotation_y(0.9))
            .with_scale(Vec3::new(2.5, 2.5, 2.5)),
          ..default()
      }, Name::new("Arm chair")));
      commands.spawn((SceneBundle {
          scene: asset_server.load("polyhaven.com/modern_wooden_cabinet/modern_wooden_cabinet_1k.gltf#Scene0"),
          transform: Transform::from_translation(Vec3::new(4.4, -5.0, 0.9))
            .with_rotation(Quat::from_rotation_y(-90.0_f32.to_radians()))
            .with_scale(Vec3::new(2.5, 2.5, 2.5)),
          ..default()
      }, Name::new("Wooden cabinet")));
      commands.spawn((SceneBundle {
          scene: asset_server.load("polyhaven.com/modern_coffee_table/modern_coffee_table_01_1k.gltf#Scene0"),
          transform: Transform::from_translation(Vec3::new(-4.3, -5.0, 1.9))
            //.with_rotation(Quat::from_rotation_y(1.5))
            .with_scale(Vec3::new(2.5, 2.5, 2.5)),
          ..default()
      }, Name::new("Coffee table")));
      commands.spawn((SceneBundle {
          scene: asset_server.load("polyhaven.com/wooden_bookshelf_worn/wooden_bookshelf_worn_1k.gltf#Scene0"),
          transform: Transform::from_translation(Vec3::new(2.8, -5.0, -4.8))
            .with_scale(Vec3::new(2.5, 2.5, 2.5)),
          ..default()
      }, Name::new("Bookshelf")));

      // light
      commands.spawn(PointLightBundle {
          point_light: PointLight {
              intensity: 100000.0,
              shadows_enabled: true,
              ..default()
          },
          transform: Transform::from_xyz(0.0, (ROOM_SIZE.y/2.0) - 0.3, 0.0),
          ..default()
      });
    });
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
    /*
    commands
        .spawn((PointLightBundle {
            transform: Transform::from_xyz(0.8, 2.0, -1.1),
            point_light: PointLight {
                intensity: 226.0,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        }, Name::new("Light 1")))
        .with_children(|commands| {
            // represent the light source as a sphere
            let mesh = meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 0.05,
                    subdivisions: 3,
                }).expect("Icosphere mesh"),
            );
            commands.spawn(PbrBundle { mesh, ..default() });
        });

    // light
    commands.spawn((PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    }, Name::new("Light 2")));
    // */

    // light
    commands.spawn((PointLightBundle {
        point_light: PointLight {
            intensity: 1000000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-5.0, (ROOM_SIZE.y/2.0) + 0.5, 6.5),
        ..default()
    }, Name::new("Spot Light")));

    // circular base
    commands.spawn((PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
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
    let interior1 = interiors.add(StandardFakeInteriorMaterial {
      base: StandardMaterial {
        base_color_texture: Some(asset_server.load("textures/room_gltf.png")),
        emissive: LinearRgba::WHITE * 15.0,
        emissive_texture: Some(asset_server.load("textures/room_gltf_E.png")),
        reflectance: 0.2,
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
    let interior_n = interiors.add(StandardFakeInteriorMaterial {
      base: StandardMaterial {
        //perceptual_roughness: 0.4,
        base_color_texture: Some(asset_server.load("textures/room_gltf.png")),
        emissive: LinearRgba::WHITE * 15.0,
        emissive_texture: Some(asset_server.load("textures/room_gltf_E.png")),
        normal_map_texture: Some(asset_server.load("textures/room_gltf_normal.png")),
        reflectance: 0.2,
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
    let interior_d = interiors.add(StandardFakeInteriorMaterial {
      base: StandardMaterial {
        //perceptual_roughness: 0.4,
        base_color_texture: Some(asset_server.load("textures/room_gltf.png")),
        emissive: LinearRgba::WHITE * 15.0,
        emissive_texture: Some(asset_server.load("textures/room_gltf_E.png")),
        normal_map_texture: Some(asset_server.load("textures/room_gltf_normal.png")),
        depth_map: Some(asset_server.load("textures/room_gltf_depth.png")),
        reflectance: 0.2,
        max_parallax_layer_count: 0.0,
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

    let mesh = meshes.add(PlaneMeshBuilder::from_length(10.0).subdivisions(0).build()
      .with_generated_tangents().unwrap());
    // wall 1
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(-20.0, 0.0, 5.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: interior1,
        ..default()
    });
    wall.insert(Name::new("Wall 1"));
    // wall 2
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(-10.0, 0.0, 5.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: interior_n.clone(),
        ..default()
    });
    wall.insert(Name::new("Wall 2"));
    // wall 3
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(10.0, 0.0, 5.0)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: interior_d,
        ..default()
    });
    wall.insert(Name::new("Wall 3"));
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
    // window 4
    let mut window = commands.spawn(MaterialMeshBundle {
        mesh: mesh.clone(),
        transform: Transform::from_xyz(10., 0.0, 5.001)
          .with_rotation(Quat::from_rotation_x(1.570796)),
        material: simple_window,
        ..default()
    });
    window.insert(Name::new("Window 4"));
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
            end: 20.0,
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
              mesh: meshes.add(Rectangle::new(20.0, 20.0)),
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
