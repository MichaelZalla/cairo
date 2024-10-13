use crate::{
    color::Color,
    entity::Entity,
    matrix::Mat4,
    mesh::Mesh,
    render::{
        culling::{FaceCullingReject, FaceCullingStrategy},
        options::{rasterizer::RasterizerOptions, RenderOptions},
    },
    resource::arena::Arena,
    software_renderer::SoftwareRenderer,
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn _render_entity_aabb(
        &mut self,
        entity: &Entity,
        world_transform: &Mat4,
        mesh_arena: &Arena<Mesh>,
        color: &Color,
    ) {
        match mesh_arena.get(&entity.mesh) {
            Ok(entry) => {
                let mesh = &entry.item;

                let original_options = self.options;

                self.options = RenderOptions {
                    wireframe_color: *color,
                    draw_wireframe: true,
                    do_rasterization: false,
                    do_lighting: false,
                    do_deferred_lighting: false,
                    rasterizer_options: RasterizerOptions {
                        face_culling_strategy: FaceCullingStrategy {
                            reject: FaceCullingReject::None,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                };

                self.render_entity_mesh(mesh, world_transform);

                self.options = original_options;
            }
            Err(err) => panic!(
                "Failed to get Mesh from Arena with Handle {:?}: {}",
                entity.mesh, err
            ),
        }
    }
}
