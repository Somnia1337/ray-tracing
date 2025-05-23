use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use nalgebra::Vector3;
use std::cmp::Ordering;
use std::sync::Arc;

/// 轴对齐包围盒
#[derive(Clone)]
pub struct AaBb {
    /// 最小点
    pub min: Vector3<f32>,

    /// 最大点
    pub max: Vector3<f32>,
}

impl AaBb {
    const fn new() -> Self {
        Self {
            min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    /// 能包裹两个包围盒的最小包围盒
    fn surrounding_box(box0: &Self, box1: &Self) -> Self {
        let small = box0.min.zip_map(&box1.min, f32::min);
        let big = box0.max.zip_map(&box1.max, f32::max);

        Self {
            min: small,
            max: big,
        }
    }

    /// 光线与包围盒相交
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.direction()[a];
            let mut t0 = (self.min[a] - ray.origin()[a]) * inv_d;
            let mut t1 = (self.max[a] - ray.origin()[a]) * inv_d;

            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            let t_min = t0.max(t_min);
            let t_max = t1.min(t_max);

            if t_max <= t_min {
                return false;
            }
        }

        true
    }

    /// 分割轴 (选取最长的轴)
    fn split_axis(&self) -> usize {
        let x = self.max.x - self.min.x;
        let y = self.max.y - self.min.y;
        let z = self.max.z - self.min.z;
        let max = x.max(y).max(z);

        if x == max {
            0
        } else if y == max {
            1
        } else {
            2
        }
    }
}

/// 可被 BVH 管理的有界实体
pub trait Bounded: Hittable + Send {
    /// 实体的包围盒
    fn bounding_box(&self) -> AaBb;
}

/// BVH 节点
pub enum BVHNode {
    /// 叶子节点, 包含一个实体
    Leaf {
        object: Arc<dyn Bounded + Sync + Send>,
    },

    /// 内部节点, 包含左右子树和包围盒
    Node {
        left: Arc<BVHNode>,
        right: Arc<BVHNode>,
        bbox: AaBb,
    },
}

impl BVHNode {
    /// 构建 BVH 树
    pub fn build(mut objects: Vec<Arc<dyn Bounded + Sync + Send>>) -> Self {
        let len = objects.len();

        if len == 1 {
            Self::Leaf {
                object: objects.remove(0),
            }
        } else if len == 2 {
            let left = objects.remove(0);
            let right = objects.remove(0);
            let left_box = left.bounding_box();
            let right_box = right.bounding_box();
            let bbox = AaBb::surrounding_box(&left_box, &right_box);

            Self::Node {
                left: Arc::new(Self::Leaf { object: left }),
                right: Arc::new(Self::Leaf { object: right }),
                bbox,
            }
        } else {
            let mut aabb = AaBb::new();
            for obj in &objects {
                aabb = AaBb::surrounding_box(&aabb, &obj.bounding_box());
            }
            let axis = aabb.split_axis();

            objects.sort_by(|a, b| {
                let box_a = a.bounding_box();
                let box_b = b.bounding_box();

                box_a.min[axis]
                    .partial_cmp(&box_b.min[axis])
                    .unwrap_or(Ordering::Equal)
            });

            let mid = len / 2;
            let right = objects.split_off(mid);
            let left = objects;
            let left_node = Self::build(left);
            let right_node = Self::build(right);
            let bbox = AaBb::surrounding_box(&left_node.bounding_box(), &right_node.bounding_box());

            Self::Node {
                left: Arc::new(left_node),
                right: Arc::new(right_node),
                bbox,
            }
        }
    }

    /// 当前节点的包围盒
    fn bounding_box(&self) -> AaBb {
        match self {
            Self::Leaf { object } => object.bounding_box(),
            Self::Node { bbox, .. } => bbox.clone(),
        }
    }
}

impl Hittable for BVHNode {
    /// 光线与 BVH 节点相交
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self {
            Self::Leaf { object } => object.hit(ray, t_min, t_max),

            Self::Node { left, right, bbox } => {
                if !bbox.hit(ray, t_min, t_max) {
                    return None;
                }

                let left_hit = left.hit(ray, t_min, t_max);
                let t_max = left_hit.as_ref().map_or(t_max, |hit| hit.distance);
                let right_hit = right.hit(ray, t_min, t_max);

                match (left_hit, right_hit) {
                    (Some(l), Some(r)) => {
                        if l.distance < r.distance {
                            Some(l)
                        } else {
                            Some(r)
                        }
                    }
                    (Some(l), None) => Some(l),
                    (None, Some(r)) => Some(r),
                    (None, None) => None,
                }
            }
        }
    }
}
