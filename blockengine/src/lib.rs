extern crate cgmath;
extern crate lagato;

use {
    std::f32::{NAN},
    cgmath::{Point3, Vector3, InnerSpace},
    lagato::{camera::{Ray}, grid::{Voxels}},
};

pub fn cast_ray(
    ray: &Ray, mut radius: f32, voxels: &Voxels<bool>,
) -> Option<(Point3<i32>, Vector3<i32>)> {
    // Cube containing origin point
    let mut voxel = ray.origin.map(|v| v.floor() as i32);

    // Direction to increment x,y,z when stepping
    let step = ray.direction.map(|v| signum(v));

    // T when reaching the next voxel on an axis
    let mut t_max = Vector3::new(
        intbound(ray.origin.x, ray.direction.x),
        intbound(ray.origin.y, ray.direction.y),
        intbound(ray.origin.z, ray.direction.z),
    );

    // The change in t when taking a step (always positive)
    let t_delta = Vector3::new(
        step.x as f32 / ray.direction.x,
        step.y as f32 / ray.direction.y,
        step.z as f32 / ray.direction.z,
    );

    let mut normal = Vector3::new(0, 0, 0);

    // Avoids an infinite loop
    if ray.direction.x == 0.0 && ray.direction.y == 0.0 && ray.direction.z == 0.0 {
        panic!("Raycast in zero direction")
    }
    if ray.direction.x == NAN || ray.direction.y == NAN || ray.direction.z == NAN {
        panic!("Raycast in NaN direction")
    }

    // Rescale from units of 1 cube-edge to units of 'direction' so we can
    // compare with 't'
    radius /= ray.direction.magnitude();

    while is_in_bounds_step(step, voxels.size(), voxel) {
        // If it's solid, we're done
        if let Ok(true) = voxels.get(voxel) {
            return Some((voxel, normal))
        }

        // t_max.x stores the t-value at which we cross a cube boundary along the
        // X axis, and similarly for Y and Z. Therefore, choosing the least t_max
        // chooses the closest cube boundary. Only the first case of the four
        // has been commented in detail.
        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                if t_max.x > radius { break }
                // Update which cube we are now in.
                voxel.x += step.x;
                // Adjust t_max.x to the next X-oriented boundary crossing.
                t_max.x += t_delta.x;
                // Record the normal vector of the cube face we entered.
                normal = Vector3::new(-step.x, 0, 0);
            } else {
                if t_max.z > radius { break }
                voxel.z += step.z;
                t_max.z += t_delta.z;
                normal = Vector3::new(0, 0, -step.z);
            }
        } else {
            if t_max.y < t_max.z {
                if t_max.y > radius { break }
                voxel.y += step.y;
                t_max.y += t_delta.y;
                normal = Vector3::new(0, -step.y, 0);
            } else {
                // Identical to the second case, repeated for simplicity in
                // the conditionals.
                if t_max.z > radius { break }
                voxel.z += step.z;
                t_max.z += t_delta.z;
                normal = Vector3::new(0, 0, -step.z);
            }
        }
    }

    None
}

fn is_in_bounds_step(step: Vector3<i32>, size: Vector3<i32>, voxel: Point3<i32>) -> bool {
    let x = if step.x > 0 { voxel.x < size.x } else { voxel.x >= 0 };
    let y = if step.y > 0 { voxel.y < size.y } else { voxel.y >= 0 };
    let z = if step.z > 0 { voxel.z < size.z } else { voxel.z >= 0 };
    x && y && z
}

fn signum(x: f32) -> i32 {
    if x > 0.0 {
        1
    } else {
        if x < 0.0 {
            -1
        } else {
            0
        }
    }
}

fn intbound(mut s: f32, ds: f32) -> f32 {
    // Find the smallest positive t such that s+t*ds is an integer
    if ds < 0.0 {
        intbound(-s, -ds)
    } else {
        s = modulus(s, 1.0);
        // problem is now s+t*ds = 1
        (1.0 - s) / ds
    }
}

fn modulus(value: f32, modulus: f32) -> f32 {
    // This is different but I'm not sure in what way
    (value % modulus + modulus) % modulus
}
