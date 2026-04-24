pub mod orthographic;
pub mod perspective;

use glam::{Mat4, Quat, Vec3};
use orthographic::OrthographicProjection;
use perspective::PerspectiveProjection;

pub struct Frustum {
    planes: [glam::Vec4; 6],
}

impl Frustum {
    pub fn from_matrix(m: &glam::Mat4) -> Self {
        let mut planes = [glam::Vec4::ZERO; 6];
        let m = m.to_cols_array_2d();

        // Left plane
        planes[0] = glam::vec4(
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        );
        // Right plane
        planes[1] = glam::vec4(
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        );
        // Bottom plane
        planes[2] = glam::vec4(
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        );
        // Top plane
        planes[3] = glam::vec4(
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        );
        // Near plane
        planes[4] = glam::vec4(
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        );
        // Far plane
        planes[5] = glam::vec4(
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        );

        for plane in &mut planes {
            let length = glam::Vec3::new(plane.x, plane.y, plane.z).length();
            *plane /= length;
        }

        Self { planes }
    }

    pub fn contains_aabb(&self, aabb: &crate::mesh::primitives::AABB) -> bool {
        for plane in &self.planes {
            let mut p = aabb.min;
            if plane.x >= 0.0 {
                p.x = aabb.max.x;
            }
            if plane.y >= 0.0 {
                p.y = aabb.max.y;
            }
            if plane.z >= 0.0 {
                p.z = aabb.max.z;
            }

            if plane.x * p.x + plane.y * p.y + plane.z * p.z + plane.w < 0.0 {
                return false;
            }
        }
        true
    }
}

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
    frustum: Frustum,
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
            frustum: Frustum::from_matrix(&projection_matrix),
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

    pub fn frustum(&self) -> &Frustum {
        &self.frustum
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
        self.frustum = Frustum::from_matrix(&self.view_projection_matrix);
    }
}
