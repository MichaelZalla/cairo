use crate::{
    color::Color,
    entity::Entity,
    matrix::Mat4,
    mesh::Mesh,
    pipeline::{
        options::{PipelineFaceCullingReject, PipelineFaceCullingStrategy, PipelineOptions},
        Pipeline,
    },
    resource::arena::Arena,
};

impl Pipeline {
    pub(in crate::pipeline) fn _render_entity_aabb(
        &mut self,
        entity: &Entity,
        world_transform: &Mat4,
        mesh_arena: &Arena<Mesh>,
        color: Color,
    ) {
        match mesh_arena.get(&entity.mesh) {
            Ok(entry) => {
                let mesh = &entry.item;

                let original_options = self.options;

                self.options = PipelineOptions {
                    wireframe_color: color,
                    do_wireframe: true,
                    do_rasterized_geometry: false,
                    do_lighting: false,
                    do_deferred_lighting: false,
                    face_culling_strategy: PipelineFaceCullingStrategy {
                        reject: PipelineFaceCullingReject::None,
                        ..Default::default()
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
