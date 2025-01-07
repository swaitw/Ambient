use std::collections::{hash_map::Entry, BTreeMap, HashMap};

use ambient_native_std::events::EventDispatcher;
use ambient_shared_types::ComponentIndex;
use once_cell::sync::Lazy;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::*;
use crate::ComponentVTable;

static COMPONENT_REGISTRY: Lazy<RwLock<ComponentRegistry>> =
    Lazy::new(|| RwLock::new(ComponentRegistry::default()));
static COMPONENT_ATTRIBUTES: RwLock<BTreeMap<ComponentIndex, AttributeStore>> =
    RwLock::new(BTreeMap::new());

pub(crate) fn get_external_attributes(index: ComponentIndex) -> AttributeStoreGuard {
    let guard = COMPONENT_ATTRIBUTES.read();

    RwLockReadGuard::map(guard, |val| {
        val.get(&index).expect("No external attributes")
    })
}

pub(crate) fn get_external_attributes_init(index: ComponentIndex) -> AttributeStoreGuardMut {
    let guard = COMPONENT_ATTRIBUTES.write();

    RwLockWriteGuard::map(guard, |val| val.entry(index).or_default())
}

pub fn with_component_registry<R>(f: impl FnOnce(&ComponentRegistry) -> R + Sync + Send) -> R {
    let lock = COMPONENT_REGISTRY.read();
    f(&lock)
}

pub(crate) struct RegistryComponent {
    pub(crate) desc: ComponentDesc,
    pub(crate) primitive_component: Option<PrimitiveComponent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalComponentDesc {
    pub path: String,
    pub ty: PrimitiveComponentType,
    pub name: Option<String>,
    pub description: Option<String>,
    pub attributes: ExternalComponentAttributes,
}

impl From<&PrimitiveComponent> for ExternalComponentDesc {
    fn from(pc: &PrimitiveComponent) -> Self {
        ExternalComponentDesc {
            path: pc.desc.path(),
            ty: pc.ty,
            name: pc.desc.attribute::<Name>().map(|n| n.0.clone()),
            description: pc.desc.attribute::<Description>().map(|n| n.0.clone()),
            attributes: ExternalComponentAttributes::from_existing_component(pc.desc),
        }
    }
}

macro_rules! define_external_component_attribute {
    (
        standard: {$($field_name:ident: $type_name:ty),*},
        special: {$($special_field_name:ident: $special_type_name:ty),*}
    ) => {
        #[derive(Serialize, Deserialize, Clone, Debug, Default)]
        pub struct ExternalComponentAttributes {
            $(pub $field_name: bool,)*
            $(pub $special_field_name: bool,)*
        }
        impl ExternalComponentAttributes {
            pub fn from_existing_component(desc: ComponentDesc) -> Self {
                Self {
                    $($field_name: desc.has_attribute::<$type_name>(),)*
                    $($special_field_name: desc.has_attribute::<$special_type_name>(),)*
                }
            }

            pub fn iter(&self) -> impl Iterator<Item = &'static str> {
                [
                    $(self.$field_name.then_some(stringify!($type_name)),)*
                    $(self.$special_field_name.then_some(stringify!($special_type_name)),)*
                ]
                .into_iter()
                .flatten()
            }

            pub fn construct_for_store<T: Debug + Serialize + for<'de> Deserialize<'de> + Clone + ComponentValue>(&self, store: &mut AttributeStore) {
                $(
                    if self.$field_name {
                        <$type_name as AttributeConstructor<T, _>>::construct(store, ());
                    }
                )*

                if self.enum_ {
                    <Enum as AttributeConstructor<u32, _>>::construct(store, ());
                }
            }
        }
        impl<'a> FromIterator<&'a str> for ExternalComponentAttributes {
            fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
                let mut flags = Self::default();
                for flag_str in iter {
                    match flag_str {
                        $(stringify!($type_name) => { flags.$field_name = true; },)*
                        $(stringify!($special_type_name) => { flags.$special_field_name = true; },)*
                        _ => panic!("Unexpected attribute flag: {flag_str}"),
                    }
                }
                flags
            }
        }
    }
}
define_external_component_attribute! {
    standard: {
        debuggable: Debuggable,
        networked: Networked,
        resource: Resource,
        store: Store,
        maybe_resource: MaybeResource
    },
    special: {
        enum_: Enum
    }
}

#[derive(Default)]
pub struct ComponentRegistry {
    pub(crate) components: Vec<RegistryComponent>,
    pub component_paths: HashMap<String, ComponentIndex>,
    pub next_index: ComponentIndex,

    /// Handlers are called with a write-lock on ComponentRegistry, which will result in deadlock if your operation
    /// requires a read-lock on ComponentRegistry. Consider deferring your operation to a later time.
    pub on_external_components_change: EventDispatcher<dyn Fn() + Sync + Send>,
}
impl ComponentRegistry {
    pub fn get() -> RwLockReadGuard<'static, Self> {
        COMPONENT_REGISTRY.read()
    }
    pub fn get_mut() -> RwLockWriteGuard<'static, Self> {
        COMPONENT_REGISTRY.write()
    }

    pub fn add_external(&mut self, components: Vec<ExternalComponentDesc>) {
        for desc in components {
            desc.ty.register(
                self,
                &desc.path,
                desc.name.as_deref(),
                desc.description.as_deref(),
                desc.attributes,
            );
        }

        for handler in self.on_external_components_change.iter() {
            handler();
        }
    }

    fn register(
        &mut self,
        path: String,
        vtable: &'static ComponentVTable<()>,
        attributes: Option<AttributeStore>,
    ) -> ComponentDesc {
        let index = match self.component_paths.entry(path.to_owned()) {
            Entry::Occupied(slot) => *slot.get(),
            Entry::Vacant(slot) => {
                let index = self
                    .components
                    .len()
                    .try_into()
                    .expect("Maximum component count exceeded");
                slot.insert(index);

                let desc = ComponentDesc::new(index, vtable);

                // If a PrimitiveComponentType can be created from this component's type, create a PrimitiveComponent for it
                let primitive_component = TYPE_ID_TO_PRIMITIVE_TYPE
                    .get(&(vtable.get_type_id)())
                    .copied()
                    .map(|ty| PrimitiveComponent { ty, desc });

                self.components.push(RegistryComponent {
                    desc,
                    primitive_component,
                });

                index
            }
        };

        let slot = &mut self.components[index as usize];

        let mut dst = (vtable.attributes_init)(slot.desc);
        dst.set(ComponentPath(path));

        if let Some(src) = attributes {
            dst.append(&src);
        }

        slot.desc
    }

    pub(crate) fn register_external(
        &mut self,
        path: String,
        vtable: &'static ComponentVTable<()>,
        mut attributes: AttributeStore,
    ) -> ComponentDesc {
        assert_eq!(
            None, vtable.path,
            "Static name does not match provided name"
        );

        tracing::debug!("Registering external component: {path}");

        attributes.set(External);
        self.register(path, vtable, Some(attributes))
    }

    pub fn register_static(
        &mut self,
        path: &str,
        vtable: &'static ComponentVTable<()>,
    ) -> ComponentDesc {
        tracing::debug!("Registering static component: {path}");
        self.register(path.into(), vtable, Default::default())
    }

    pub fn path_to_index(&self, path: &str) -> Option<ComponentIndex> {
        self.component_paths.get(path).copied()
    }

    pub fn get_by_path(&self, path: &str) -> Option<ComponentDesc> {
        let index = *self.component_paths.get(path)?;
        Some(self.components[index as usize].desc)
    }

    pub fn get_by_index(&self, index: ComponentIndex) -> Option<ComponentDesc> {
        self.components.get(index as usize).map(|b| b.desc)
    }

    pub fn get_primitive_component(&self, idx: ComponentIndex) -> Option<PrimitiveComponent> {
        self.components
            .get(idx as usize)
            .unwrap()
            .primitive_component
            .clone()
    }

    /// Returns an iterator over all primitive components and their descs.
    pub fn all_primitive(&self) -> impl Iterator<Item = &PrimitiveComponent> + '_ {
        self.components
            .iter()
            .filter_map(|v| v.primitive_component.as_ref())
    }

    /// Returns an iterator over all primitive components that were externally defined and their descs.
    pub fn all_external(
        &self,
    ) -> impl Iterator<Item = (ExternalComponentDesc, ComponentDesc)> + '_ {
        self.all_primitive()
            .filter(|pc| pc.desc.has_attribute::<External>())
            .map(|pc| (pc.into(), pc.desc))
    }

    pub fn all(&self) -> impl Iterator<Item = ComponentDesc> + '_ {
        self.components.iter().map(|v| v.desc)
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}
