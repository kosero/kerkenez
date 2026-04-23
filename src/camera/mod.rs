pub mod orthographic;
pub mod perspective;

use glam::{Mat4, Quat, Vec3};
use orthographic::OrthographicProjection;
use perspective::PerspectiveProjection;

pub enum CameraProjection {
    Orthographic(OrthographicProjection),
    Perspective(PerspectiveProjection),
}

pub struct Camera {
    projection: CameraProjection,
    position: Vec3,
    rotation: Quat,

    view_matrix: Mat4,
    projection_matrix: Mat4,
    view_projection_matrix: Mat4,
}

impl Camera {
    pub fn new(projection: CameraProjection) -> Self {
        let projection_matrix = match &projection {
            CameraProjection::Orthographic(ortho) => ortho.projection_matrix(),
            CameraProjection::Perspective(persp) => persp.projection_matrix(),
        };

        let mut cam = Self {
            projection,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            view_matrix: Mat4::IDENTITY,
            projection_matrix,
            view_projection_matrix: projection_matrix,
        };

        cam.recalculate_matrices();
        cam
    }

    pub fn new_perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self::new(CameraProjection::Perspective(PerspectiveProjection::new(
            fov,
            aspect_ratio,
            near,
            far,
        )))
    }

    pub fn new_orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self::new(CameraProjection::Orthographic(OrthographicProjection::new(
            left, right, bottom, top, near, far,
        )))
    }

    pub fn projection(&self) -> &CameraProjection {
        &self.projection
    }

    pub fn set_projection(&mut self, projection: CameraProjection) {
        self.projection_matrix = match &projection {
            CameraProjection::Orthographic(ortho) => ortho.projection_matrix(),
            CameraProjection::Perspective(persp) => persp.projection_matrix(),
        };
        self.projection = projection;
        self.recalculate_matrices();
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.recalculate_matrices();
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.recalculate_matrices();
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.view_projection_matrix
    }

    fn recalculate_matrices(&mut self) {
        let transform = Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation);

        self.view_matrix = transform.inverse();
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
    }
}
