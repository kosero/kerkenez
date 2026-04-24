use crate::mesh::Vertex;

pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl AABB {
    pub fn from_vertices(vertices: &[Vertex]) -> Self {
        let mut min = glam::Vec3::splat(f32::MAX);
        let mut max = glam::Vec3::splat(f32::MIN);

        for v in vertices {
            let pos = glam::Vec3::from_array(v.position);
            min = min.min(pos);
            max = max.max(pos);
        }

        Self { min, max }
    }

    pub fn transform(&self, matrix: &glam::Mat4) -> Self {
        let corners = [
            glam::vec3(self.min.x, self.min.y, self.min.z),
            glam::vec3(self.min.x, self.min.y, self.max.z),
            glam::vec3(self.min.x, self.max.y, self.min.z),
            glam::vec3(self.min.x, self.max.y, self.max.z),
            glam::vec3(self.max.x, self.min.y, self.min.z),
            glam::vec3(self.max.x, self.min.y, self.max.z),
            glam::vec3(self.max.x, self.max.y, self.min.z),
            glam::vec3(self.max.x, self.max.y, self.max.z),
        ];

        let mut min = glam::Vec3::splat(f32::MAX);
        let mut max = glam::Vec3::splat(f32::MIN);

        for &c in &corners {
            let transformed = matrix.transform_point3(c);
            min = min.min(transformed);
            max = max.max(transformed);
        }

        Self { min, max }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounding_box: AABB,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let bounding_box = AABB::from_vertices(&vertices);
        Self {
            vertices,
            indices,
            bounding_box,
        }
    }

    pub fn square() -> Self {
        let vertices = vec![
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            }, // Top Right
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            }, // Bottom Right
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            }, // Bottom Left
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            }, // Top Left
        ];
        let indices = vec![0, 1, 3, 1, 2, 3];
        Self::new(vertices, indices)
    }

    pub fn triangle() -> Self {
        let vertices = vec![
            Vertex {
                position: [0.0, 0.5, 0.0],
                tex_coords: [0.5, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2];
        Self::new(vertices, indices)
    }

    pub fn cube() -> Self {
        let vertices = vec![
            // Front face (normal: +Z)
            Vertex {
                position: [-0.5, -0.5, 0.5],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            // Back face (normal: -Z)
            Vertex {
                position: [-0.5, -0.5, -0.5],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, -1.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, -1.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, -1.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
            },
            // Top face (normal: +Y)
            Vertex {
                position: [-0.5, 0.5, 0.5],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            // Bottom face (normal: -Y)
            Vertex {
                position: [-0.5, -0.5, 0.5],
                tex_coords: [0.0, 0.0],
                normal: [0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                tex_coords: [1.0, 0.0],
                normal: [0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                tex_coords: [1.0, 1.0],
                normal: [0.0, -1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5],
                tex_coords: [0.0, 1.0],
                normal: [0.0, -1.0, 0.0],
            },
            // Right face (normal: +X)
            Vertex {
                position: [0.5, -0.5, 0.5],
                tex_coords: [0.0, 1.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                tex_coords: [0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                tex_coords: [1.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                tex_coords: [1.0, 1.0],
                normal: [1.0, 0.0, 0.0],
            },
            // Left face (normal: -X)
            Vertex {
                position: [-0.5, -0.5, 0.5],
                tex_coords: [1.0, 1.0],
                normal: [-1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                tex_coords: [1.0, 0.0],
                normal: [-1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                tex_coords: [0.0, 0.0],
                normal: [-1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5],
                tex_coords: [0.0, 1.0],
                normal: [-1.0, 0.0, 0.0],
            },
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // front
            4, 5, 6, 6, 7, 4, // back
            8, 9, 10, 10, 11, 8, // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];

        Self::new(vertices, indices)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshType {
    Square,
    Triangle,
    Cube,
}
