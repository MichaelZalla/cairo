use crate::{
    color::Color,
    entity::Entity,
    matrix::Mat4,
    pipeline::{
        options::{PipelineFaceCullingReject, PipelineFaceCullingStrategy, PipelineOptions},
        Pipeline,
    },
};

impl<'a> Pipeline<'a> {
    pub fn render_entity_aabb(&mut self, entity: &Entity, world_transform: &Mat4, color: Color) {
        let original_options = self.options.clone();

        self.options = PipelineOptions {
            wireframe_color: color,
            do_wireframe: true,
            do_rasterized_geometry: false,
            do_lighting: false,
            do_deferred_lighting: false,
            do_bloom: false,
            do_visualize_normals: false,
            face_culling_strategy: PipelineFaceCullingStrategy {
                reject: PipelineFaceCullingReject::None,
                ..Default::default()
            },
        };

        self.render_entity_mesh(entity, world_transform, &entity.bounds_mesh, None);

        self.options = original_options;
    }
}
