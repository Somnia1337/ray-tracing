use crate::hittable::HitRecord;
use crate::ray::Ray;

use nalgebra::Vector3;
use rand::Rng;

/// 单位球内的随机点
fn random_in_unit_sphere() -> Vector3<f32> {
    let mut rng = rand::rng();
    let unit = Vector3::new(1.0, 1.0, 1.0);

    loop {
        // 拒绝采样法
        let p =
            2.0 * Vector3::new(
                rng.random::<f32>(),
                rng.random::<f32>(),
                rng.random::<f32>(),
            ) - unit;
        if p.magnitude_squared() < 1.0 {
            return p;
        }
    }
}

/// 反射向量
///
/// R = v - 2 * (v · n) * n
fn reflect(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * v.dot(n) * n
}

/// 折射向量
///
/// 全反射时为 None
fn refract(v: &Vector3<f32>, n: &Vector3<f32>, ni_over_nt: f32) -> Option<Vector3<f32>> {
    let uv = v.normalize();
    let dt = uv.dot(n);
    let discriminant = 1.0 - ni_over_nt.powi(2) * (1.0 - dt.powi(2));

    if discriminant > 0.0 {
        let refracted = ni_over_nt * (uv - n * dt) - n * discriminant.sqrt();
        Some(refracted)
    } else {
        None
    }
}

/// Schlick 近似下的反射系数
fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);

    (1.0 - r0) * (1.0 - cosine).powi(5) + r0
}

/// 材质
pub trait Material: Sync {
    /// 光线散射
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f32>)>;
}

/// 漫反射材质
pub struct Lambertian {
    /// 反射率
    albedo: Vector3<f32>,
}

impl Lambertian {
    pub const fn from(albedo: Vector3<f32>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f32>)> {
        // 随机反射
        let target = hit.position + hit.normal + random_in_unit_sphere();
        let scattered = Ray::from(hit.position, target - hit.position);

        Some((scattered, self.albedo))
    }
}

/// 金属材质
pub struct Metal {
    /// 反射率
    albedo: Vector3<f32>,

    /// 模糊
    fuzz: f32,
}

impl Metal {
    pub fn from(albedo: Vector3<f32>, fuzz: f32) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f32>)> {
        let mut reflected = reflect(&ray.direction().normalize(), &hit.normal);

        // 模糊
        if self.fuzz > 0.0 {
            reflected += self.fuzz * random_in_unit_sphere();
        }

        // 检查反射方向是否在表面上方
        if reflected.dot(&hit.normal) > 0.0 {
            let scattered = Ray::from(hit.position, reflected);
            Some((scattered, self.albedo))
        } else {
            None
        }
    }
}

/// 电介质材质 (玻璃)
pub struct Dielectric {
    /// 折射率
    ref_idx: f32,
}

impl Dielectric {
    pub const fn from(ref_idx: f32) -> Self {
        Self { ref_idx }
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<(Ray, Vector3<f32>)> {
        let attenuation = Vector3::new(1.0, 1.0, 1.0);

        // 入射方向 (从空气到材质或从材质到空气)
        let (outward_normal, ni_over_nt, cosine) = if ray.direction().dot(&hit.normal) > 0.0 {
            let cosine =
                self.ref_idx * ray.direction().dot(&hit.normal) / ray.direction().magnitude();
            (-hit.normal, self.ref_idx, cosine)
        } else {
            let cosine = -ray.direction().dot(&hit.normal) / ray.direction().magnitude();
            (hit.normal, 1.0 / self.ref_idx, cosine)
        };

        // 尝试折射
        if let Some(refracted) = refract(&ray.direction(), &outward_normal, ni_over_nt) {
            let reflect_prob = schlick(cosine, self.ref_idx);
            if rand::rng().random::<f32>() >= reflect_prob {
                let scattered = Ray::from(hit.position, refracted);
                return Some((scattered, attenuation));
            }
        }

        let reflected = reflect(&ray.direction(), &hit.normal);
        let scattered = Ray::from(hit.position, reflected);

        Some((scattered, attenuation))
    }
}
