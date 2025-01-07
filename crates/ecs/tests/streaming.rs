use std::sync::Arc;

use ambient_ecs::{
    components, ArchetypeFilter, Entity, EntityId, FrozenWorldDiff, Serializable, World, WorldDiff,
    WorldStream, WorldStreamFilter,
};
use itertools::Itertools;

components!("test", {
    @[Serializable]
    a: f32,
    b: f32,
    c: f32,
    no_sync: (),
});

fn init() {
    init_components();
}

#[test]
fn from_a_to_b_diff() {
    init();
    let mut from = World::new_unknown("from_a_to_b_diff");
    Entity::new().with(a(), 5.).with(b(), 2.).spawn(&mut from);
    let to = from.clone();
    let diff = WorldDiff::from_a_to_b(WorldStreamFilter::default(), &from, &to);

    let mut replicated = from.clone();
    diff.apply(&mut replicated, Entity::default());
    assert_eq!(dump_content_string(&replicated), dump_content_string(&to));
}

#[test]
fn from_a_to_b_remove_component() {
    init();
    let mut from = World::new_unknown("from_a_to_b_remove_component");
    let x = Entity::new().with(a(), 5.).with(b(), 2.).spawn(&mut from);
    let y = Entity::new().with(a(), 5.).with(b(), 2.).spawn(&mut from);
    let mut to = from.clone();
    to.remove_component(x, b()).unwrap();
    to.remove_component(y, b()).unwrap();
    let diff = WorldDiff::from_a_to_b(WorldStreamFilter::default(), &from, &to);

    let mut replicated = from.clone();
    diff.apply(&mut replicated, Entity::default());
    assert_eq!(dump_content_string(&replicated), dump_content_string(&to));
}

#[test]
fn streaming() {
    init();
    let mut source =
        World::new_with_config("streaming_src", ambient_ecs::WorldContext::Unknown, true);
    source.init_shape_change_tracking();
    source
        .add_component(source.resource_entity(), no_sync(), ())
        .ok();
    let mut dest = World::new_unknown("streaming_dst");
    let mut stream = WorldStream::new(WorldStreamFilter::new(
        ArchetypeFilter::new().excl(no_sync()),
        Arc::new(|_, _| true),
    ));

    let x = Entity::new().with(a(), 1.).spawn(&mut source);
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new());
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.set(x, a(), 2.).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new());
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.add_component(x, b(), 9.).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new());
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.remove_component(x, a()).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new());
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));

    source.despawn(x).unwrap();
    let diff = stream.next_diff(&source);
    diff.apply(&mut dest, Entity::new());
    assert_eq!(dump_content_string(&source), dump_content_string(&dest));
}

fn dump_content_string(world: &World) -> String {
    let mut entities = world.entities();
    entities.sort_unstable_by_key(|(id, _)| *id);

    entities
        .into_iter()
        .filter_map(|(id, entity)| {
            if id != world.resource_entity() {
                let data = entity
                    .iter()
                    .map(|entry| format!("{:?}: {:?}", entry.path(), entry))
                    .join(", ");
                Some(format!("[{id} {data}]"))
            } else {
                None
            }
        })
        .join(" ")
}

#[test]
fn serialization_of_worlddiff_variants() {
    // Arrange
    init();
    let some_entity = EntityId::new();
    let diff = WorldDiff::new()
        .add_component(some_entity, a(), 5.)
        .add_component(EntityId::new(), a(), 42.)
        .set(some_entity, a(), 10.0);
    let frozen_diff: FrozenWorldDiff = diff.clone().into();

    // Act
    let serialized = bincode::serialize(&diff).unwrap();
    let frozen_serialized = bincode::serialize(&frozen_diff).unwrap();

    // Assert
    assert_eq!(serialized, frozen_serialized);
}
