use crate::mesh::Vertex;
use glam::{Vec4, vec4};

pub struct AABB {
    pub min: Vec4,
    pub max: Vec4,
}

impl AABB {
    pub fn from_vertices(vertices: &[Vertex]) -> Self {
        let mut min = Vec4::splat(f32::MAX);
        let mut max = Vec4::splat(f32::MIN);

        for v in vertices {
            let pos = Vec4::from_array(v.position);

            min = min.min(pos);
            max = max.max(pos);
        }

        Self { min, max }
    }

    pub fn transfrom(&self, matrix: &glam::Mat4) -> Self {
        let corners = [
            vec4(self.min.x, self.min.y, self.min.z, 1.0),
            vec4(self.min.x, self.min.y, self.max.z, 1.0),
            vec4(self.min.x, self.max.y, self.min.z, 1.0),
            vec4(self.min.x, self.max.y, self.max.z, 1.0),
            vec4(self.max.x, self.min.y, self.min.z, 1.0),
            vec4(self.max.x, self.min.y, self.max.z, 1.0),
            vec4(self.max.x, self.max.y, self.min.z, 1.0),
            vec4(self.max.x, self.max.y, self.max.z, 1.0),
        ];

        let mut min = Vec4::splat(f32::MAX);
        let mut max = Vec4::splat(f32::MIN);

        for &c in &corners {
            let res_v3 = matrix.transform_point3(c.truncate());

            let transformed = res_v3.extend(1.0);

            min = min.min(transformed);
            max = max.max(transformed);
        }

        Self { min, max }
    }
}
