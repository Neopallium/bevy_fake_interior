#import bevy_pbr::{
	pbr_fragment::pbr_input_from_standard_material,
	pbr_functions::alpha_discard,
	pbr_bindings::{
		base_color_texture,
		base_color_sampler,
	},
	mesh_view_bindings::view,
	mesh_functions,
	skinning,
	view_transformations::position_world_to_clip,
}
#import bevy_render::instance_index::get_instance_index

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
	prepass_io::{Vertex, VertexOutput, FragmentOutput},
	pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
	forward_io::{Vertex, VertexOutput, FragmentOutput},
	pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
	pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
#endif

struct FakeInteriorMaterial {
  atlas_rooms: vec2<f32>,
  rooms: vec2<f32>,
	depth: f32,
	room_seed: f32,
	light_seed: f32,
	light_threshold: f32,
};

@group(2) @binding(100) var<uniform> material: FakeInteriorMaterial;

fn random2D(s: f32) -> vec2<f32> {
  return fract(sin(s * vec2<f32>(12.9898,78.233)) * 43758.5453);
}

fn random1D(s: f32) -> f32 {
  return fract(sin(s * 12.9898) * 43758.5453);
}

@fragment
fn fragment(
  v_in: VertexOutput,
  @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
	var in = v_in;
	let atlas_rooms = material.atlas_rooms;
	let UV = in.uv * material.rooms;
	// room uvs
	let room_uv = fract(UV);
	var room_index_uv = floor(UV);
	let room_index = (room_index_uv.x + room_index_uv.y * material.rooms.x);
	let room_seed = room_index * material.room_seed;

	// randomize the rooms
	let n = floor(random2D(room_seed) * atlas_rooms);
	room_index_uv += n;

	// get room depth from room atlas alpha else use the Depth paramater
	let depth_uv = fract((room_index_uv + 0.5) / atlas_rooms);
  var far_frac = textureSample(base_color_texture, base_color_sampler, depth_uv).a;
	if (far_frac >= 0.99) {
		far_frac = material.depth;
		if (far_frac >= 1.0 || far_frac < 0.0) {
			far_frac = 0.5;
		}
	}
	
	let depth_scale = 1.0 / (1.0 - far_frac) - 1.0;

	// raytrace material from view dir
	var pos = vec3<f32>(room_uv * 2.0 - 1.0, -1.0);
	let view_dir = in.world_position.xyz - view.world_position;
	let bitangent = normalize(cross(in.world_tangent.xyz, in.world_normal));
	// get tangent space camera vector
	let tangent_view_dir = vec3(
		dot(view_dir, in.world_tangent.xyz),
		dot(view_dir, bitangent),
		dot(view_dir, in.world_normal) * -depth_scale);
	let id = 1.0 / tangent_view_dir;
	let k = abs(id) - pos * id;
	let k_min = min(min(k.x, k.y), k.z);
	pos += k_min * tangent_view_dir;

	// 0.0 - 1.0 room depth
	var interp = pos.z * 0.5 + 0.5;

	// account for perspective in "room" textures
	// assumes camera with an fov of 53.13 degrees (atan(0.5))
	let real_z = clamp(interp, 0.0, 1.0) / depth_scale + 1.0;
	interp = 1.0 - (1.0 / real_z);
	interp *= depth_scale + 1.0;

	// iterpolate from wall back to near wall
	var interior_uv = pos.xy * mix(1.0, far_frac, interp);
	interior_uv = interior_uv * 0.5 + 0.5;

	// Update UV for PBR shader.
	in.uv = fract((room_index_uv + interior_uv) / atlas_rooms);
	
	// Randomly turn on room light.
	var has_light = random1D(room_index * material.light_seed);
	if has_light >= material.light_threshold {
		has_light = 0.0;
	} else {
		has_light = 1.0;
	}

	// get PbrInput from StandardMaterial bindings.
	var pbr_input = pbr_input_from_standard_material(in, is_front);

	pbr_input.material.emissive *= has_light;
	// alpha discard
  pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
	// No lighting in deferred mode.
	let out = deferred_output(in, pbr_input);
#else
	var out: FragmentOutput;
  if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
		out.color = apply_pbr_lighting(pbr_input);
	} else {
		out.color = pbr_input.material.base_color;
	}

	// Apply PBR post processing.
	out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

  return out;
}
