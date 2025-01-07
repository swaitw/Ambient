use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    iter::once,
    sync::atomic::{AtomicU64, Ordering},
};

use ambient_native_std::sparse_vec::SparseVec;
use ambient_shared_types::ComponentIndex;
use bit_set::BitSet;
use bit_vec::BitVec;
use itertools::Itertools;
/// Expose to macros
#[doc(hidden)]
pub use once_cell::sync::{Lazy, OnceCell};
/// Expose to macros
#[doc(hidden)]
pub use parking_lot;
use parking_lot::Mutex;
/// Expose to macros
#[doc(hidden)]
pub use paste;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod generated;

mod archetype;
mod attributes;
pub mod component;
mod component_entry;
mod component_registry;
mod component_ser;
mod component_traits;
mod entity;
mod events;
mod helpers;
mod index;
mod location;
mod message_serde;
mod primitive_component;
mod query;
mod serialization;
mod stream;
pub use ambient_package_rt::message_serde::*;
pub use archetype::*;
pub use attributes::*;
pub use component::{Component, ComponentDesc, ComponentValue, ComponentValueBase, EnumComponent};
pub use component_entry::*;
pub use component_registry::*;
pub use component_ser::*;
pub use entity::*;
pub use events::*;
pub use helpers::*;
pub use index::*;
pub use location::*;
pub use message_serde::*;
pub use primitive_component::*;
pub use query::*;
pub use serialization::*;
pub use stream::*;

pub struct DebugWorldArchetypes<'a> {
    world: &'a World,
}

impl<'a> Debug for DebugWorldArchetypes<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_map();

        for arch in &self.world.archetypes {
            s.entry(&arch.id, &arch.dump_info());
        }

        s.finish()
    }
}

mod internal_components {
    use super::Message;

    use crate::{
        components, Description, Resource, WorldEventReader, WorldEventSource, WorldEvents,
    };

    pub trait WorldEventsExt {
        fn add_message<M: Message>(&mut self, message: M);
    }

    impl WorldEventsExt for WorldEvents {
        fn add_message<M: Message>(&mut self, message: M) {
            self.add_event((
                WorldEventSource::Runtime,
                M::id().to_string(),
                message.serialize_message().unwrap(),
            ));
        }
    }

    pub fn read_messages<M: Message>(
        reader: &mut WorldEventReader,
        events: &WorldEvents,
    ) -> Vec<M> {
        reader
            .iter(events)
            .filter(|(_, (_, name, _))| *name == M::id())
            .map(|(_, (_, _, event))| M::deserialize_message(event).unwrap())
            .collect()
    }

    components!("ecs", {
        @[
            Resource,
            Description["A global general event queue for this ecs World. Can be used to dispatch or listen to any kinds of events."]
        ]
        world_events: WorldEvents,
    });
}
pub use generated::ecs::components::*;
pub use internal_components::{read_messages, world_events, WorldEventsExt};

pub fn init_components() {
    generated::init();
    internal_components::init_components();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Indicates what the intended purpose for a [`World`] is
pub enum WorldContext {
    /// The world is running on the server
    Server,
    /// The world is running on the client
    Client,
    /// The world is running in an app, and may be used to present a client world
    App,
    /// The world exists for a prefab that can be instantiated
    Prefab,
    /// The context of this world is unimportant (tests, etc)
    Unknown,
}

#[derive(Clone)]
pub struct World {
    name: &'static str,
    context: WorldContext,
    archetypes: Vec<Archetype>,
    locs: HashMap<EntityId, EntityLocation, EntityIdHashBuilder>,
    loc_changed: FramedEvents<EntityId>,
    version: CloneableAtomicU64,
    shape_change_events: Option<FramedEvents<WorldChange>>,
    /// Used for reset_events. Prevents change events in queries when you use reset_events
    ignore_query_inits: bool,
    query_ticker: CloneableAtomicU64,
}
impl World {
    pub fn new_unknown(name: &'static str) -> Self {
        Self::new(name, WorldContext::Unknown)
    }
    pub fn new(name: &'static str, context: WorldContext) -> Self {
        Self::new_with_config(name, context, true)
    }
    pub fn new_with_config(name: &'static str, context: WorldContext, resources: bool) -> Self {
        Self::new_with_config_internal(name, context, resources)
    }
    fn new_with_config_internal(
        name: &'static str,
        context: WorldContext,
        resources: bool,
    ) -> Self {
        let mut world = Self {
            name,
            context,
            archetypes: Vec::new(),
            locs: HashMap::with_hasher(EntityIdHashBuilder),
            loc_changed: FramedEvents::new(),
            version: CloneableAtomicU64::new(0),
            shape_change_events: None,
            ignore_query_inits: false,
            query_ticker: CloneableAtomicU64::new(0),
        };
        if resources {
            world.spawn_with_id(EntityId::resources(), Entity::new());
        }
        world
    }
    /// Clones all entities specified in the source world and returns a new world with them
    pub fn from_entities(
        world: &World,
        entities: impl IntoIterator<Item = EntityId>,
        serializable_only: bool,
    ) -> Self {
        let mut res = World::new_with_config("from_entities", world.context(), false);
        for id in entities {
            let mut entity = world.clone_entity(id).unwrap();
            if serializable_only {
                entity = entity.serializable();
            }
            entity.spawn(&mut res);
        }
        res
    }

    #[cfg(not(target_os = "unknown"))]
    pub async fn from_file(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        use anyhow::Context;
        let content = ambient_sys::fs::read(path.as_ref())
            .await
            .with_context(|| format!("Failed to read ECS slice from {:?}", path.as_ref()))?;
        Self::from_slice(&content)
    }

    pub fn from_slice(content: &[u8]) -> anyhow::Result<Self> {
        let DeserWorldWithWarnings { world, warnings } = serde_json::from_slice(content)?;
        warnings.log_warnings();
        Ok(world)
    }

    pub fn spawn(&mut self, entity_data: Entity) -> EntityId {
        self.batch_spawn(entity_data, 1).pop().unwrap()
    }

    pub fn batch_spawn(&mut self, entity_data: Entity, count: usize) -> Vec<EntityId> {
        let ids = (0..count).map(|_| EntityId::new()).collect_vec();
        for id in &ids {
            self.locs.insert(*id, EntityLocation::empty());
        }
        self.batch_spawn_with_ids(entity_data, ids.clone());
        ids
    }

    /// Returns false if the id already exists
    pub fn spawn_with_id(&mut self, entity_id: EntityId, entity_data: Entity) -> bool {
        if let std::collections::hash_map::Entry::Vacant(e) = self.locs.entry(entity_id) {
            e.insert(EntityLocation::empty());
            let version = self.inc_version();
            self.batch_spawn_with_ids_internal(
                EntityMoveData::from_entity_data(entity_data, version),
                vec![entity_id],
            );
            true
        } else {
            false
        }
    }
    pub fn batch_spawn_with_ids(&mut self, entity_data: Entity, ids: Vec<EntityId>) {
        if let Some(events) = &mut self.shape_change_events {
            events.add_events(
                ids.iter()
                    .map(|id| WorldChange::Spawn(*id, entity_data.clone())),
            );
        }
        let version = self.inc_version();
        self.batch_spawn_with_ids_internal(
            EntityMoveData::from_entity_data(entity_data, version),
            ids.clone(),
        );
    }
    fn batch_spawn_with_ids_internal(&mut self, entity_data: EntityMoveData, ids: Vec<EntityId>) {
        let arch_id = self
            .archetypes
            .iter()
            .position(|x| x.active_components == entity_data.active_components);
        let arch_id = if let Some(arch_id) = arch_id {
            arch_id
        } else {
            let arch_id = self.archetypes.len();
            self.archetypes
                .push(Archetype::new(arch_id, entity_data.components()));
            arch_id
        };
        let arch = &mut self.archetypes[arch_id];
        for (i, id) in ids.iter().enumerate() {
            let loc = self.locs.get_mut(id).expect("No such entity id");
            loc.archetype = arch.id;
            loc.index = arch.next_index() + i;
        }
        arch.movein(ids, entity_data);
    }
    pub fn despawn(&mut self, entity_id: EntityId) -> Option<Entity> {
        if let Some(loc) = self.locs.remove(&entity_id) {
            let version = self.inc_version();
            if let Some(events) = &mut self.shape_change_events {
                events.add_event(WorldChange::Despawn(entity_id));
            }
            let arch = self
                .archetypes
                .get_mut(loc.archetype)
                .expect("No such archetype");
            let last_entity_in_arch = *arch.entity_indices_to_ids.last().unwrap();
            if last_entity_in_arch != entity_id {
                self.locs.get_mut(&last_entity_in_arch).unwrap().index = loc.index;
                self.loc_changed.add_event(last_entity_in_arch);
            }
            Some(arch.moveout(loc.index, entity_id, version).into())
        } else {
            None
        }
    }
    pub fn despawn_all(&mut self) {
        let entity_ids: Vec<EntityId> = query_mut((), ())
            .iter(self, None)
            .map(|(id, _, _)| id)
            .collect();
        for id in entity_ids {
            self.despawn(id);
        }
    }
    #[profiling::function]
    pub fn next_frame(&mut self) {
        for arch in &mut self.archetypes {
            arch.next_frame();
        }
        if let Some(events) = &mut self.shape_change_events {
            events.next_frame();
        }
        self.ignore_query_inits = false;
    }

    pub fn set<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: crate::component::Component<T>,
        value: T,
    ) -> Result<T, ECSError> {
        let p = self.get_mut(entity_id, component)?;
        Ok(std::mem::replace(p, value))
    }

    pub fn set_entry(
        &mut self,
        entity_id: EntityId,
        entry: ComponentEntry,
    ) -> Result<ComponentEntry, ECSError> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let version = self.inc_version();
            let arch = self
                .archetypes
                .get_mut(loc.archetype)
                .expect("Archetype doesn't exist");
            arch.replace_with_entry(entity_id, loc.index, entry, version)
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }

    pub fn set_components(&mut self, entity_id: EntityId, data: Entity) -> Result<(), ECSError> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let version = self.inc_version();
            let arch = self
                .archetypes
                .get_mut(loc.archetype)
                .expect("Archetype doesn't exist");
            for entry in data {
                arch.replace_with_entry(entity_id, loc.index, entry, version)?;
            }
            Ok(())
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }

    /// Sets the value iff it is different to the current
    pub fn set_if_changed<T: ComponentValue + PartialEq>(
        &mut self,
        entity_id: EntityId,
        component: Component<T>,
        value: T,
    ) -> Result<(), ECSError> {
        let old = self.get_ref(entity_id, component)?;
        if old != &value {
            self.set(entity_id, component, value)?;
        }
        Ok(())
    }
    pub fn get_mut<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: Component<T>,
    ) -> Result<&mut T, ECSError> {
        self.get_mut_unsafe(entity_id, component)
    }
    pub fn get_mut_unsafe<T: ComponentValue>(
        &self,
        entity_id: EntityId,
        component: Component<T>,
    ) -> Result<&mut T, ECSError> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let version = self.inc_version();
            let arch = self
                .archetypes
                .get(loc.archetype)
                .expect("Archetype doesn't exist");
            match arch.get_component_mut(loc.index, entity_id, component, version) {
                Some(d) => Ok(d),
                None => Err(ECSError::EntityDoesntHaveComponent {
                    component_index: component.desc().index() as _,
                    name: component.path(),
                }),
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn get<T: Copy + ComponentValue>(
        &self,
        entity_id: EntityId,
        component: Component<T>,
    ) -> Result<T, ECSError> {
        self.get_ref(entity_id, component).map(|x| *x)
    }
    pub fn get_cloned<T: Clone + ComponentValue>(
        &self,
        entity_id: EntityId,
        component: Component<T>,
    ) -> Result<T, ECSError> {
        self.get_ref(entity_id, component).map(|x| x.clone())
    }
    pub fn get_ref<T: ComponentValue>(
        &self,
        entity_id: EntityId,
        component: Component<T>,
    ) -> Result<&T, ECSError> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let arch = self
                .archetypes
                .get(loc.archetype)
                .expect("Archetype doesn't exist");
            match arch.get_component(loc.index, component) {
                Some(d) => Ok(d),
                None => Err(ECSError::EntityDoesntHaveComponent {
                    component_index: component.desc().index() as usize,
                    name: component.path(),
                }),
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn get_entry(
        &self,
        entity_id: EntityId,
        component: ComponentDesc,
    ) -> Result<ComponentEntry, ECSError> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let arch = self
                .archetypes
                .get(loc.archetype)
                .expect("Archetype doesn't exist");
            match arch.get_component_buffer_untyped(component) {
                Some(d) => Ok(d.clone_value_boxed(loc.index)),
                None => Err(ECSError::EntityDoesntHaveComponent {
                    component_index: component.index() as usize,
                    name: component.path(),
                }),
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn has_component_index(
        &self,
        entity_id: EntityId,
        component_index: ComponentIndex,
    ) -> bool {
        self.archetype_for_entity(entity_id)
            .map(|arch| {
                arch.active_components
                    .contains_index(component_index as usize)
            })
            .unwrap_or(false)
    }
    #[inline]
    pub fn has_component_ref(
        &self,
        entity_id: EntityId,
        component: impl Into<ComponentDesc>,
    ) -> bool {
        self.has_component_index(entity_id, component.into().index() as _)
    }
    #[inline]
    pub fn has_component(&self, entity_id: EntityId, component: impl Into<ComponentDesc>) -> bool {
        self.has_component_ref(entity_id, component.into())
    }
    pub fn has_components(&self, entity_id: EntityId, components: &ComponentSet) -> bool {
        self.archetype_for_entity(entity_id)
            .map(|arch| arch.active_components.is_superset(components))
            .unwrap_or(false)
    }
    pub fn get_components(&self, entity_id: EntityId) -> Result<Vec<ComponentDesc>, ECSError> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let arch = self
                .archetypes
                .get(loc.archetype)
                .expect("Archetype doesn't exist");
            Ok(arch.components.iter().map(|x| x.component).collect_vec())
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }

    pub fn clone_entity(&self, entity_id: EntityId) -> Result<Entity, ECSError> {
        self.get_components(entity_id).map(|components| {
            let mut ed = Entity::new();
            for comp in components {
                ed.set_entry(self.get_entry(entity_id, comp).unwrap());
            }
            ed
        })
    }

    pub fn entities(&self) -> Vec<(EntityId, Entity)> {
        query(())
            .iter(self, None)
            .map(|(id, _)| (id, self.clone_entity(id).unwrap()))
            .collect()
    }
    pub fn exists(&self, entity_id: EntityId) -> bool {
        self.locs.contains_key(&entity_id)
    }

    fn map_entity(
        &mut self,
        entity_id: EntityId,
        map: impl FnOnce(MapEntity) -> MapEntity,
    ) -> Result<(), ECSError> {
        if let Some(loc) = self.locs.get(&entity_id).cloned() {
            let version = self.inc_version();
            let prev_comps = self
                .archetypes
                .get_mut(loc.archetype)
                .expect("No such archetype")
                .active_components
                .clone();

            let mapping = map(MapEntity {
                sets: HashMap::new(),
                removes: HashSet::new(),
                active_components: prev_comps.clone(),
            });

            if mapping.active_components == prev_comps {
                assert_eq!(mapping.removes.len(), 0);
                let arch = self
                    .archetypes
                    .get_mut(loc.archetype)
                    .expect("No such archetype");
                for (_, value) in mapping.sets.into_iter() {
                    arch.set_component_raw(loc.index, entity_id, value, version);
                }
            } else {
                let arch = self
                    .archetypes
                    .get_mut(loc.archetype)
                    .expect("No such archetype");
                let last_entity_in_arch = *arch.entity_indices_to_ids.last().unwrap();
                if entity_id != last_entity_in_arch {
                    self.locs.get_mut(&last_entity_in_arch).unwrap().index = loc.index;
                }
                self.loc_changed.add_event(last_entity_in_arch);
                self.loc_changed.add_event(entity_id);
                let mut data = arch.moveout(loc.index, entity_id, version);
                mapping.write_to_entity_data(&mut data, version);
                self.batch_spawn_with_ids_internal(data, vec![entity_id]);
            }
            Ok(())
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }

    pub fn add_components(&mut self, entity_id: EntityId, data: Entity) -> Result<(), ECSError> {
        // Safety check against adding a resource to an entity
        if entity_id != self.resource_entity() {
            if let Some(component) = data.iter().find(|c| c.has_attribute::<Resource>()) {
                return Err(ECSError::AddedResourceToEntity {
                    component_path: component.path(),
                    entity_id,
                });
            }
        }

        if let Some(events) = &mut self.shape_change_events {
            events.add_event(WorldChange::AddComponents(entity_id, data.clone()));
        }
        self.map_entity(entity_id, |ed| ed.append(data))
    }
    /// will also replace the existing component of the same type if it exists
    pub fn add_component<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: Component<T>,
        value: T,
    ) -> Result<(), ECSError> {
        self.add_components(entity_id, Entity::new().with(component, value))
    }

    /// Adds the component to the entity if it does not already have that component. Otherwise, does nothing.
    pub fn add_component_if_required<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: Component<T>,
        value: T,
    ) -> Result<(), ECSError> {
        if self.has_component(entity_id, component) {
            return Ok(()); // Already has the component
        }

        self.add_component(entity_id, component, value)
    }

    pub fn add_resource<T: ComponentValue>(&mut self, component: Component<T>, value: T) {
        self.add_component(self.resource_entity(), component, value)
            .unwrap()
    }

    /// Does nothing if the component does not exist
    pub fn remove_component(
        &mut self,
        entity_id: EntityId,
        component: impl Into<ComponentDesc>,
    ) -> Result<(), ECSError> {
        self.remove_components(entity_id, vec![component.into()])
    }

    pub fn remove_components(
        &mut self,
        entity_id: EntityId,
        components: Vec<ComponentDesc>,
    ) -> Result<(), ECSError> {
        if let Some(events) = &mut self.shape_change_events {
            events.add_event(WorldChange::RemoveComponents(entity_id, components.clone()));
        }
        self.map_entity(entity_id, |entity| entity.remove_components(components))
    }
    pub fn resource_entity(&self) -> EntityId {
        EntityId::resources()
    }

    pub fn resource_opt<T: ComponentValue>(&self, component: Component<T>) -> Option<&T> {
        Self::warn_on_non_resource_component(component);
        self.get_ref(self.resource_entity(), component).ok()
    }
    pub fn resource<T: ComponentValue>(&self, component: Component<T>) -> &T {
        match self.resource_opt(component) {
            Some(val) => val,
            None => panic!("Resource {} does not exist", component.path()),
        }
    }
    pub fn resource_mut_opt<T: ComponentValue>(
        &mut self,
        component: Component<T>,
    ) -> Option<&mut T> {
        Self::warn_on_non_resource_component(component);
        self.get_mut(self.resource_entity(), component).ok()
    }
    pub fn resource_mut<T: ComponentValue>(&mut self, component: Component<T>) -> &mut T {
        match self.resource_mut_opt(component) {
            Some(val) => val,
            None => panic!("Resource {} does not exist", component.path()),
        }
    }
    fn warn_on_non_resource_component<T: ComponentValue>(component: Component<T>) {
        if !component.has_attribute::<Resource>() && !component.has_attribute::<MaybeResource>() {
            tracing::warn!("Attempt to access non-resource component as a resource: {component:?}");
        }
    }

    pub fn archetypes(&self) -> &Vec<Archetype> {
        &self.archetypes
    }
    pub fn entity_loc(&self, id: EntityId) -> Option<&EntityLocation> {
        self.locs.get(&id)
    }

    pub fn id_from_lod(&self, archetype: usize, index: usize) -> EntityId {
        self.archetypes[archetype].entity_indices_to_ids[index]
    }

    /// Returns the content version of this component, which only changes when the component is written to (not when the entity changes archetype)
    pub fn get_component_content_version(
        &self,
        entity_id: EntityId,
        index: ComponentIndex,
    ) -> Result<u64, ECSError> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let arch = self
                .archetypes
                .get(loc.archetype)
                .expect("Archetype doesn't exist");
            match arch.get_component_content_version(*loc, index) {
                Some(d) => Ok(d),
                None => Err(ECSError::EntityDoesntHaveComponent {
                    component_index: index as _,
                    name: "".to_string(),
                }),
            }
        } else {
            Err(ECSError::NoSuchEntity { entity_id })
        }
    }
    pub fn loc_changed(&self) -> &FramedEvents<EntityId> {
        &self.loc_changed
    }
    pub fn init_shape_change_tracking(&mut self) {
        self.shape_change_events = Some(FramedEvents::new());
    }
    pub fn reset_events(&mut self) {
        self.loc_changed = FramedEvents::new();
        if let Some(shape_change_events) = &mut self.shape_change_events {
            *shape_change_events = FramedEvents::new();
        }
        for arch in self.archetypes.iter_mut() {
            arch.reset_events();
        }
        self.ignore_query_inits = true;
    }
    /// Spawn all entities of this world into the destination world
    pub fn spawn_into_world(&self, world: &mut World, components: Option<Entity>) -> Vec<EntityId> {
        let mut old_to_new_ids = HashMap::new();
        for (old_id, mut entity) in self.entities().into_iter() {
            if old_id != self.resource_entity() {
                if let Some(components) = components.as_ref() {
                    entity.merge(components.clone());
                }
                let new_id = entity.spawn(world);
                old_to_new_ids.insert(old_id, new_id);
            }
        }

        let migraters = COMPONENT_ENTITY_ID_MIGRATERS.lock();
        for migrater in migraters.iter() {
            for id in old_to_new_ids.values() {
                migrater(world, *id, &old_to_new_ids);
            }
        }
        old_to_new_ids.into_values().collect()
    }
    fn version(&self) -> u64 {
        self.version.0.load(Ordering::Relaxed)
    }
    fn inc_version(&self) -> u64 {
        self.version.0.fetch_add(1, Ordering::Relaxed) + 1
    }
    /// Number of entities in the world, including the resource entity
    pub fn len(&self) -> usize {
        self.archetypes.iter().fold(0, |p, x| p + x.entity_count())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn debug_archetypes(&self) -> DebugWorldArchetypes {
        DebugWorldArchetypes { world: self }
    }

    pub fn dump(&self, f: &mut dyn std::io::Write) {
        for arch in &self.archetypes {
            if arch.entity_count() > 0 {
                arch.dump(f);
            }
        }
    }
    #[cfg(not(target_os = "unknown"))]
    pub fn dump_to_tmp_file(&self) {
        std::fs::create_dir_all("tmp").ok();
        let mut f = std::fs::File::create("tmp/ecs.txt").expect("Unable to create file");
        self.dump(&mut f);
        tracing::info!("Wrote ecs to tmp/ecs.txt");
    }

    pub fn dump_entity_to_string(&self, id: EntityId) -> String {
        let mut s = Vec::new();
        self.dump_entity(id, 0, &mut s);
        String::from_utf8_lossy(&s).into_owned()
    }

    pub fn dump_entity(&self, entity_id: EntityId, indent: usize, f: &mut dyn std::io::Write) {
        if let Some(loc) = self.locs.get(&entity_id) {
            let arch = self
                .archetypes
                .get(loc.archetype)
                .expect("No such archetype");

            arch.dump_entity(loc.index, indent, f);
        } else {
            let indent = format!("{:indent$}", "", indent = indent);
            writeln!(f, "{indent}ERROR, NO SUCH ENTITY: {entity_id}").unwrap();
        }
    }

    pub fn dump_entity_to_yml(
        &self,
        entity_id: EntityId,
    ) -> Option<(String, yaml_rust::yaml::Hash)> {
        if let Some(loc) = self.locs.get(&entity_id) {
            let arch = self
                .archetypes
                .get(loc.archetype)
                .expect("No such archetype");
            Some(arch.dump_entity_to_yml(loc.index))
        } else {
            None
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = name;
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn add_entry(&mut self, id: EntityId, entry: ComponentEntry) -> Result<(), ECSError> {
        self.add_components(id, once(entry).collect())
    }

    pub fn context(&self) -> WorldContext {
        self.context
    }
}
impl World {
    fn archetype_for_entity(&self, id: EntityId) -> Option<&Archetype> {
        self.locs.get(&id).map(|loc| {
            self.archetypes
                .get(loc.archetype)
                .expect("Archetype doesn't exist")
        })
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World").finish()
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

// TODO(fred): Move this into the actual components instead
pub static COMPONENT_ENTITY_ID_MIGRATERS: Mutex<
    Vec<fn(&mut World, EntityId, &HashMap<EntityId, EntityId>)>,
> = Mutex::new(Vec::new());

#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq)]
pub enum ECSError {
    #[error("Entity doesn't have component: {component_index} {name}")]
    EntityDoesntHaveComponent {
        component_index: usize,
        name: String,
    },
    #[error("No such entity: {entity_id}")]
    NoSuchEntity { entity_id: EntityId },
    #[error(
        "Attempted to add resource component `{component_path}` to non-resource entity {entity_id}"
    )]
    AddedResourceToEntity {
        component_path: String,
        entity_id: EntityId,
    },
}

struct MapEntity {
    sets: HashMap<ComponentIndex, ComponentEntry>,
    removes: HashSet<ComponentIndex>,
    active_components: ComponentSet,
}
impl MapEntity {
    fn append(mut self, other: Entity) -> Self {
        for entry in other {
            self.active_components.insert(entry.desc());
            self.sets.insert(entry.desc().index() as _, entry);
        }
        self
    }

    fn remove_components(mut self, components: Vec<ComponentDesc>) -> Self {
        for desc in components {
            if self.active_components.contains(desc) {
                self.active_components.remove(desc);
                self.removes.insert(desc.index() as _);
            }
        }
        self
    }
    fn write_to_entity_data(self, data: &mut EntityMoveData, version: u64) {
        for value in self.sets.into_values() {
            data.set(value, version);
        }

        for comp in self.removes.into_iter() {
            data.remove(comp as _);
        }
    }
}

pub enum Command {
    Set(EntityId, ComponentEntry),
    AddComponent(EntityId, ComponentEntry),
    RemoveComponent(EntityId, ComponentDesc),
    Despawn(EntityId),
    Defer(Box<dyn Fn(&mut World) -> Result<(), ECSError> + Sync + Send + 'static>),
}

impl Command {
    fn apply(self, world: &mut World) -> Result<(), ECSError> {
        match self {
            Command::Set(id, entry) => {
                world.set_entry(id, entry)?;
                Ok(())
            }
            Command::AddComponent(entity, entry) => world.add_entry(entity, entry),
            Command::RemoveComponent(entity, component) => {
                world.remove_component(entity, component)
            }
            Command::Despawn(id) => {
                if world.despawn(id).is_none() {
                    Err(ECSError::NoSuchEntity { entity_id: id })
                } else {
                    Ok(())
                }
            }
            Command::Defer(func) => func(world),
        }
    }
}
pub struct Commands(Vec<Command>);
impl Commands {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn set<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: Component<T>,
        value: impl Into<T>,
    ) {
        self.0.push(Command::Set(
            entity_id,
            ComponentEntry::new(component, value.into()),
        ))
    }
    pub fn add_component<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: Component<T>,
        value: T,
    ) {
        self.0.push(Command::AddComponent(
            entity_id,
            ComponentEntry::new(component, value),
        ))
    }
    pub fn remove_component<T: ComponentValue>(
        &mut self,
        entity_id: EntityId,
        component: impl Into<ComponentDesc>,
    ) {
        self.0
            .push(Command::RemoveComponent(entity_id, component.into()));
    }
    pub fn despawn(&mut self, entity_id: EntityId) {
        self.0.push(Command::Despawn(entity_id));
    }

    /// Defers a function to execute upon the world.
    pub fn defer(
        &mut self,
        func: impl Fn(&mut World) -> Result<(), ECSError> + Sync + Send + 'static,
    ) {
        self.0.push(Command::Defer(Box::new(func)))
    }

    pub fn apply(&mut self, world: &mut World) -> Result<(), ECSError> {
        for command in self.0.drain(..) {
            command.apply(world)?;
        }
        Ok(())
    }
    /// Like apply, but doesn't stop on an error, instead just logs a warning
    pub fn soft_apply(&mut self, world: &mut World) {
        for command in self.0.drain(..) {
            if let Err(err) = command.apply(world) {
                tracing::warn!("soft_apply error: {:?}", err);
            }
        }
    }
    /// Like soft apply, but doesn't even issue a warning
    pub fn softer_apply(&mut self, world: &mut World) {
        for command in self.0.drain(..) {
            command.apply(world).ok();
        }
    }
}

pub(crate) struct CloneableAtomicU64(pub AtomicU64);
impl CloneableAtomicU64 {
    pub fn new(value: u64) -> Self {
        Self(AtomicU64::new(value))
    }
}
impl Clone for CloneableAtomicU64 {
    fn clone(&self) -> Self {
        Self(AtomicU64::new(self.0.load(Ordering::SeqCst)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSet(pub BitSet);
impl ComponentSet {
    pub fn new() -> Self {
        Self(BitSet::with_capacity(with_component_registry(|cr| {
            cr.component_count()
        })))
    }

    pub fn insert(&mut self, component: ComponentDesc) {
        self.insert_by_index(component.index() as _);
    }
    pub fn insert_by_index(&mut self, component_index: usize) {
        self.0.insert(component_index);
    }
    pub fn remove(&mut self, component: ComponentDesc) {
        self.remove_by_index(component.index() as _)
    }
    pub fn remove_by_index(&mut self, component_index: usize) {
        self.0.remove(component_index);
    }
    pub fn union_with(&mut self, rhs: &ComponentSet) {
        self.0.union_with(&rhs.0);
    }

    #[inline]
    pub fn contains(&self, desc: ComponentDesc) -> bool {
        self.contains_index(desc.index() as _)
    }
    pub fn contains_index(&self, component_index: usize) -> bool {
        self.0.contains(component_index)
    }
    pub fn is_superset(&self, other: &ComponentSet) -> bool {
        self.0.is_superset(&other.0)
    }
    pub fn is_disjoint(&self, other: &ComponentSet) -> bool {
        self.0.is_disjoint(&other.0)
    }
    pub fn intersection<'a>(&'a self, rhs: &'a ComponentSet) -> impl Iterator<Item = usize> + 'a {
        self.0.intersection(&rhs.0)
    }
}
#[derive(Serialize, Deserialize)]
struct ComponentSetSerialized(u64, Vec<u8>);
impl Serialize for ComponentSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ComponentSetSerialized(
            self.0.len() as u64,
            self.0.clone().into_bit_vec().to_bytes(),
        )
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for ComponentSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let css = ComponentSetSerialized::deserialize(deserializer)?;
        let mut bv = BitVec::from_bytes(&css.1);
        bv.truncate(css.0 as usize);

        Ok(ComponentSet(BitSet::from_bit_vec(bv)))
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum WorldEventSource {
    Runtime,
    Server,
    Client(String),
    Local(EntityId),
}

pub type WorldEvents = FramedEvents<(WorldEventSource, String, Vec<u8>)>;
pub type WorldEventReader = FramedEventsReader<(WorldEventSource, String, Vec<u8>)>;

#[derive(Debug)]
pub struct WorldEventsSystem;
impl System for WorldEventsSystem {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        world.resource_mut(world_events()).next_frame();
    }
}
