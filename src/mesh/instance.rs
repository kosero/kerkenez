#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model_matrix: glam::Mat4,
    pub tint: glam::Vec4,
}
