mod bvh;
mod camera;
mod hittable;
mod material;
mod ray;
mod rng;
mod sphere;

use crate::bvh::{BVHNode, Bounded};
use crate::camera::Camera;
use crate::hittable::{Hittable, HittableList};
use crate::material::{Dielectric, Lambertian, Metal};
use crate::ray::Ray;
use crate::rng::get_rng;
use crate::sphere::Sphere;

use material::Material;
use nalgebra::Vector3;
use rand::Rng;
use rand::seq::IndexedRandom;
use rayon::prelude::*;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{
    f32,
    io::{self, Write},
};

// 材质比例
const LAMBERTIAN_PROP: usize = 10;
const METAL_PROP: usize = 3;
const DIELECTRIC_PROP: usize = 2;

// 图像属性
const NX: usize = 1200;
const NY: usize = 800;
const NS: usize = 10;
const MAX_DEPTH: usize = 50;

/// 生成随机场景
fn random_scene() -> HittableList {
    let mut rng = get_rng();
    let origin = Vector3::new(4.0, 0.2, 0.0);
    let mut scene = HittableList::default();

    // 地面
    scene.push(Sphere::from(
        Vector3::new(0.0, -1000.0, 0.0),
        1000.0,
        Box::new(Lambertian::from(Vector3::new(0.5, 0.5, 0.5))),
    ));

    let mut materials_list = vec![];
    materials_list.extend(std::iter::repeat_n(0, LAMBERTIAN_PROP));
    materials_list.extend(std::iter::repeat_n(1, METAL_PROP));
    materials_list.extend(std::iter::repeat_n(2, DIELECTRIC_PROP));

    // 小球
    for a in -11..11 {
        for b in -11..11 {
            let center = Vector3::new(
                a as f32 + 0.9 * rng.random::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.random::<f32>(),
            );

            if (center - origin).magnitude() > 0.9 {
                let material_pick = *materials_list.choose(&mut rng).unwrap();

                let material: Box<dyn Material> = if material_pick == 0 {
                    Box::new(Lambertian::from(Vector3::new(
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>(),
                        rng.random::<f32>() * rng.random::<f32>(),
                    )))
                } else if material_pick == 1 {
                    Box::new(Metal::from(
                        Vector3::new(
                            0.5 * (1.0 + rng.random::<f32>()),
                            0.5 * (1.0 + rng.random::<f32>()),
                            0.5 * (1.0 + rng.random::<f32>()),
                        ),
                        0.5 * rng.random::<f32>(),
                    ))
                } else {
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
fn ray_color(ray: &Ray, scene: &impl Hittable, depth: usize) -> Vector3<f32> {
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
    eprint!("Constructing scene...");
    let scene_list = random_scene();
    eprintln!("\rScene constructed{}", " ".repeat(10));

    // 构建 BVH
    eprint!("Building BVH...");
    let objects: Vec<_> = scene_list
        .list
        .into_iter()
        .filter_map(|obj| {
            let hittable_ref = obj.as_ref();
            (hittable_ref as &dyn std::any::Any)
                .downcast_ref::<Sphere>()
                .map(|sphere| Arc::new(sphere.clone_sphere()) as Arc<dyn Bounded + Sync + Send>)
        })
        .collect();
    let scene = BVHNode::build(objects);
    eprintln!("\rBVH built{}", " ".repeat(10));

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
            let rng = &mut get_rng();
            let res = (0..NX)
                .flat_map(|x| {
                    // 对每个像素进行多次采样
                    let mut col = Vector3::zeros();
                    for _ in 0..NS {
                        let u = (x as f32 + rng.random::<f32>()) / NX as f32;
                        let v = (y as f32 + rng.random::<f32>()) / NY as f32;

                        let ray = cam.camera_ray(u, v);
                        col += ray_color(&ray, &scene, 0);
                    }

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

    eprintln!(
        "\rRendered in {:.1}s{}",
        timer.elapsed().as_secs_f32(),
        " ".repeat(20)
    );
    eprint!("Writing file...");

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
    )?;

    eprintln!("\rFile written{}", " ".repeat(10));

    Ok(())
}
