use std::collections::HashMap;

use ambient_ecs::{query, Entity, System, WorldDiff};
use ambient_native_std::friendly_id;
use ambient_rpc::RpcRegistry;
use serde::{Deserialize, Serialize};

use crate::{
    server::{
        create_player_entity_data, player_connection_id, player_entity_stream, player_transport,
        ForkingEvent, RpcArgs as ServerRpcArgs, WorldInstance, MAIN_INSTANCE_ID,
    },
    ServerWorldExt,
};
use ambient_core::player::user_id;

pub fn register_server_rpcs(reg: &mut RpcRegistry<ServerRpcArgs>) {
    reg.register(rpc_world_diff);
    reg.register(rpc_fork_instance);
    reg.register(rpc_join_instance);
    reg.register(rpc_get_instances_info);
}

pub async fn rpc_world_diff(args: ServerRpcArgs, diff: WorldDiff) {
    diff.apply(
        &mut args
            .state
            .lock()
            .get_player_world_instance_mut(&args.user_id)
            .unwrap()
            .world,
        Entity::new(),
    );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcForkInstance {
    pub resources: Entity,
    pub synced_res: Entity,
    pub id: Option<String>,
}

/// This clones the current world instance of the player, and returns the id to the new instance.
pub async fn rpc_fork_instance(
    args: ServerRpcArgs,
    RpcForkInstance {
        resources,
        synced_res,
        id,
    }: RpcForkInstance,
) -> String {
    let mut state = args.state.lock();
    let id = id.unwrap_or(friendly_id());
    if !state.instances.contains_key(&id) {
        let new_instance = {
            let instance = state.get_player_world_instance(&args.user_id).unwrap();
            let mut world = instance.world.clone();

            for (id, _) in query(user_id()).collect_cloned(&world, None) {
                world.despawn(id);
            }
            world
                .add_components(
                    world.resource_entity(),
                    resources.with_merge(ambient_core::async_ecs::async_ecs_resources()),
                )
                .unwrap();
            world
                .add_components(world.synced_resource_entity().unwrap(), synced_res)
                .unwrap();

            let mut on_forking = (state.create_on_forking_systems)();
            on_forking.run(&mut world, &ForkingEvent);

            world.reset_events();

            WorldInstance {
                systems: (state.create_server_systems)(&mut world),
                world,
                world_stream: instance.world_stream.clone(),
            }
        };
        state.instances.insert(id.clone(), new_instance);
    }
    id
}
pub async fn rpc_join_instance(args: ServerRpcArgs, new_instance_id: String) {
    let mut state = args.state.lock();
    let old_instance_id = state.players.get(&args.user_id).unwrap().instance.clone();
    if old_instance_id == new_instance_id {
        return;
    }

    let instances = &mut state.instances;

    // Borrow the new world mutably to broadcast its diffs.
    instances
        .get_mut(&new_instance_id)
        .unwrap()
        .broadcast_diffs();

    // Borrow both worlds immutably to extract the old world's player count and the diff between the two, and
    // to broadcast the latest diffs for the new instance.
    let (old_player_count, diff) = {
        let (old_instance, new_instance) = instances
            .get(&old_instance_id)
            .zip(instances.get(&new_instance_id))
            .unwrap();
        (
            old_instance.player_count(),
            WorldDiff::from_a_to_b(
                old_instance.world_stream.filter().clone(),
                &old_instance.world,
                &new_instance.world,
            ),
        )
    };

    // Borrow the old world mutably to remove the player and their streams.
    let entities_tx;
    let connection_id;
    let conn;

    {
        let mut ed = instances
            .get_mut(&old_instance_id)
            .unwrap()
            .despawn_player(&args.user_id)
            .unwrap();
        entities_tx = ed.remove_self(player_entity_stream()).unwrap();

        connection_id = ed.remove_self(player_connection_id()).unwrap();
        conn = ed.remove_self(player_transport()).unwrap();
    };

    // Borrow the new world mutably to spawn the player in with their old streams.
    instances
        .get_mut(&new_instance_id)
        .unwrap()
        .spawn_player(create_player_entity_data(
            conn,
            args.user_id.clone(),
            entities_tx.clone(),
            connection_id,
        ));
    state.players.get_mut(&args.user_id).unwrap().instance = new_instance_id.to_string();

    entities_tx.send(diff.into()).ok();

    // Remove old instance
    if old_player_count == 1 && old_instance_id != MAIN_INSTANCE_ID {
        state.remove_instance(&old_instance_id);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancesInfo {
    pub instances: HashMap<String, InstanceInfo>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub n_players: u32,
}

#[allow(clippy::let_unit_value)]
pub async fn rpc_get_instances_info(args: ServerRpcArgs, _: ()) -> InstancesInfo {
    let state = args.state.lock();
    InstancesInfo {
        instances: state
            .instances
            .iter()
            .map(|(key, instance)| {
                (
                    key.clone(),
                    InstanceInfo {
                        n_players: instance.player_count() as u32,
                    },
                )
            })
            .collect(),
    }
}
