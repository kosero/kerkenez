use glam::{Mat4, Vec3, Vec4, vec4};

use crate::mesh::AABB;

pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    pub fn from_matrix(m: &Mat4) -> Self {
        let mut planes = [Vec4::ZERO; 6];
        let m = m.to_cols_array_2d();

        // Left plane
        planes[0] = vec4(
            m[0][3] + m[0][0],
            m[1][3] + m[1][0],
            m[2][3] + m[2][0],
            m[3][3] + m[3][0],
        );
        // Right plane
        planes[1] = vec4(
            m[0][3] - m[0][0],
            m[1][3] - m[1][0],
            m[2][3] - m[2][0],
            m[3][3] - m[3][0],
        );
        // Bottom plane
        planes[2] = vec4(
            m[0][3] + m[0][1],
            m[1][3] + m[1][1],
            m[2][3] + m[2][1],
            m[3][3] + m[3][1],
        );
        // Top plane
        planes[3] = vec4(
            m[0][3] - m[0][1],
            m[1][3] - m[1][1],
            m[2][3] - m[2][1],
            m[3][3] - m[3][1],
        );
        // Near plane
        planes[4] = vec4(
            m[0][3] + m[0][2],
            m[1][3] + m[1][2],
            m[2][3] + m[2][2],
            m[3][3] + m[3][2],
        );
        // Far plane
        planes[5] = vec4(
            m[0][3] - m[0][2],
            m[1][3] - m[1][2],
            m[2][3] - m[2][2],
            m[3][3] - m[3][2],
        );

        for plane in &mut planes {
            let length = Vec3::new(plane.x, plane.y, plane.z).length();
            *plane /= length;
        }

        Self { planes }
    }

    pub fn contains_aabb(&self, aabb: &AABB) -> bool {
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
