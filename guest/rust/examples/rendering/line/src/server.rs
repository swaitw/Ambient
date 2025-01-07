use ambient_api::{
    core::{
        app::components::{name, ui_scene},
        primitives::components::quad,
        rect::components::{line_width, pixel_line_from, pixel_line_to},
        transform::components::scale,
    },
    prelude::*,
};
use packages::orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional};

pub mod packages;

#[main]
pub fn main() {
    main2();
}
fn main2() {
    OrbitCamera {
        is_orbit_camera: (),
        optional: OrbitCameraOptional {
            lookat_target: Some(Vec3::Z),
            camera_angle: Some(vec2(0., 20f32.to_radians())),
            camera_distance: Some(20.),
        },
    }
    .make()
    .spawn();

    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 20.)
        .spawn();

    Entity::new()
        .with(name(), "The line".to_string())
        .with(pixel_line_from(), vec3(0., 0., 1.))
        .with(pixel_line_to(), vec3(10., 0., 1.))
        .with(line_width(), 10.)
        .with(ui_scene(), ())
        .spawn();
}
