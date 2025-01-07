use crate::{
    collider::{collider_shapes, collider_shapes_convex},
    main_physics_scene,
    physx::physics_shape,
    picking_scene, trigger_areas_scene,
};
use ambient_core::{
    transform::{get_world_transform, local_to_world},
    ui_scene,
};
use ambient_ecs::{
    components, dont_store, ensure_has_component, ensure_has_component_with_default,
    generated::rect::components::{background_color, line_width, pixel_line_from, pixel_line_to},
    query, Entity, EntityId, FnSystem, SystemGroup, World,
};
use ambient_gizmos::{local_gizmos, GizmoPrimitive};

use glam::{vec4, Vec3};
use itertools::Itertools;
use physxx::{
    PxActor, PxDebugLine, PxRenderBuffer, PxRigidActor, PxShape, PxShapeFlag,
    PxVisualizationParameter,
};

pub use ambient_ecs::generated::physics::components::visualize_collider;

components!("physics", {
    physx_viz_line: PxDebugLine,
});

pub fn run_visualize_collider(world: &mut World, entity: EntityId, enabled: bool) -> Option<()> {
    for shape in world
        .get_ref(entity, collider_shapes())
        .into_iter()
        .flatten()
    {
        visualize_shape(shape, enabled);
    }

    for shape in world
        .get_ref(entity, collider_shapes_convex())
        .into_iter()
        .flatten()
    {
        visualize_shape(shape, enabled);
    }

    if enabled {
        world
            .add_component(entity, visualize_collider(), ())
            .unwrap();
    } else {
        world.remove_component(entity, visualize_collider()).ok();
    }
    Some(())
}

fn visualize_shape(shape: &PxShape, enabled: bool) -> Option<()> {
    let scene = shape.get_actor()?.get_scene()?;
    shape.set_flag(PxShapeFlag::VISUALIZATION, enabled);

    scene.set_visualization_parameter(PxVisualizationParameter::SCALE, 10.0);
    scene.set_visualization_parameter(PxVisualizationParameter::JOINT_LOCAL_FRAMES, 1.0);
    scene.set_visualization_parameter(PxVisualizationParameter::JOINT_LIMITS, 1.0);
    // scene.set_visualization_parameter(PxVisualizationParameter::ACTOR_AXES, 1.0);
    scene.set_visualization_parameter(PxVisualizationParameter::COLLISION_SHAPES, 1.0);
    Some(())
}

pub(crate) fn server_systems() -> SystemGroup {
    SystemGroup::new(
        "visualization/server",
        vec![
            // This is needed as duplicating an object does not carry over the physx flags, but
            // the object is still recognized as visualize_collider
            query((visualize_collider(), collider_shapes().changed())).to_system_with_name(
                "visualization/ensure_visualization",
                |q, w, qs, _| {
                    for id in q.collect_ids(w, qs) {
                        run_visualize_collider(w, id, true);
                    }
                },
            ),
            query((visualize_collider(), collider_shapes()))
                .spawned()
                .to_system_with_name(
                    "visualization/ensure_visualization_spawned",
                    |q, w, qs, _| {
                        for id in q.collect_ids(w, qs) {
                            run_visualize_collider(w, id, true);
                        }
                    },
                ),
            query(visualize_collider()).despawned().to_system_with_name(
                "visualization/despawned",
                |q, w, qs, _| {
                    profiling::scope!("server_shape_visualize_remove");
                    let mut ids = Vec::new();
                    for (id, _) in q.iter(w, qs) {
                        ids.push(id);
                    }

                    for id in ids {
                        tracing::debug!("Removing visualize_collider from {id:?}");
                        w.remove_component(id, local_gizmos()).ok();
                    }
                },
            ),
            query(visualize_collider()).to_system_with_name(
                "visualization/shapes",
                |q, w, qs, _| {
                    profiling::scope!("server_shape_visualize");
                    let mut primitives = Vec::new();

                    for (id, ()) in q.iter(w, qs) {
                        let ltw = get_world_transform(w, id).unwrap_or_default();

                        let mut current = Vec::new();

                        let (_, _, pos) = ltw.to_scale_rotation_translation();

                        current.push(GizmoPrimitive::sphere(pos, 0.15).with_color(Vec3::X));

                        if let Ok(shape) = w.get_ref(id, physics_shape()) {
                            let actor = shape.get_actor().unwrap();
                            current.push(
                                GizmoPrimitive::sphere(actor.get_global_pose().translation(), 0.1)
                                    .with_color(Vec3::Y),
                            );
                        }

                        primitives.push((id, current))
                    }

                    for (id, p) in primitives {
                        w.add_component(id, local_gizmos(), p)
                            .expect("Invalid component");
                    }
                },
            ),
            Box::new(FnSystem::new(|world, _| {
                let mut render_buffer = PxRenderBuffer::default();
                for scene in [main_physics_scene(), picking_scene(), trigger_areas_scene()] {
                    let scene = world.resource(scene);
                    if scene.get_visualization_parameter(PxVisualizationParameter::SCALE) > 0. {
                        let rb = scene.get_render_buffer();
                        render_buffer.points.extend(rb.points.into_iter());
                        render_buffer.lines.extend(rb.lines.into_iter());
                    }
                }

                let existing = query(())
                    .incl(physx_viz_line())
                    .iter(world, None)
                    .map(|(id, _)| id)
                    .collect_vec();
                for (entity, line) in existing.iter().zip(render_buffer.lines.iter()) {
                    let _ = world.set_if_changed(*entity, physx_viz_line(), line.clone());
                }

                #[allow(clippy::comparison_chain)]
                if render_buffer.lines.len() > existing.len() {
                    for i in existing.len()..render_buffer.lines.len() {
                        Entity::new()
                            .with(dont_store(), ())
                            .with(physx_viz_line(), render_buffer.lines[i].clone())
                            .spawn(world);
                    }
                } else if existing.len() > render_buffer.lines.len() {
                    for entity in existing.iter().skip(render_buffer.lines.len()) {
                        world.despawn(*entity);
                    }
                }
            })),
            Box::new(SystemGroup::new(
                "visualization/server_lines",
                vec![
                    ensure_has_component_with_default(physx_viz_line(), pixel_line_from()),
                    ensure_has_component_with_default(physx_viz_line(), pixel_line_to()),
                    ensure_has_component(physx_viz_line(), line_width(), 1.),
                    ensure_has_component(
                        physx_viz_line(),
                        background_color(),
                        vec4(1., 0., 0., 1.),
                    ),
                    ensure_has_component_with_default(physx_viz_line(), ui_scene()),
                    ensure_has_component_with_default(physx_viz_line(), local_to_world()),
                    query((physx_viz_line(),)).to_system(|q, world, qs, _| {
                        for (id, (line,)) in q.collect_cloned(world, qs) {
                            let from = line.pos0;
                            let to = line.pos1;
                            world.set_if_changed(id, pixel_line_from(), from).unwrap();
                            world.set_if_changed(id, pixel_line_to(), to).unwrap();
                        }
                    }),
                ],
            )),
        ],
    )
}
