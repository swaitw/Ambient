use ambient_api::{
    core::{
        physics::components::{plane_collider, sphere_collider},
        player::components::is_player,
        primitives::{components::quad, concepts::Sphere},
        rendering::components::color,
        transform::components::{scale, translation},
    },
    prelude::*,
};
use packages::third_person_controller::{
    components::camera_distance, concepts::ThirdPersonController,
};

pub mod packages;

#[main]
pub fn main() {
    Entity::new()
        .with(quad(), ())
        .with(scale(), Vec3::ONE * 10.)
        .with(color(), vec4(1., 0., 0., 1.))
        .with(plane_collider(), ())
        .spawn();

    Entity::new()
        .with_merge(Sphere::suggested())
        .with(sphere_collider(), 0.5)
        .with(translation(), vec3(5., 5., 1.))
        .spawn();

    spawn_query(is_player()).bind(move |players| {
        for (id, _) in players {
            entity::add_components(
                id,
                Entity::new()
                    .with_merge(ThirdPersonController::suggested())
                    .with(camera_distance(), 0.0),
            );
        }
    });
}
