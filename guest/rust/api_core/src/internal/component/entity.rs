use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::internal::{
    conversion::{FromBindgen, IntoBindgen},
    wit::{self},
};

use super::{
    Component, ComponentIndex, ComponentValue, SupportedValue, SupportedValueRef, UntypedComponent,
};

/// An [Entity] is a collection of components and associated values.
///
/// Use the [spawn](Entity::spawn) method to insert the [Entity] into the world.
#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Entity(pub(crate) HashMap<ComponentIndex, ComponentValue>);
impl Entity {
    /// Creates a new `Entity`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of component (values) in the entity.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the entity has no components (values).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this has `component`.
    pub fn has<T: SupportedValue>(&self, component: Component<T>) -> bool {
        self.0.contains_key(&component.index())
    }

    /// Returns true if this has all of `components`.
    pub fn has_components(&self, components: &[&dyn UntypedComponent]) -> bool {
        components
            .iter()
            .all(|component| self.0.contains_key(&component.index()))
    }

    /// Gets the data for `component` in this, if it exists.
    pub fn get<T: SupportedValue>(&self, component: Component<T>) -> Option<T> {
        T::from_value(self.0.get(&component.index())?.clone())
    }

    /// Gets a reference to the data for `component` in this, if it exists.
    pub fn get_ref<T: SupportedValueRef>(&self, component: Component<T>) -> Option<&T> {
        T::from_value_ref(self.0.get(&component.index())?)
    }

    /// Adds `component` to this with `value`. It will replace an existing component if present.
    pub fn set<T: SupportedValue>(&mut self, component: Component<T>, value: T) {
        self.0.insert(component.index(), value.into_value());
    }

    /// Sets the `component` in this to the default value for `T`.
    pub fn set_default<T: SupportedValue + Default>(&mut self, component: Component<T>) {
        self.set(component, T::default())
    }

    /// Adds `component` to this with `value`, and returns `self` to allow for easy chaining.
    pub fn with<T: SupportedValue>(mut self, component: Component<T>, value: T) -> Self {
        self.set(component, value);
        self
    }

    /// Merges in the `other` Entity and returns this; any fields that were present in both will be replaced by `other`'s.
    pub fn with_merge(mut self, other: impl Into<Entity>) -> Self {
        self.merge(other.into());
        self
    }

    /// Removes `component` to this with `value`, and returns `self` to allow for easy chaining.
    pub fn without<T: SupportedValue>(mut self, component: Component<T>) -> Self {
        self.0.remove(&component.index());
        self
    }

    /// Removes the specified component from this, and returns the value if it was present.
    pub fn remove<T: SupportedValue>(&mut self, component: Component<T>) -> Option<T> {
        T::from_value(self.0.remove(&component.index())?)
    }

    /// Merges in the `other` Entity; any fields that were present in both will be replaced by `other`'s.
    pub fn merge(&mut self, other: Entity) {
        self.0.extend(other.0);
    }

    /// Spawns an entity with these components.
    ///
    /// Returns `spawned_entity_uid`.
    pub fn spawn(&self) -> crate::prelude::EntityId {
        crate::entity::spawn(self)
    }
}
impl FromBindgen for wit::component::Entity {
    type Item = Entity;

    fn from_bindgen(self) -> Self::Item {
        Entity(
            self.into_iter()
                .map(|(k, v)| (k, v.from_bindgen()))
                .collect(),
        )
    }
}
impl IntoBindgen for Entity {
    type Item = wit::component::Entity;

    fn into_bindgen(self) -> Self::Item {
        self.0
            .into_iter()
            .map(|(k, v)| (k, v.into_bindgen()))
            .collect()
    }
}
impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.0.iter().map(|(k, v)| {
                (
                    wit::component::get_id(*k).unwrap_or_else(|| format!("unknown component {k}")),
                    v,
                )
            }))
            .finish()
    }
}
