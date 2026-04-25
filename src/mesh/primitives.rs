use crate::mesh::{Vertex, aabb::AABB};

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
                position: [0.5, 0.5, 0.0, 0.0],
                tex_coords: [1.0, 1.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            }, // Top Right
            Vertex {
                position: [0.5, -0.5, 0.0, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            }, // Bottom Right
            Vertex {
                position: [-0.5, -0.5, 0.0, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            }, // Bottom Left
            Vertex {
                position: [-0.5, 0.5, 0.0, 0.0],
                tex_coords: [0.0, 1.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            }, // Top Left
        ];
        let indices = vec![0, 1, 3, 1, 2, 3];
        Self::new(vertices, indices)
    }

    pub fn triangle() -> Self {
        let vertices = vec![
            Vertex {
                position: [0.0, 0.5, 0.0, 0.0],
                tex_coords: [0.5, 1.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
        ];
        let indices = vec![0, 1, 2];
        Self::new(vertices, indices)
    }

    pub fn cube() -> Self {
        let vertices = vec![
            // Front face (normal: +Z)
            Vertex {
                position: [-0.5, -0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5, 0.0],
                tex_coords: [1.0, 1.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 1.0, 0.0],
            },
            // Back face (normal: -Z)
            Vertex {
                position: [-0.5, -0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0, 0.0, 0.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0, 0.0, 0.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [0.0, 0.0, -1.0, 0.0],
            },
            // Top face (normal: +Y)
            Vertex {
                position: [-0.5, 0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5, 0.0],
                tex_coords: [1.0, 1.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0, 0.0],
            },
            // Bottom face (normal: -Y)
            Vertex {
                position: [-0.5, -0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0, 0.0, 0.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0, 0.0, 0.0],
                normal: [0.0, -1.0, 0.0, 0.0],
            },
            // Right face (normal: +X)
            Vertex {
                position: [0.5, -0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0, 0.0, 0.0],
                normal: [1.0, 0.0, 0.0, 0.0],
            },
            // Left face (normal: -X)
            Vertex {
                position: [-0.5, -0.5, 0.5, 0.0],
                tex_coords: [1.0, 1.0, 0.0, 0.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0, 0.0, 0.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0, 0.0, 0.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0, 0.0, 0.0],
                normal: [-1.0, 0.0, 0.0, 0.0],
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

    pub fn sphere(radius: f32, sectors: u32, stacks: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=stacks {
            let stack_angle =
                std::f32::consts::PI / 2.0 - i as f32 * (std::f32::consts::PI / stacks as f32);
            let xy = radius * stack_angle.cos();
            let z = radius * stack_angle.sin();

            for j in 0..=sectors {
                let sector_angle = j as f32 * (2.0 * std::f32::consts::PI / sectors as f32);

                let x = xy * sector_angle.cos();
                let y = xy * sector_angle.sin();

                vertices.push(Vertex {
                    position: [x, y, z, 1.0], // W değerini 1.0 yapalım
                    tex_coords: [
                        j as f32 / sectors as f32,
                        i as f32 / stacks as f32,
                        0.0,
                        0.0,
                    ],
                    normal: [x / radius, y / radius, z / radius, 0.0],
                });
            }
        }

        for i in 0..stacks {
            for j in 0..sectors {
                let first = i * (sectors + 1) + j;
                let second = first + sectors + 1;

                // İki üçgen (quad)
                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        Self::new(vertices, indices)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshType {
    Square,
    Triangle,
    Cube,
    Sphere { sectors: u32, stacks: u32 },
}

impl MeshType {
    pub const DEFAULT_SPHERE_SECTORS: u32 = 12;
    pub const DEFAULT_SPHERE_STACKS: u32 = 6;

    pub fn default_sphere() -> Self {
        Self::Sphere {
            sectors: Self::DEFAULT_SPHERE_SECTORS,
            stacks: Self::DEFAULT_SPHERE_STACKS,
        }
    }
}
