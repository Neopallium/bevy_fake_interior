//! A shader and a material that uses it.

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MaterialPlugin::<CustomMaterial>::default()))
        .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
        .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
        .add_plugins(bevy_spectator::SpectatorPlugin)
        .register_asset_reflect::<CustomMaterial>()
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut mesh = Mesh::from(shape::Plane { size: 1.0, subdivisions: 0 });
    //let mut mesh = Mesh::from(shape::Cube { size: 1.0 });
    //let mut mesh = Mesh::from(shape::Box::new(1.0, 1.0, 1.0));
    mesh.generate_tangents().unwrap();
    // wall
    let mut wall = commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(mesh),
        transform: Transform::from_rotation(Quat::from_rotation_x(1.570796)),
        material: materials.add(CustomMaterial {
            params: CustomUniform {
              atlas_rooms: Vec2::new(1.0, 1.0),
              rooms: Vec2::new(1.0, 1.0),
              depth: 0.5,
              ..default()
            },
            room_texture: Some(asset_server.load("textures/room_3_E.png")),
            //room_texture: Some(asset_server.load("textures/interior_2d.png")),
        }),
        ..default()
    });
    wall.insert(Name::new("Wall"));

    // camera
    let mut cam = commands.spawn(Camera3dBundle {
      transform: Transform::from_xyz(5.0, 0.0, 0.0)
        .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
      ..default()
    });

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

// The uniform parameters.
#[derive(Clone, Reflect, ShaderType)]
pub struct CustomUniform {
  atlas_rooms: Vec2,
  rooms: Vec2,
  depth: f32,
  room_seed: f32,
  emission_seed: f32,
  emission_threshold: f32,
}

impl Default for CustomUniform {
  fn default() -> Self {
    Self {
      atlas_rooms: Vec2::new(1.0, 1.0),
      rooms: Vec2::new(1.0, 1.0),
      depth: 0.5,
      room_seed: 1.0,
      emission_seed: 1.0,
      emission_threshold: 1.0,
    }
  }
}

// This is the struct that will be passed to your shader
#[derive(Asset, AsBindGroup, Reflect, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    params: CustomUniform,
    #[texture(1)]
    #[sampler(2)]
    room_texture: Option<Handle<Image>>,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
/// When using the GLSL shading language for your shader, the specialize method must be overridden.
impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}
