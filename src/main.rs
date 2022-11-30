use std::f32::consts::FRAC_PI_4;

use bevy::{
    prelude::*,
    render::settings::{WgpuFeatures, WgpuSettings},
};
use camera::pan_orbit_camera::{sys_pan_orbit_camera, sys_spawn_camera};

#[allow(unused)]
use bevy_editor_pls::prelude::*;
use rendering::{
    planet_material::PlanetMaterial,
    sphere_mesh::{SphereMeshComponent, SphereMeshPlugin},
};

mod camera;
mod editor;
mod rendering;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes: true,
        ..Default::default()
    }))
    .insert_resource(WgpuSettings {
        features: WgpuFeatures::POLYGON_MODE_LINE,
        ..Default::default()
    })
    .add_plugin(EditorPlugin)
    .insert_resource(ClearColor { 0: Color::GRAY })
    .add_plugin(MaterialPlugin::<PlanetMaterial>::default())
    .add_startup_system(sys_spawn_camera)
    .add_system(sys_pan_orbit_camera)
    .add_startup_system(sys_spawn_light)
    .add_plugin(SphereMeshPlugin);

    app.register_type::<SphereMeshComponent>();

    app.run();
}

fn sys_spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::YELLOW,
            ..Default::default()
        },
        transform: Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -FRAC_PI_4, FRAC_PI_4, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.2,
    });
}
