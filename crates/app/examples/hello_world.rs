use ambient_app::{App, AppBuilder};
use ambient_core::{
    camera::active_camera,
    main_scene,
    transform::{scale, translation},
};
use ambient_element::ElementComponentExt;
use ambient_native_std::math::SphericalCoords;
use ambient_primitives::{Cube, Quad};
use ambient_renderer::{cast_shadows, color, outline};
use glam::{vec3, vec4, Vec3, Vec4};

async fn init(app: &mut App) {
    let world = &mut app.world;

    Cube.el()
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with(translation(), Vec3::Z)
        .with(cast_shadows(), ())
        .with(outline(), Vec4::ONE)
        .spawn_static(world);
    Quad.el().with(scale(), Vec3::ONE * 10.).spawn_static(world);

    ambient_cameras::spherical::new(
        vec3(0., 0., 0.),
        SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.),
    )
    .with(active_camera(), 0.)
    .with(main_scene(), ())
    .spawn(world);
}

fn main() {
    // wgpu_subscriber::initialize_default_subscriber(None);
    AppBuilder::simple().block_on(init);
}
