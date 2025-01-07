use ambient_api::{
    core::{prefab::components::prefab_from_url, transform::concepts::Transformable},
    prelude::*,
};

use packages::{orbit_camera::concepts::OrbitCamera, this::assets};

pub mod packages;

#[main]
pub async fn main() {
    // Camera
    OrbitCamera::suggested().spawn();

    // Model
    Entity::new()
        .with_merge(Transformable {
            local_to_world: Default::default(),
            optional: Default::default(),
        })
        .with(prefab_from_url(), assets::url("Cube.glb"))
        .spawn();
}
