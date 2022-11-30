use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "249f2570-f69e-466c-b85d-d3e6a7f35ac7"]
pub struct PlanetMaterial {}

impl Material for PlanetMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/planet.wgsl".into()
    }
}
