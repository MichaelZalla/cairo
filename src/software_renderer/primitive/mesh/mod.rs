use std::rc::Rc;

use crate::{
    geometry::{accelerator::static_triangle_bvh::StaticTriangleBVH, intersect::test_aabb_aabb},
    matrix::Mat4,
    mesh::{face::Face, mesh_geometry::MeshGeometry, Mesh},
    software_renderer::SoftwareRenderer,
    vec::vec3::Vec3,
    vertex::{default_vertex_in::DefaultVertexIn, default_vertex_out::DefaultVertexOut},
};

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn render_entity_mesh(
        &mut self,
        mesh: &Mesh,
        world_transform: &Mat4,
    ) {
        // Otherwise, cull individual triangles.

        let original_world_transform: Mat4;

        {
            let mut context = self.shader_context.borrow_mut();

            original_world_transform = context.get_world_transform();

            context.set_world_transform(*world_transform);
        }

        match (&mesh.collider, &mesh.faces) {
            (Some(bvh), faces) => {
                self.render_mesh_geometry_bvh(&mesh.geometry, faces, bvh);
            }
            (None, faces) => {
                self.render_mesh_geometry(&mesh.geometry, faces);
            }
        }

        // Reset the shader context's original world transform.
        {
            let mut context = self.shader_context.borrow_mut();

            context.set_world_transform(original_world_transform);
        }
    }

    fn render_mesh_geometry(&mut self, geometry: &MeshGeometry, faces: &[Face]) {
        self.process_object_space_vertices(geometry, faces);
    }

    fn render_mesh_geometry_bvh(
        &mut self,
        geometry: &MeshGeometry,
        faces: &[Face],
        bvh: &Rc<StaticTriangleBVH>,
    ) {
        self.render_mesh_geometry_bvh_node(geometry, faces, bvh, 0);
    }

    fn render_mesh_geometry_bvh_node(
        &mut self,
        geometry: &MeshGeometry,
        faces: &[Face],
        bvh: &Rc<StaticTriangleBVH>,
        node_index: usize,
    ) {
        let node = &bvh.nodes[node_index];

        if !test_aabb_aabb(self.clipping_frustum.get_aabb(), &node.aabb) {
            return;
        }

        if node.is_leaf() {
            for i in node.primitives_start_index as usize
                ..(node.primitives_start_index + node.primitives_count) as usize
            {
                let face_index = bvh.tri_indices[i];

                let faces_slice = &faces[face_index..face_index + 1];

                self.process_object_space_vertices(geometry, faces_slice);
            }
        } else {
            let left = node.left_child_index as usize;
            self.render_mesh_geometry_bvh_node(geometry, faces, bvh, left);

            let right = left + 1;
            self.render_mesh_geometry_bvh_node(geometry, faces, bvh, right);
        }
    }

    fn process_object_space_vertices(&mut self, geometry: &MeshGeometry, faces: &[Face]) {
        // Map each face to a set of 3 unique instances of DefaultVertexIn.

        let mut vertices_in: Vec<DefaultVertexIn> = Vec::with_capacity(faces.len() * 3);

        for face in faces {
            let [v0_in, v1_in, v2_in] = get_vertices_in(geometry, face);

            vertices_in.push(v0_in);
            vertices_in.push(v1_in);
            vertices_in.push(v2_in);
        }

        // Process mesh vertices from object-space to world-space.
        let projection_space_vertices: Vec<DefaultVertexOut>;

        {
            let shader_context = self.shader_context.borrow();

            projection_space_vertices = vertices_in
                .into_iter()
                .map(|v_in| (self.vertex_shader)(&shader_context, &v_in))
                .collect();
        }

        self.process_triangles(faces, projection_space_vertices.as_slice());
    }
}

fn get_vertices_in(geometry: &MeshGeometry, face: &Face) -> [DefaultVertexIn; 3] {
    let (v0, v1, v2) = (
        geometry.vertices[face.vertices[0]],
        geometry.vertices[face.vertices[1]],
        geometry.vertices[face.vertices[2]],
    );

    let (normal0, normal1, normal2) = (
        geometry.normals[face.normals[0]],
        geometry.normals[face.normals[1]],
        geometry.normals[face.normals[2]],
    );

    let (uv0, uv1, uv2) = (
        geometry.uvs[face.uvs[0]],
        geometry.uvs[face.uvs[1]],
        geometry.uvs[face.uvs[2]],
    );

    let (tangent0, tangent1, tangent2) = (face.tangents[0], face.tangents[1], face.tangents[2]);

    let (bitangent0, bitangent1, bitangent2) =
        (face.bitangents[0], face.bitangents[1], face.bitangents[2]);

    static WHITE: Vec3 = Vec3::ones();

    let v0_in = DefaultVertexIn {
        position: v0,
        normal: normal0,
        uv: uv0,
        tangent: tangent0,
        bitangent: bitangent0,
        color: WHITE,
    };

    let v1_in = DefaultVertexIn {
        position: v1,
        normal: normal1,
        uv: uv1,
        tangent: tangent1,
        bitangent: bitangent1,
        color: WHITE,
    };

    let v2_in = DefaultVertexIn {
        position: v2,
        normal: normal2,
        uv: uv2,
        tangent: tangent2,
        bitangent: bitangent2,
        color: WHITE,
    };

    [v0_in, v1_in, v2_in]
}
