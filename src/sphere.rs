use crate::bvh::{AaBb, Bounded};
use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use nalgebra::Vector3;

/// 球体
pub struct Sphere {
    /// 球心
    center: Vector3<f32>,

    /// 半径
    radius: f32,

    /// 材质
    material: Box<dyn Material>,
}

impl Sphere {
    pub fn from(center: Vector3<f32>, radius: f32, material: Box<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    pub fn clone_sphere(&self) -> Self {
        Self {
            center: self.center,
            radius: self.radius,
            material: self.material.clone(),
        }
    }
}

impl Hittable for Sphere {
    /// 光线是否与球体相交
    ///
    /// 用二次方程求解光线与球体的交点,
    /// (P(t) - C) · (P(t) - C) = r * r,
    /// 其中 P(t) 为光线上的点, C 为球心, r 为半径
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // 光线起点到球心的向量
        let oc = ray.origin() - self.center;

        // 方程系数
        let d = ray.direction();
        let a = d.dot(&d);
        let b = oc.dot(&d);
        let c = oc.dot(&oc) - self.radius * self.radius;

        // 判定式
        let disc = b.powi(2) - a * c;

        if disc > 0.0 {
            let sqrt_disc = disc.sqrt();

            // 交点 1
            let t = (-b - sqrt_disc) / a;
            if t > t_min && t < t_max {
                let p = ray.point_at_t(t);
                let normal = (p - self.center) / self.radius;

                return Some(HitRecord {
                    distance: t,
                    position: p,
                    normal,
                    material: &*self.material,
                });
            }

            // 交点 2
            let t = (-b + sqrt_disc) / a;
            if t > t_min && t < t_max {
                let p = ray.point_at_t(t);
                let normal = (p - self.center) / self.radius;

                return Some(HitRecord {
                    distance: t,
                    position: p,
                    normal,
                    material: &*self.material,
                });
            }
        }

        None
    }
}

impl Bounded for Sphere {
    fn bounding_box(&self) -> AaBb {
        let min = self.center - Vector3::new(self.radius, self.radius, self.radius);
        let max = self.center + Vector3::new(self.radius, self.radius, self.radius);

        AaBb { min, max }
    }
}
