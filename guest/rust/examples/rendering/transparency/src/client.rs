use std::f32::consts::{FRAC_PI_4, FRAC_PI_6};

use ambient_api::{
    core::{
        primitives::components::{cube, quad},
        rendering::components::{color, transparency_group},
        transform::components::{scale, translation},
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

pub mod packages;

#[main]
fn main() {
    // Camera
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            camera_angle: Some(vec2(-FRAC_PI_4, FRAC_PI_6)),
            camera_distance: Some(5.),
            ..default()
        },
    }
    .spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .spawn();

    Entity::new()
        .with(cube(), ())
        .with(translation(), vec3(0., 0., 1.))
        .with(scale(), Vec3::ONE * 2.)
        .with(color(), vec4(0., 1., 0., 0.5))
        .with(transparency_group(), 0)
        .spawn();
}
