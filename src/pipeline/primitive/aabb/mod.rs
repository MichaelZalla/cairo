use crate::{
    color::Color,
    entity::Entity,
    pipeline::{options::PipelineOptions, Pipeline},
    shader::{
        alpha::AlphaShader, fragment::FragmentShader, geometry::GeometryShader,
        vertex::VertexShader,
    },
};

impl<'a, F, V, A, G> Pipeline<'a, F, V, A, G>
where
    F: FragmentShader<'a>,
    V: VertexShader<'a>,
    A: AlphaShader<'a>,
    G: GeometryShader<'a>,
{
    pub fn render_entity_aabb(&mut self, entity: &Entity, color: Color) {
        let original_options = self.options.clone();

        self.options = PipelineOptions {
            wireframe_color: color,
            show_wireframe: true,
            show_rasterized_geometry: false,
            show_lighting: false,
            show_normals: false,
            cull_backfaces: false,
        };

        self.render_entity_mesh(entity, &entity.bounds_mesh, None);

        self.options = original_options;
    }
}