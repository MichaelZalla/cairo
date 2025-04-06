use crate::{
    matrix::Mat4, render::Renderer, scene::empty::EmptyDisplayKind,
    software_renderer::SoftwareRenderer, vec::vec4::Vec4,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_empty(
        &mut self,
        transform: &Mat4,
        _display_kind: EmptyDisplayKind,
    ) {
        let world_position = (Vec4::new(Default::default(), 1.0) * *transform).to_vec3();

        self.render_axes(Some(world_position), None);
    }
}
