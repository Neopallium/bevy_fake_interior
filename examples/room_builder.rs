//! Room texture builder.

use bevy::{
    color::palettes::css,
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions,
    pbr::NotShadowCaster,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    render::view::screenshot::{save_to_disk, Screenshot},
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
        .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_quit,
                toggle_prepass_view.run_if(common_conditions::input_just_pressed(KeyCode::KeyP)),
                screenshot_on_spacebar
                    .run_if(common_conditions::input_just_pressed(KeyCode::Space)),
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
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut depth_materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    // wall back
    let mut mesh = Mesh::from(Cuboid::new(ROOM_SIZE.x, ROOM_SIZE.y, WALL_THICKNESS));
    mesh.generate_tangents().unwrap();
    let mesh = Mesh3d(meshes.add(mesh));
    let mut wall = commands.spawn((
        mesh,
        Transform::from_translation(WALL_BACK),
        MeshMaterial3d(materials.add(Color::from(css::BLUE))),
    ));
    wall.insert(Name::new("Wall back"));

    let cube = Mesh3d(meshes.add(Cuboid::from_length(1.0)));
    // back wall door.
    commands.spawn((
        cube.clone(),
        MeshMaterial3d(materials.add(Color::from(css::SILVER))),
        Transform::from_translation(Vec3::new(-0.8, -1.5, -5.0))
            .with_scale(Vec3::new(3.0, 7.0, 0.5)),
        Name::new("Back door"),
    ));

    // back wall left corner table
    commands.spawn((
        cube.clone(),
        MeshMaterial3d(materials.add(Color::from(css::GOLD))),
        Transform::from_translation(Vec3::new(-3.9, -4.0, -3.2))
            .with_scale(Vec3::new(1.7, 1.7, 1.8)),
        Name::new("Back left corner table"),
    ));

    // back wall bookcase
    commands.spawn((
        cube.clone(),
        MeshMaterial3d(materials.add(Color::from(css::OLIVE))),
        Transform::from_translation(Vec3::new(2.7, -3.0, -4.3))
            .with_scale(Vec3::new(3.1, 4.2, 2.0)),
        Name::new("Back bookcase"),
    ));

    // left side sofa.
    commands
        .spawn((
            cube.clone(),
            MeshMaterial3d(materials.add(Color::from(css::DARK_GREEN))),
            Transform::from_translation(Vec3::new(-4.0, -4.6, 1.1))
                .with_scale(Vec3::new(1.7, 1.2, 6.0)),
            Name::new("Left Sofa"),
        ))
        .with_children(|commands| {
            commands.spawn((
                cube.clone(),
                MeshMaterial3d(materials.add(Color::from(css::DARK_GREEN))),
                Transform::from_translation(Vec3::new(-0.4, 1.0, 0.0))
                    .with_scale(Vec3::new(0.2, 1.0, 1.0)),
                Name::new("Sofa Back"),
            ));
        });

    // Coffee table.
    commands.spawn((
        cube.clone(),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::new(-1.3, -4.5, 1.1))
            .with_scale(Vec3::new(1.5, 0.8, 5.0)),
        Name::new("Coffee table"),
    ));

    // TV.
    commands.spawn((
        cube.clone(),
        MeshMaterial3d(materials.add(Color::BLACK)),
        Transform::from_translation(Vec3::new(4.8, -1.5, 1.1)).with_scale(Vec3::new(0.2, 3.0, 5.0)),
        Name::new("TV"),
    ));

    // wall Left
    let mut mesh = Mesh::from(Cuboid::new(WALL_THICKNESS, ROOM_SIZE.y, ROOM_SIZE.z));
    mesh.generate_tangents().unwrap();
    let mesh = Mesh3d(meshes.add(mesh));
    let mut wall = commands.spawn((
        mesh.clone(),
        Transform::from_translation(WALL_LEFT),
        MeshMaterial3d(materials.add(Color::from(css::RED))),
    ));
    wall.insert(Name::new("Wall left"));

    // wall Right
    let mut wall = commands.spawn((
        mesh,
        Transform::from_translation(WALL_RIGHT),
        MeshMaterial3d(materials.add(Color::from(css::RED))),
    ));
    wall.insert(Name::new("Wall right"));

    // wall Floor
    let mut mesh = Mesh::from(Cuboid::new(ROOM_SIZE.x, WALL_THICKNESS, ROOM_SIZE.z));
    mesh.generate_tangents().unwrap();
    let mesh = Mesh3d(meshes.add(mesh));
    let mut wall = commands.spawn((
        mesh.clone(),
        Transform::from_translation(WALL_FLOOR),
        MeshMaterial3d(materials.add(Color::from(css::GREEN))),
    ));
    wall.insert(Name::new("Wall floor"));

    // wall Ceiling
    let mut wall = commands.spawn((
        mesh,
        Transform::from_translation(WALL_CEILING),
        MeshMaterial3d(materials.add(Color::from(css::GREEN))),
    ));
    wall.insert(Name::new("Wall ceiling"));

    // light
    commands.spawn((
        PointLight {
            intensity: 1500000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, (ROOM_SIZE.y / 2.0) - 2.0, 0.0),
    ));
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
        Transform::from_xyz(0., 0., 9.0),
        NotShadowCaster,
    ));
}
