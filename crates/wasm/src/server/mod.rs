use crate::shared::{self, message::MessageExt};
use ambient_ecs::{generated::messages, query, EntityId, FnSystem, SystemGroup, World};
use ambient_native_std::asset_cache::AssetCache;
use ambient_network::server::{ForkingEvent, ShutdownEvent};
use std::{path::PathBuf, sync::Arc};

mod implementation;
mod network;

pub fn initialize(
    world: &mut World,
    assets: &AssetCache,
    hosted: bool,
    data_path: PathBuf,
    messenger: Arc<dyn Fn(&World, EntityId, shared::MessageType, &str) + Send + Sync>,
) -> anyhow::Result<()> {
    shared::initialize(
        world,
        assets,
        messenger,
        Arc::new(move |id| Bindings {
            base: Default::default(),
            world_ref: Default::default(),
            id,
            reqwest_client: reqwest::Client::new(),
            last_http_request_id: 0,
            hosted,
        }),
        if hosted {
            None
        } else {
            Some(data_path.as_ref())
        },
    )?;

    network::initialize(world);

    Ok(())
}

pub fn systems() -> SystemGroup {
    SystemGroup::new(
        "core/wasm/server",
        vec![
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module collision event");
                // trigger collision event
                let collisions = match world.resource_opt(ambient_physics::collisions()) {
                    Some(collisions) => collisions.lock().clone(),
                    None => return,
                };
                for (a, b, positions, normals) in collisions.into_iter() {
                    messages::Collision::new(vec![a, b], positions, normals)
                        .run(world, None)
                        .unwrap();
                }
            })),
            Box::new(FnSystem::new(move |world, _| {
                profiling::scope!("WASM module collider loads");
                // trigger collider loads
                let collider_loads = match world.resource_opt(ambient_physics::collider_loads()) {
                    Some(collider_loads) => collider_loads.clone(),
                    None => return,
                };

                if collider_loads.is_empty() {
                    return;
                }

                messages::ColliderLoads::new(collider_loads)
                    .run(world, None)
                    .unwrap();
            })),
            Box::new(shared::systems()),
        ],
    )
}

pub fn on_forking_systems() -> SystemGroup<ForkingEvent> {
    SystemGroup::new(
        "core/wasm/server/on_forking_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            // Reset the states of all the modules when we fork.
            shared::reload_all(world);
        }))],
    )
}

pub fn on_shutdown_systems() -> SystemGroup<ShutdownEvent> {
    SystemGroup::new(
        "core/wasm/server/on_shutdown_systems",
        vec![Box::new(FnSystem::new(move |world, _| {
            let modules = query(()).incl(shared::is_module()).collect_ids(world, None);
            for module_id in modules {
                shared::unload(world, module_id, "shutting down");
            }
        }))],
    )
}

#[derive(Clone)]
struct Bindings {
    base: shared::bindings::BindingsBase,
    world_ref: shared::bindings::WorldRef,
    id: EntityId,
    reqwest_client: reqwest::Client,
    last_http_request_id: u64,
    /// Whether or not this server is running in a hosted environment,
    /// and should thus have some of its functionality disabled
    hosted: bool,
}

impl Bindings {
    pub fn world(&self) -> &World {
        unsafe { self.world_ref.world() }
    }
    pub fn world_mut(&mut self) -> &mut World {
        unsafe { self.world_ref.world_mut() }
    }
}

impl shared::bindings::BindingsBound for Bindings {
    fn base(&self) -> &shared::bindings::BindingsBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut shared::bindings::BindingsBase {
        &mut self.base
    }
    fn set_world(&mut self, world: &mut World) {
        unsafe {
            self.world_ref.set_world(world);
        }
    }
    fn clear_world(&mut self) {
        unsafe {
            self.world_ref.clear_world();
        }
    }
}
