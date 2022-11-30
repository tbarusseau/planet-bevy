#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var N = normalize(in.world_normal);
    var L = normalize(view.world_position.xyz - in.world_position.xyz);

    var NdotL = max(dot(N, L), 0.0001);

    return vec4(vec3(NdotL), 1.0);
}
