use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::rendering::sphere_mesh::SphereMeshComponent;

pub struct SphereMeshEditorPlugin;

impl Plugin for SphereMeshEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(sys_ui)
            .insert_resource(SphereMeshEditorState::default());
    }
}

#[derive(Default, Resource)]
pub struct SphereMeshEditorState {
    opened: bool,
}

fn sys_ui(
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<SphereMeshEditorState>,
    mut q: Query<(Entity, &mut SphereMeshComponent)>,
) {
    egui::TopBottomPanel::top("top-menu").show(egui_context.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            if ui
                .selectable_label(state.opened, "Sphere mesh editor")
                .clicked()
            {
                state.opened = !state.opened;
            }
        });
    });

    if !state.opened {
        return;
    }

    egui::Window::new("Sphere mesh editor").show(egui_context.ctx_mut(), |ui| {
        for (entity, mut sphere_mesh_component) in q.iter_mut() {
            ui.heading(format!("{:?}", entity));
            ui.horizontal(|ui| {
                ui.label("Resolution");
                let mut res = sphere_mesh_component.resolution.to_string();
                if ui.text_edit_singleline(&mut res).changed() {
                    if let Ok(v) = usize::from_str_radix(&res, 10) {
                        sphere_mesh_component.resolution = v;
                    }
                }
            });
            ui.separator();
        }
    });
}
