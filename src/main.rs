mod camera;
mod hittable;
mod material;
mod ray;
mod sphere;

use crate::camera::Camera;
use crate::hittable::{Hittable, HittableList};
use crate::material::{Dielectric, Lambertian, Metal};
use crate::ray::Ray;
use crate::sphere::Sphere;

use material::Material;
use nalgebra::Vector3;
use rand::Rng;
use rayon::prelude::*;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{
    f32,
    io::{self, Write},
};

const LAMBERTIAN_PROP: f32 = 0.7;
const METAL_PROP: f32 = 0.2;

const NX: usize = 1200;
const NY: usize = 800;
const NS: usize = 512;
const MAX_DEPTH: usize = 50;

/// 生成随机场景
fn random_scene() -> HittableList {
    let mut rng = rand::rng();
    let origin = Vector3::new(4.0, 0.2, 0.0);
    let mut scene = HittableList::default();

    // 地面
    scene.push(Sphere::from(
        Vector3::new(0.0, -1000.0, 0.0),
        1000.0,
        Box::new(Lambertian::from(Vector3::new(0.5, 0.5, 0.5))),
    ));

    // 小球
    for a in -11..11 {
        for b in -11..11 {
            let material_rng = rng.random::<f32>();
            let center = Vector3::new(
                a as f32 + 0.9f32 * rng.random::<f32>(),
                0.2,
                b as f32 + 0.9f32 * rng.random::<f32>(),
            );

            if (center - origin).magnitude() > 0.9 {
                let material: Box<dyn Material> = if material_rng < LAMBERTIAN_PROP {
                    // 漫反射材质
                    Box::new(Lambertian::from(Vector3::new(
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>(),
                    )))
                } else if material_rng < LAMBERTIAN_PROP + METAL_PROP {
                    // 金属材质
                    Box::new(Metal::from(
                        Vector3::new(
                            0.5 * (1.0 + rng.random::<f32>()),
                            0.5 * (1.0 + rng.random::<f32>()),
                            0.5 * (1.0 + rng.random::<f32>()),
                        ),
                        0.5 * rng.random::<f32>(),
                    ))
                } else {
                    // 玻璃材质
                    Box::new(Dielectric::from(1.5))
                };

                scene.push(Sphere::from(center, 0.2, material));
            }
        }
    }

    // 大球
    scene.push(Sphere::from(
        Vector3::new(0.0, 1.0, 0.0),
        1.0,
        Box::new(Dielectric::from(1.5)),
    ));

    scene.push(Sphere::from(
        Vector3::new(-4.0, 1.0, 0.0),
        1.0,
        Box::new(Lambertian::from(Vector3::new(0.4, 0.2, 0.1))),
    ));

    scene.push(Sphere::from(
        Vector3::new(4.0, 1.0, 0.0),
        1.0,
        Box::new(Metal::from(Vector3::new(0.7, 0.6, 0.5), 0.0)),
    ));

    scene
}

/// 光线颜色
fn ray_color(ray: &Ray, scene: &HittableList, depth: usize) -> Vector3<f32> {
    if let Some(hit) = scene.hit(ray, 0.001, f32::MAX) {
        if depth < MAX_DEPTH {
            if let Some((scattered, attenuation)) = hit.material.scatter(ray, &hit) {
                return attenuation.zip_map(&ray_color(&scattered, scene, depth + 1), |l, r| l * r);
            }
        }

        Vector3::new(0.0, 0.0, 0.0)
    } else {
        // 背景颜色
        let unit_direction = ray.direction().normalize();
        let t = 0.5 * (unit_direction[1] + 1.0);

        (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0)
    }
}

fn main() -> io::Result<()> {
    // 场景
    let scene = random_scene();

    // 相机参数
    let look_from = Vector3::new(13.0, 2.0, 3.0);
    let look_at = Vector3::new(0.0, 0.0, 0.0);
    let focus_dist = 10.0;
    let aperture = 0.1;

    let cam = Camera::from(
        look_from,
        look_at,
        Vector3::new(0.0, 1.0, 0.0),
        20.0,
        NX as f32 / NY as f32,
        aperture,
        focus_dist,
    );

    // 跟踪渲染进度
    let finished_count = Arc::new(AtomicUsize::new(0));
    let timer = Instant::now();

    // 并行渲染
    let image = (0..NY)
        .into_par_iter()
        .rev()
        .flat_map(|y| {
            let res = (0..NX)
                .flat_map(|x| {
                    // 对每个像素进行多次采样
                    let col: Vector3<f32> = (0..NS)
                        .map(|_| {
                            let mut rng = rand::rng();
                            // 像素内采样
                            let u = (x as f32 + rng.random::<f32>()) / NX as f32;
                            let v = (y as f32 + rng.random::<f32>()) / NY as f32;
                            let ray = cam.camera_ray(u, v);
                            ray_color(&ray, &scene, 0)
                        })
                        .sum();

                    // 颜色值转 u8
                    col.iter()
                        .map(|c| (255.99 * (c / NS as f32).sqrt().clamp(0.0, 1.0)) as u8)
                        .collect::<Vec<u8>>()
                })
                .collect::<Vec<u8>>();

            // 更新进度
            let count = finished_count.fetch_add(1, Ordering::SeqCst) + 1;
            let elapsed = timer.elapsed().as_millis() as usize;
            let avg_speed = elapsed / count;
            let remaining = NY - count;
            eprint!(
                "\rRemaining: {:>4} | ETA: {:>4}s",
                remaining,
                remaining * avg_speed / 1000
            );

            res
        })
        .collect::<Vec<u8>>();

    println!("\nImage ready in {:.1}s", timer.elapsed().as_secs_f32());
    println!("Writing file...");

    // 写入结果
    let image = image
        .chunks(3)
        .map(|col| format!("{} {} {}", col[0], col[1], col[2]))
        .collect::<Vec<_>>()
        .join("\n");

    writeln!(
        &mut File::create("result.ppm")?,
        "P3\n{} {}\n255\n{}",
        NX,
        NY,
        image
    )
}
