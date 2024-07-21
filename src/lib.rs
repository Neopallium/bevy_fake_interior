//! A shader and a material that uses it.

use bevy::{
  prelude::*,
  reflect::Reflect,
  render::{render_asset::*, render_resource::*, texture::GpuImage},
  pbr::{ExtendedMaterial, MaterialExtension},
};

pub type StandardFakeInteriorMaterial = ExtendedMaterial<StandardMaterial, FakeInteriorMaterial>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[uniform(100, FakeInteriorMaterialUniform)]
#[reflect(Default, Debug)]
pub struct FakeInteriorMaterial {
  pub atlas_rooms: Vec2,
  pub rooms: Vec2,
  pub depth: f32,
  pub room_seed: f32,
  pub emission_seed: f32,
  pub emission_threshold: f32,
}

impl Default for FakeInteriorMaterial {
  fn default() -> Self {
    Self {
      atlas_rooms: Vec2::new(1.0, 1.0),
      rooms: Vec2::new(1.0, 1.0),
      depth: 0.5,
      room_seed: 1.0,
      emission_seed: 1.0,
      emission_threshold: 0.5,
    }
  }
}

#[derive(Clone, Default, ShaderType)]
pub struct FakeInteriorMaterialUniform {
  pub atlas_rooms: Vec2,
  pub rooms: Vec2,
  pub depth: f32,
  pub room_seed: f32,
  pub emission_seed: f32,
  pub emission_threshold: f32,
}

impl AsBindGroupShaderType<FakeInteriorMaterialUniform> for FakeInteriorMaterial {
  fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> FakeInteriorMaterialUniform {
    FakeInteriorMaterialUniform {
      atlas_rooms: self.atlas_rooms,
      rooms: self.rooms,
      depth: self.depth,
      room_seed: self.room_seed,
      emission_seed: self.emission_seed,
      emission_threshold: self.emission_threshold,
    }
  }
}

impl MaterialExtension for FakeInteriorMaterial {
  fn fragment_shader() -> ShaderRef {
    "shaders/fake_interior.wgsl".into()
  }
}

#[derive(Default, Clone, Debug)]
pub struct FakeInteriorMaterialPlugin;

impl Plugin for FakeInteriorMaterialPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(MaterialPlugin::<StandardFakeInteriorMaterial>::default())
      .register_asset_reflect::<StandardFakeInteriorMaterial>()
      .register_asset_reflect::<FakeInteriorMaterial>();
  }
}
