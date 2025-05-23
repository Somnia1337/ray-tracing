use crate::material::Material;
use crate::ray::Ray;

use nalgebra::Vector3;
use std::any::Any;

/// 光线与实体的相交
pub struct HitRecord<'a> {
    /// 交点到光线起点的距离
    pub distance: f32,

    /// 交点的位置
    pub position: Vector3<f32>,

    /// 交点处的表面法线
    pub normal: Vector3<f32>,

    /// 交点处的材质
    pub material: &'a dyn Material,
}

/// 可被光线击中
pub trait Hittable: Sync + Any + 'static {
    /// 光线与实体相交
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

/// 可击中实体列表
#[derive(Default)]
pub struct HittableList {
    pub list: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn push(&mut self, hittable: impl Hittable + 'static) {
        self.list.push(Box::new(hittable));
    }
}

impl Hittable for HittableList {
    /// 光线是否与列表中的任何实体相交, 返回最近的相交点信息
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest = t_max;
        let mut closest_hit: Option<HitRecord> = None;

        // 与列表中实体最近的相交点
        for h in &self.list {
            if let Some(hit) = h.hit(ray, t_min, closest) {
                closest = hit.distance;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }
}
