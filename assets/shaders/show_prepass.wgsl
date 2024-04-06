#import bevy_pbr::{
    mesh_view_bindings::globals,
    prepass_utils,
    forward_io::VertexOutput,
}

struct ShowPrepassSettings {
    show_depth: u32,
    show_normals: u32,
}
@group(2) @binding(0) var<uniform> settings: ShowPrepassSettings;

@fragment
fn fragment(
#ifdef MULTISAMPLED
    @builtin(sample_index) sample_index: u32,
#endif
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
#ifndef MULTISAMPLED
    let sample_index = 0u;
#endif
    if settings.show_depth == 1u {
        let depth = 1.0 - (bevy_pbr::prepass_utils::prepass_depth(mesh.position, sample_index) * 100.0);
        return vec4(depth, depth, depth, 1.0);
    } else if settings.show_normals == 1u {
        var normal = bevy_pbr::prepass_utils::prepass_normal(mesh.position, sample_index);
        normal = normalize(normal + vec3(0.5, 0.5, 0.5));
        return vec4(normal, 1.0);
    }

    return vec4(0.0);
}
