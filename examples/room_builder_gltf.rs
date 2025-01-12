//! Room texture builder.

use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions,
    pbr::{NotShadowCaster, PointLightShadowMap},
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    render::view::screenshot::{save_to_disk, Screenshot},
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
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Room builder".into(),
                    resolution: (512., 512.).into(),
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

    app.add_plugins(MaterialPlugin::<PrepassOutputMaterial> {
        // This material only needs to read the prepass textures,
        // but the meshes using it should not contribute to the prepass render, so we can disable it.
        prepass_enabled: false,
        ..default()
    });
    app.add_plugins(FakeInteriorMaterialPlugin)
        .add_plugins(
            bevy_inspector_egui::quick::WorldInspectorPlugin::new()
                .run_if(common_conditions::input_toggle_active(false, KeyCode::KeyE)),
        )
        .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin);

    app.insert_resource(PointLightShadowMap { size: 4096 });

    app.add_systems(Startup, (setup, setup_room)).add_systems(
        Update,
        (
            handle_quit,
            toggle_prepass_view.run_if(common_conditions::input_just_pressed(KeyCode::KeyP)),
            screenshot_on_spacebar.run_if(common_conditions::input_just_pressed(KeyCode::Space)),
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
    material_handle: Query<&MeshMaterial3d<PrepassOutputMaterial>>,
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

fn screenshot_on_spacebar(mut commands: Commands, mut counter: Local<u32>) {
    let path = format!("./assets/textures/screenshot-{}.png", *counter);
    *counter += 1;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
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
    .spawn((
      Transform::from_xyz(0., 0., 0.),
      Visibility::default(),
    Name::new("Room")))
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
      let mesh = Mesh3d(meshes.add(mesh));
      let mut wall = commands.spawn((
          mesh,
          Transform::from_translation(WALL_BACK),
          //MeshMaterial3d(materials.add(Color::BLUE)),
          MeshMaterial3d(plaster_material.clone()),
      ));
      wall.insert(Name::new("Wall back"));

      // wall Left
      let mut mesh = Mesh::from(Cuboid::new(WALL_THICKNESS, ROOM_SIZE.y, ROOM_SIZE.z));
      mesh.generate_tangents().unwrap();
      let mesh = Mesh3d(meshes.add(mesh));
      let mut wall = commands.spawn((
          mesh.clone(),
          Transform::from_translation(WALL_LEFT),
          //MeshMaterial3d(materials.add(Color::RED)),
          MeshMaterial3d(plaster_material.clone()),
      ));
      wall.insert(Name::new("Wall left"));

      // wall Right
      let mut wall = commands.spawn((
          mesh,
          Transform::from_translation(WALL_RIGHT),
          //MeshMaterial3d(materials.add(Color::RED)),
          MeshMaterial3d(plaster_material.clone()),
      ));
      wall.insert(Name::new("Wall right"));

      // wall Floor
      let mut mesh = Mesh::from(Cuboid::new(ROOM_SIZE.x, WALL_THICKNESS, ROOM_SIZE.z));
      mesh.generate_tangents().unwrap();
      let mesh = Mesh3d(meshes.add(mesh));
      let mut wall = commands.spawn((
          mesh.clone(),
          Transform::from_translation(WALL_FLOOR),
          MeshMaterial3d(carpet_material.clone()),
      ));
      wall.insert(Name::new("Wall floor"));

      // wall Ceiling
      let mut wall = commands.spawn((
          mesh,
          Transform::from_translation(WALL_CEILING),
          //MeshMaterial3d(materials.add(Color::GREEN)),
          MeshMaterial3d(plaster_material.clone()),
      ));
      wall.insert(Name::new("Wall ceiling"));

      // GLTF objects.
      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/modern_arm_chair/modern_arm_chair_01_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(-3.3, -5.0, -3.0))
            .with_rotation(Quat::from_rotation_y(0.9))
            .with_scale(Vec3::new(2.5, 2.5, 2.5)),
      Name::new("Arm chair")));
      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/modern_wooden_cabinet/modern_wooden_cabinet_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(4.7, -5.0, 0.9))
            .with_rotation(Quat::from_rotation_y(-90.0_f32.to_radians()))
            .with_scale(Vec3::new(2.5, 1.6, 1.1)),
      Name::new("Wooden cabinet")));
      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/modern_coffee_table/modern_coffee_table_01_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(-1.4, -5.0, 1.9))
            .with_scale(Vec3::new(2.5, 1.6, 2.5)),
      Name::new("Coffee table")));

      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/modern_ceiling_lamp/modern_ceiling_lamp_01_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(0.0, 3.47, 0.0))
            .with_scale(Vec3::new(2.5, 1.3, 2.5)),
      Name::new("Modern ceiling lamp")));

      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/hanging_picture_frame_01/hanging_picture_frame_01_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(-5.0, 0.8, 1.9))
            .with_rotation(Quat::from_rotation_y(90.0_f32.to_radians()))
            .with_scale(Vec3::new(4.0, 4.0, 4.0)),
      Name::new("Hanging picture frame 01")));
      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/hanging_picture_frame_02/hanging_picture_frame_02_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(5.0, 0.5, 1.9))
            .with_rotation(Quat::from_rotation_y(-90.0_f32.to_radians()))
            .with_scale(Vec3::new(4.0, 4.0, 4.0)),
      Name::new("Hanging picture frame 02")));
      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/fancy_picture_frame/fancy_picture_frame_01_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(2.6, 2.0, -4.9))
            .with_scale(Vec3::new(4.5, 4.8, 4.5)),
      Name::new("Fancy picture frame 01")));

      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/dartboard/dartboard_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(-0.6, 0.7, -5.0))
            .with_scale(Vec3::new(3.0, 3.0, 3.0)),
      Name::new("Dartboard")));

      commands.spawn((
          SceneRoot(asset_server.load("polyhaven.com/wooden_bookshelf_worn/wooden_bookshelf_worn_1k.gltf#Scene0")),
          Transform::from_translation(Vec3::new(2.8, -5.0, -4.8))
            .with_scale(Vec3::new(2.5, 2.5, 2.5)),
      Name::new("Bookshelf")))
      .with_children(|commands| {
        let books = asset_server.load("polyhaven.com/book_encyclopedia_set/book_encyclopedia_set_01_1k.gltf#Scene0");
        commands.spawn((
            SceneRoot(books.clone()),
            Transform::from_translation(Vec3::new(-0.54, 1.65, 0.1))
              //.with_rotation(Quat::from_rotation_y(1.5))
              .with_scale(Vec3::new(2.1, 1.4, 1.0)),
        Name::new("Encyclopedia set")));
      });

      // light
      commands.spawn((
          PointLight {
              intensity: 100000.0,
              shadows_enabled: true,
              ..default()
          },
          Transform::from_xyz(0.0, 3.47, 0.0),
      ));
    });
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
    mut depth_materials: ResMut<Assets<PrepassOutputMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    /*
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
        //parallax_depth_scale: 0.09,
        //max_parallax_layer_count: 32.0,
        ..default()
    });
    // wall back
    let mut mesh = Mesh::from(Cuboid::new(ROOM_SIZE.x, ROOM_SIZE.y, WALL_THICKNESS));
    mesh.generate_tangents().unwrap();
    let mesh = Mesh3d(meshes.add(mesh));
    let mut wall = commands.spawn((
        mesh,
        Transform::from_translation(WALL_BACK),
        //MeshMaterial3d(materials.add(Color::BLUE)),
        MeshMaterial3d(plaster_material.clone()),
    ));
    wall.insert(Name::new("Wall back"));

    // wall Left
    let mut mesh = Mesh::from(Cuboid::new(WALL_THICKNESS, ROOM_SIZE.y, ROOM_SIZE.z));
    mesh.generate_tangents().unwrap();
    let mesh = Mesh3d(meshes.add(mesh));
    let mut wall = commands.spawn((
        Mesh3d(mesh.clone()),
        Transform::from_translation(WALL_LEFT),
        //MeshMaterial3d(materials.add(Color::RED)),
        MeshMaterial3d(plaster_material.clone()),
    ));
    wall.insert(Name::new("Wall left"));

    // wall Right
    let mut wall = commands.spawn((
        mesh,
        Transform::from_translation(WALL_RIGHT),
        //MeshMaterial3d(materials.add(Color::RED)),
        MeshMaterial3d(plaster_material.clone()),
    ));
    wall.insert(Name::new("Wall right"));

    // wall Floor
    let mut mesh = Mesh::from(Cuboid::new(ROOM_SIZE.x, WALL_THICKNESS, ROOM_SIZE.z));
    mesh.generate_tangents().unwrap();
    let mesh = Mesh3d(meshes.add(mesh));
    let mut wall = commands.spawn((
        Mesh3d(mesh.clone()),
        Transform::from_translation(WALL_FLOOR),
        MeshMaterial3d(carpet_material.clone()),
    ));
    wall.insert(Name::new("Wall floor"));

    // wall Ceiling
    let mut wall = commands.spawn((
        mesh,
        Transform::from_translation(WALL_CEILING),
        //MeshMaterial3d(materials.add(Color::GREEN)),
        MeshMaterial3d(plaster_material.clone()),
    ));
    wall.insert(Name::new("Wall ceiling"));

    // GLTF objects.
    commands.spawn((
        SceneRoot(asset_server.load("polyhaven.com/modern_arm_chair/modern_arm_chair_01_1k.gltf#Scene0")),
        Transform::from_translation(Vec3::new(-3.3, -5.0, -3.0))
          .with_rotation(Quat::from_rotation_y(0.9))
          .with_scale(Vec3::new(2.5, 2.5, 2.5)),
    Name::new("Arm chair")));
    commands.spawn((
        SceneRoot(asset_server.load("polyhaven.com/modern_wooden_cabinet/modern_wooden_cabinet_1k.gltf#Scene0")),
        Transform::from_translation(Vec3::new(4.4, -5.0, 0.9))
          .with_rotation(Quat::from_rotation_y(-90.0_f32.to_radians()))
          .with_scale(Vec3::new(2.5, 2.5, 2.5)),
    Name::new("Wooden cabinet")));
    commands.spawn((
        SceneRoot(asset_server.load("polyhaven.com/modern_coffee_table/modern_coffee_table_01_1k.gltf#Scene0")),
        Transform::from_translation(Vec3::new(-4.3, -5.0, 1.9))
          //.with_rotation(Quat::from_rotation_y(1.5))
          .with_scale(Vec3::new(2.5, 2.5, 2.5)),
    Name::new("Coffee table")));
    commands.spawn((
        SceneRoot(asset_server.load("polyhaven.com/wooden_bookshelf_worn/wooden_bookshelf_worn_1k.gltf#Scene0")),
        Transform::from_translation(Vec3::new(2.8, -5.0, -4.8))
          .with_scale(Vec3::new(2.5, 2.5, 2.5)),
    Name::new("Bookshelf")));

    // light
    commands.spawn((
        PointLight {
            intensity: 100000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, (ROOM_SIZE.y/2.0) - 0.3, 0.0),
    ));
    */

    // camera
    let mut cam = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 10.0),
        Projection::Perspective(PerspectiveProjection {
            fov: 53.13_f32.to_radians(),
            ..default()
        }),
    ));
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
    });
    // */
    cam.insert(Name::new("Camera"));

    // A quad that shows the outputs of the prepass
    // To make it easy, we just draw a big quad right in front of the camera.
    // For a real application, this isn't ideal.
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(20.0, 20.0))),
        MeshMaterial3d(depth_materials.add(PrepassOutputMaterial {
            settings: ShowPrepassSettings::default(),
        })),
        Transform::from_xyz(0., 0., 5.0),
        NotShadowCaster,
    ));
}
