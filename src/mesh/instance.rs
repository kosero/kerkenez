use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Instance {
    pub model_matrix: Mat4,
    pub color: Vec3,
}
