use crate::mesh::MeshType;

pub struct DrawCommand {
    pub mesh_type: MeshType,
    pub material: crate::renderer::material::MaterialId,
    pub position: glam::Vec3,
    pub scale: glam::Vec3,
    pub rotation: glam::Quat,
    pub tint: glam::Vec4,
}

impl DrawCommand {
    pub fn new(mesh_type: MeshType) -> Self {
        Self {
            mesh_type,
            material: crate::renderer::material::MaterialId(0),
            position: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
            rotation: glam::Quat::IDENTITY,
            tint: glam::Vec4::ONE,
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
        self.tint = glam::vec4(r, g, b, a);
        self
    }
}
