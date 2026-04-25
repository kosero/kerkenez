use glam::{Quat, Vec3};

use crate::{
    mesh::MeshType,
    renderer::{color::Color, material::MaterialId},
};

#[derive(Clone)]
pub struct DrawCommand {
    pub mesh_type: MeshType,
    pub material: MaterialId,
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
    pub tint: Color,
}

impl DrawCommand {
    pub fn new(mesh_type: MeshType) -> Self {
        Self {
            mesh_type,
            material: MaterialId::new(0),
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            tint: Color::WHITE,
        }
    }

    pub fn square() -> Self {
        Self::new(MeshType::Square)
    }

    pub fn triangle() -> Self {
        Self::new(MeshType::Triangle)
    }

    pub fn cube() -> Self {
        Self::new(MeshType::Cube)
    }

    pub fn material(mut self, id: crate::renderer::material::MaterialId) -> Self {
        self.material = id;
        self
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = glam::vec3(x, y, z);
        self
    }

    pub fn rotate(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
        self
    }

    pub fn scale(mut self, s: f32) -> Self {
        self.scale = glam::Vec3::splat(s);
        self
    }

    pub fn scale_xyz(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = glam::vec3(x, y, z);
        self
    }

    pub fn tint(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.tint = Color::rgba(r, g, b, a).to_linear();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.tint = color.to_linear();
        self
    }
}
