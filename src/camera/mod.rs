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

    dirty: bool,

    view_matrix: Mat4,
    projection_matrix: Mat4,
    view_projection_matrix: Mat4,
    inv_view_projection_matrix: Mat4,
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
            dirty: true,
            view_matrix: Mat4::IDENTITY,
            projection_matrix,
            view_projection_matrix: projection_matrix,
            inv_view_projection_matrix: projection_matrix.inverse(),
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

    pub fn resize(&mut self, width: f32, height: f32) {
        match &mut self.projection {
            CameraProjection::Perspective(persp) => {
                persp.aspect_ratio = width / height;
            }
            CameraProjection::Orthographic(ortho) => {
                let half_height = (ortho.top - ortho.bottom).abs() / 2.0;
                let aspect = width / height;
                ortho.left = -aspect * half_height;
                ortho.right = aspect * half_height;
            }
        }
        self.projection_matrix = match &self.projection {
            CameraProjection::Orthographic(ortho) => ortho.projection_matrix(),
            CameraProjection::Perspective(persp) => persp.projection_matrix(),
        };
        self.dirty = true;
    }

    pub fn set_projection(&mut self, projection: CameraProjection) {
        self.projection_matrix = match &projection {
            CameraProjection::Orthographic(ortho) => ortho.projection_matrix(),
            CameraProjection::Perspective(persp) => persp.projection_matrix(),
        };
        self.projection = projection;
        self.dirty = true;
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.dirty = true;
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.dirty = true;
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.view_projection_matrix
    }

    pub fn inv_view_projection_matrix(&self) -> Mat4 {
        self.inv_view_projection_matrix
    }

    pub fn update(&mut self) {
        if self.dirty {
            self.recalculate_matrices();
            self.dirty = false;
        }
    }

    fn recalculate_matrices(&mut self) {
        let transform = Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation);

        self.view_matrix = transform.inverse();
        self.view_projection_matrix = self.projection_matrix * self.view_matrix;
        self.inv_view_projection_matrix = self.view_projection_matrix.inverse();
    }
}
