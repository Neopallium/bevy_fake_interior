#import bevy_pbr::{
	mesh_view_bindings::view,
	mesh_functions,
	view_transformations::position_world_to_clip,
}
#import bevy_render::instance_index::get_instance_index

#import bevy_pbr::forward_io::Vertex

struct CustomMaterial {
  atlas_rooms: vec2<f32>,
  rooms: vec2<f32>,
	depth: f32,
	room_seed: f32,
	emission_seed: f32,
	emission_threshold: f32,
};

@group(1) @binding(0) var<uniform> material: CustomMaterial;
@group(1) @binding(1) var room_texture: texture_2d<f32>;
@group(1) @binding(2) var room_sampler: sampler;

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_tangent: vec4<f32>,
    @location(4) tangent_view_dir: vec3<f32>,
}

fn room_random(s: f32) -> vec2<f32> {
  return fract(sin((s + material.room_seed) * vec2<f32>(12.9898,78.233)) * 43758.5453);
}

fn emit_random(s: vec2<f32>) -> f32 {
  return fract(sin(dot(s + vec2<f32>(material.emission_seed), vec2<f32>(12.9898,78.233))) * 43758.5453123);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
  var out: VertexOutput;

  var model = mesh_functions::get_model_matrix(vertex.instance_index);

  out.world_normal = mesh_functions::mesh_normal_local_to_world(
		vertex.normal,
		get_instance_index(vertex.instance_index)
	);

  out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));

  out.position = position_world_to_clip(out.world_position.xyz);

  out.uv = vertex.uv * material.rooms;

#ifdef VERTEX_TANGENTS
  out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
		model,
		vertex.tangent,
		get_instance_index(vertex.instance_index)
	);
#endif

	let view_dir = out.world_position.xyz - view.world_position;
	let bitangent = normalize(cross(out.world_tangent.xyz, out.world_normal));

	// get tangent space camera vector
	out.tangent_view_dir = vec3(
		dot(view_dir, out.world_tangent.xyz),
		dot(view_dir, bitangent),
		dot(view_dir, out.world_normal));

  return out;
}

@fragment
fn fragment(
  mesh: VertexOutput,
) -> @location(0) vec4<f32> {
	let atlas_rooms = material.atlas_rooms;
	let UV = mesh.uv;
	// room uvs
	let room_uv = fract(UV);
	var room_index_uv = floor(UV);

	// randomize the rooms
	let n = floor(room_random(room_index_uv.x + room_index_uv.y * (room_index_uv.x + 1.0)) * atlas_rooms);
	room_index_uv += n;

	// get room depth from room atlas alpha else use the Depth paramater
  var far_frac = textureSample(room_texture, room_sampler, (room_index_uv+0.5)/atlas_rooms).a;
	if (far_frac == 1.0) {
		far_frac = material.depth;
		if (far_frac >= 1.0 || far_frac < 0.0) {
			far_frac = 0.5;
		}
	}
	
	let depth_scale = 1.0 / (1.0 - far_frac) - 1.0;

	// raytrace box from view dir
	var pos = vec3<f32>(room_uv * 2.0 - 1.0, -1.0);
	var tangent_view_dir = mesh.tangent_view_dir;
	tangent_view_dir.z *= -depth_scale;
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

	// sample room atlas texture
	let uv = (room_index_uv + interior_uv) / atlas_rooms;
  let room = textureSample(room_texture, room_sampler, uv).rgb;
  var emit = textureSample(room_texture, room_sampler, uv).rgb * vec3<f32>(1.2);
	
	// use emission map based on threshold parameter
	var is_emit = emit_random(floor(UV));
	if is_emit >= material.emission_threshold {
		is_emit = 0.0;
	} else {
		is_emit = 1.0;
	}
	emit *= is_emit;

	// final result
	return vec4<f32>(room, 1.0);
}
