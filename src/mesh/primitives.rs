use crate::mesh::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn square() -> Self {
        let vertices = vec![
            Vertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 1.0],
            }, // Top Right
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0],
            }, // Bottom Right
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
            }, // Bottom Left
            Vertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 1.0],
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
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ];
        let indices = vec![0, 1, 2];
        Self::new(vertices, indices)
    }
}

pub struct Square;
impl Square {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RenderCommand {
        RenderCommand::new(MeshType::Square)
    }
}

pub struct Triangle;
impl Triangle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RenderCommand {
        RenderCommand::new(MeshType::Triangle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshType {
    Square,
    Triangle,
}

pub struct RenderCommand {
    pub mesh_type: MeshType,
    pub material: crate::renderer::material::MaterialId,
    pub position: glam::Vec3,
    pub scale: glam::Vec3,
    pub rotation: glam::Quat,
    pub color: glam::Vec3,
}

impl RenderCommand {
    pub fn new(mesh_type: MeshType) -> Self {
        Self {
            mesh_type,
            material: crate::renderer::material::MaterialId(0),
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
            color: glam::Vec3::ONE,
        }
    }

    pub fn material(mut self, id: crate::renderer::material::MaterialId) -> Self {
        self.material = id;
        self
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = glam::vec3(x, y, z);
        self
    }

    pub fn scale(mut self, s: f32) -> Self {
        self.scale = glam::Vec3::splat(s);
        self
    }

    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = glam::vec3(r, g, b);
        self
    }
}
