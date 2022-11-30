use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use super::planet_material::PlanetMaterial;

pub struct SphereMeshPlugin;

impl Plugin for SphereMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(SphereMesh::sys_generate_meshes)
            // Generate an initial sphere mesh component
            .add_startup_system(|mut commands: Commands| {
                commands.spawn(SphereMeshComponent::default());
            });
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SphereMeshComponent {
    pub resolution: usize,
}

impl Default for SphereMeshComponent {
    fn default() -> Self {
        Self { resolution: 4 }
    }
}

#[derive(Default)]
struct SphereMesh {
    positions: Vec<Vec3>,
    indices: Vec<usize>,
    edges: Vec<Edge>,
    normals: Vec<Vec3>,

    num_verts_per_face: usize,
    num_divisions: usize,
}

const VERTEX_PAIRS: &[usize] = &[
    0, 1, 0, 2, 0, 3, 0, 4, 1, 2, 2, 3, 3, 4, 4, 1, 5, 1, 5, 2, 5, 3, 5, 4,
];
const EDGE_TRIPLETS: &[usize] = &[
    0, 1, 4, 1, 2, 5, 2, 3, 6, 3, 0, 7, 8, 9, 4, 9, 10, 5, 10, 11, 6, 11, 8, 7,
];
const BASE_VERTICES: &[Vec3] = &[
    Vec3::Y,
    Vec3::NEG_X,
    Vec3::NEG_Z,
    Vec3::X,
    Vec3::Z,
    Vec3::NEG_Y,
];

impl SphereMesh {
    pub fn sys_generate_meshes(
        mut commands: Commands,
        q: Query<(Entity, &SphereMeshComponent), Changed<SphereMeshComponent>>,
        q_previous_data: Query<&Transform>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<PlanetMaterial>>,
    ) {
        for (entity, sphere_mesh_comp) in q.iter() {
            let previous_data = q_previous_data.get(entity);

            // Clean-up previous PbrBundle
            commands.entity(entity).remove::<PbrBundle>();

            // Generate the new mesh
            let mut sphere_mesh = SphereMesh::default();
            sphere_mesh.generate_mesh(sphere_mesh_comp.resolution);
            let mesh = sphere_mesh.build_mesh();

            // Insert a new PbrBundle
            let material = materials.add(PlanetMaterial {});
            let mut pbr_bundle = MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: material.clone(),
                ..Default::default()
            };

            if let Ok(transform) = previous_data {
                pbr_bundle.transform = *transform;
            }

            commands.entity(entity).insert(pbr_bundle);
        }
    }

    pub fn generate_mesh(&mut self, resolution: usize) {
        self.num_divisions = usize::max(resolution, 0);
        self.num_verts_per_face =
            ((self.num_divisions + 3) * (self.num_divisions + 3) - (self.num_divisions + 3)) / 2;
        let num_verts = self.num_verts_per_face * 8 - (self.num_divisions + 2) * 12 + 6;
        let num_tris_per_face = (self.num_divisions + 1) * (self.num_divisions + 1);

        self.positions = Vec::with_capacity(num_verts);
        self.indices = Vec::with_capacity(num_tris_per_face * 8 * 3);
        self.normals = Vec::with_capacity(num_verts);

        self.positions.extend_from_slice(BASE_VERTICES);
        self.normals.extend_from_slice(BASE_VERTICES);

        // Create 12 edges, with n vertices added long them (n = num_divisions)
        self.edges = vec![Edge::default(); 12];
        for i in (0..VERTEX_PAIRS.len()).step_by(2) {
            let start_vertex = self.positions[VERTEX_PAIRS[i]];
            let end_vertex = self.positions[VERTEX_PAIRS[i + 1]];

            let mut edge_vertex_indices: Vec<usize> = vec![0; self.num_divisions + 2];
            edge_vertex_indices[0] = VERTEX_PAIRS[i];

            // Add vertices along edge
            for division_index in 0..self.num_divisions {
                let t = (division_index as f32 + 1.0) / (self.num_divisions as f32 + 1.0);
                edge_vertex_indices[division_index + 1] = self.positions.len();

                let position = slerp(&start_vertex, &end_vertex, t);
                self.positions.push(position);
                self.normals.push(position);
            }

            edge_vertex_indices[self.num_divisions + 1] = VERTEX_PAIRS[i + 1];
            let edge_index = i / 2;
            self.edges[edge_index] = Edge::new(edge_vertex_indices.clone());
        }

        // Create faces
        for i in (0..EDGE_TRIPLETS.len()).step_by(3) {
            let face_index = i / 3;
            let reverse = face_index >= 4;
            self.create_face(i, reverse);
        }
    }

    fn create_face(&mut self, idx_start: usize, reverse: bool) {
        let side_a = self.edges.get(EDGE_TRIPLETS[idx_start]).unwrap();
        let side_b = self.edges.get(EDGE_TRIPLETS[idx_start + 1]).unwrap();
        let bottom = self.edges.get(EDGE_TRIPLETS[idx_start + 2]).unwrap();

        let num_points_in_edge = side_a.vertex_indices.len();
        let mut vertex_map: Vec<usize> = Vec::with_capacity(self.num_verts_per_face);
        vertex_map.push(side_a.vertex_indices[0]);

        for i in 1..(num_points_in_edge - 1) {
            vertex_map.push(side_a.vertex_indices[i]);

            let side_a_vertex = self.positions[side_a.vertex_indices[i]];
            let side_b_vertex = self.positions[side_b.vertex_indices[i]];
            let num_inner_points = i - 1;

            for j in 0..num_inner_points {
                let t = (j as f32 + 1.0) / (num_inner_points as f32 + 1.0);
                vertex_map.push(self.positions.len());

                let position = slerp(&side_a_vertex, &side_b_vertex, t);
                self.positions.push(position);
                self.normals.push(position);
            }

            vertex_map.push(side_b.vertex_indices[i]);
        }

        for i in 0..num_points_in_edge {
            vertex_map.push(bottom.vertex_indices[i]);
        }

        // Triangulate
        let num_rows = self.num_divisions + 1;
        for row in 0..num_rows {
            let mut top_vertex = ((row + 1) * (row + 1) - row - 1) / 2;
            let mut bottom_vertex = ((row + 2) * (row + 2) - row - 2) / 2;

            let num_triangles_in_row = 1 + 2 * row;

            for column in 0..num_triangles_in_row {
                let v0;
                let v1;
                let v2;

                if column % 2 == 0 {
                    v0 = top_vertex;
                    v1 = bottom_vertex + 1;
                    v2 = bottom_vertex;

                    top_vertex += 1;
                    bottom_vertex += 1;
                } else {
                    v0 = top_vertex;
                    v1 = bottom_vertex;
                    v2 = top_vertex - 1;
                }

                self.indices.push(vertex_map[v0]);
                self.indices.push(vertex_map[if reverse { v2 } else { v1 }]);
                self.indices.push(vertex_map[if reverse { v1 } else { v2 }]);
            }
        }
    }

    pub fn build_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(
            self.indices.iter().to_owned().map(|v| *v as u32).collect(),
        )));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        // TODO: UVs
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }

    // pub fn spawn_sphere_mesh(
    //     self,
    //     commands: &mut Commands,
    //     meshes: &mut Assets<Mesh>,
    //     materials: &mut Assets<StandardMaterial>,
    // ) {
    //     let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    //     mesh.set_indices(Some(Indices::U32(
    //         self.indices.iter().to_owned().map(|v| *v as u32).collect(),
    //     )));
    //     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
    //     mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
    //     // TODO: UVs
    //     // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    //     // Spawn the associated PbrBundle
    //     commands.spawn(PbrBundle {
    //         mesh: meshes.add(mesh),
    //         material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //         ..Default::default()
    //     });
    // }
}

#[derive(Clone)]
pub struct Edge {
    pub vertex_indices: Vec<usize>,
}

impl Edge {
    pub fn new(indices: Vec<usize>) -> Edge {
        Edge {
            vertex_indices: indices,
        }
    }
}

impl Default for Edge {
    fn default() -> Self {
        Self {
            vertex_indices: Vec::new(),
        }
    }
}

fn slerp(start: &Vec3, end: &Vec3, t: f32) -> Vec3 {
    let dot = start.dot(*end);

    let dot = dot.min(1.0).max(-1.0);

    let theta_0 = dot.acos();
    let theta = theta_0 * t;

    let v2 = (*end - (*start * dot)).normalize();
    let (s, c) = theta.sin_cos();

    *start * c + v2 * s
}
