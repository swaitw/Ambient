use ambient_api::{
    core::{
        prefab::components::prefab_from_url,
        transform::components::{rotation, translation},
    },
    prelude::*,
};
use packages::{
    ambient_example_deps_assets::{assets, components::spin_speed, types::SpinDirection},
    this::{
        components::{spawned_by_us, spin_direction},
        messages::Spawn,
    },
};

pub mod packages;

#[main]
pub fn main() {
    Spawn::subscribe(|_, msg| {
        Entity::new()
            .with(spawned_by_us(), ())
            .with(prefab_from_url(), assets::url("Teapot.glb"))
            .with(spin_direction(), msg.spin_direction)
            .with(spin_speed(), msg.spin_speed)
            .with(translation(), (random::<Vec3>() - 0.5) * 5.0)
            .with(rotation(), Quat::IDENTITY)
            .spawn();
    });

    query((spin_direction(), spin_speed()))
        .requires(spawned_by_us())
        .each_frame(|r| {
            for (id, (dir, speed)) in r {
                entity::mutate_component(id, rotation(), |r| {
                    *r *= Quat::from_rotation_z(
                        speed.abs()
                            * delta_time()
                            * match dir {
                                SpinDirection::Forward => 1.0,
                                SpinDirection::Backward => -1.0,
                            },
                    )
                });
            }
        });
}
