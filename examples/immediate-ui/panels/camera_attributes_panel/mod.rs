use std::fmt::{Debug, Display};

use cairo::{
    mem::linked_list::LinkedList,
    resource::handle::Handle,
    scene::camera::{Camera, CameraProjectionKind},
    serde::PostDeserialize,
    ui::{
        fastpath::{
            radio::{radio_group, RadioOption},
            slider::{slider, SliderOptions},
            spacer::spacer,
            tab_selector::tab_selector,
            text::text,
        },
        ui_box::tree::UIBoxTree,
    },
};

use crate::{command::PendingCommand, COMMAND_BUFFER, SCENE_CONTEXT};

use super::PanelInstance;

#[derive(Clone)]
pub(crate) struct CameraAttributesPanel {
    id: String,
    camera_handle: Handle,
}

impl Debug for CameraAttributesPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CameraAttributesPanel")
            .field("id", &self.id)
            .field("camera_handle", &self.camera_handle)
            .finish()
    }
}

impl PostDeserialize for CameraAttributesPanel {
    fn post_deserialize(&mut self) {}
}

impl CameraAttributesPanel {
    pub fn new(id: &str, camera_handle: Handle) -> Self {
        Self {
            id: id.to_string(),
            camera_handle,
        }
    }
}

impl CameraAttributesPanel {
    pub fn make_command<T: Display>(&self, attribute: &str, args: &T) -> String {
        format!(
            "set camera {} {} {} {}",
            self.camera_handle.uuid, self.camera_handle.index, attribute, args,
        )
        .to_string()
    }

    pub fn render_for_camera(
        &self,
        camera: &Camera,
        tree: &mut UIBoxTree,
        pending_queue: &mut LinkedList<PendingCommand>,
    ) -> Result<(), String> {
        let tabs = vec!["Tab A", "Tab B", "Tab C"];

        match tab_selector(format!("{}.tab_test", self.id), tabs, tree)? {
            0 => {
                // Tab A
                tree.push(text(
                    format!("{}.tab_test.tab_a.label", self.id).to_string(),
                    "Tab A stuff.".to_string(),
                ))?;
            }
            1 => {
                // Tab B
                tree.push(text(
                    format!("{}.tab_test.tab_b.label", self.id).to_string(),
                    "Tab B stuff.".to_string(),
                ))?;
            }
            _ => {
                // Tab C
                tree.push(text(
                    format!("{}.tab_test.tab_c.label", self.id).to_string(),
                    "Tab C stuff.".to_string(),
                ))?;
            }
        }

        tree.push(spacer(18))?;

        // Projection kind

        tree.push(text(
            format!("{}.kind.label", self.id).to_string(),
            "Projection kind".to_string(),
        ))?;

        let current_projection_kind = camera.get_kind();

        let projection_kind_options: Vec<RadioOption> = [
            CameraProjectionKind::Orthographic.to_string(),
            CameraProjectionKind::Perspective.to_string(),
        ]
        .into_iter()
        .map(|label| RadioOption { label })
        .collect();

        let selected_projection_kind_index = [
            CameraProjectionKind::Orthographic,
            CameraProjectionKind::Perspective,
        ]
        .iter()
        .position(|i| *i == current_projection_kind)
        .unwrap();

        if let Some(new_selected_projection_kind_index) = radio_group(
            format!("{}.kind.radio_group", self.id).to_string(),
            &projection_kind_options,
            selected_projection_kind_index,
            tree,
        )? {
            let cmd_str = self.make_command("kind", &new_selected_projection_kind_index);

            pending_queue.push_back((cmd_str, false));
        }

        tree.push(spacer(18))?;

        match current_projection_kind {
            CameraProjectionKind::Perspective => {
                // Field of view

                tree.push(text(
                    format!(
                        "CameraAttributesPanel{}.perspective.field_of_view.label",
                        self.id
                    )
                    .to_string(),
                    "Field-of-view".to_string(),
                ))?;

                if let Some(new_fov) = slider(
                    format!("CameraAttributesPanel{}.perspective.field_of_view", self.id),
                    camera.get_field_of_view().unwrap(),
                    SliderOptions {
                        min: 22.5,
                        max: 160.0,
                        ..Default::default()
                    },
                    tree,
                )? {
                    let cmd_str = self.make_command("perspective.field_of_view", &new_fov);

                    pending_queue.push_back((cmd_str, false));
                }

                tree.push(spacer(18))?;
            }
            CameraProjectionKind::Orthographic => {
                // Extent
            }
        }

        // Projection Z-near

        tree.push(text(
            format!("CameraAttributesPanel{}.projection_z_near.label", self.id).to_string(),
            "Clip near".to_string(),
        ))?;

        if let Some(new_projection_z_near) = slider(
            format!("CameraAttributesPanel{}.projection_z_near", self.id),
            camera.get_projection_z_near(),
            SliderOptions {
                min: 0.1,
                max: 10.0,
                ..Default::default()
            },
            tree,
        )? {
            let cmd_str = self.make_command("projection_z_near", &new_projection_z_near);

            pending_queue.push_back((cmd_str, false));
        }

        tree.push(spacer(18))?;

        // Projection Z-far

        tree.push(text(
            format!("CameraAttributesPanel{}.projection_z_far.label", self.id).to_string(),
            "Clip far".to_string(),
        ))?;

        if let Some(new_projection_z_far) = slider(
            format!("CameraAttributesPanel{}.projection_z_far", self.id),
            camera.get_projection_z_far(),
            SliderOptions {
                min: 1.0,
                max: 100.0,
                ..Default::default()
            },
            tree,
        )? {
            let cmd_str = self.make_command("projection_z_far", &new_projection_z_far);

            pending_queue.push_back((cmd_str, false));
        }

        Ok(())
    }
}

impl PanelInstance for CameraAttributesPanel {
    fn render(&mut self, tree: &mut UIBoxTree) -> Result<(), String> {
        SCENE_CONTEXT.with(|ctx| -> Result<(), String> {
            let resources = ctx.resources.borrow();

            let camera_arena = resources.camera.borrow();

            if let Ok(entry) = camera_arena.get(&self.camera_handle) {
                let camera = &entry.item;

                COMMAND_BUFFER.with(|buffer| -> Result<(), String> {
                    let mut pending_queue = buffer.pending_commands.borrow_mut();

                    self.render_for_camera(camera, tree, &mut pending_queue)
                })?;
            } else {
                panic!(
                    "Invalid Camera handle {} assigned to CameraAttributesPanel {}!",
                    self.camera_handle.uuid, self.id
                );
            }

            Ok(())
        })
    }
}
