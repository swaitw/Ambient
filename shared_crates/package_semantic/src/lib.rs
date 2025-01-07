use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::Context as AnyhowContext;
use async_recursion::async_recursion;

use ambient_package::{
    BuildMetadata, ComponentType, Identifier, ItemPath, ItemPathBuf, Manifest, PackageId,
    PascalCaseIdentifier, SnakeCaseIdentifier,
};
use ambient_shared_types::{primitive_component_definitions, urls::API_URL};

mod scope;
use package::{GetError, ParentJoinError, RetrievableDeployment};
pub use scope::Scope;

mod package;
pub use package::{Dependency, LocalOrRemote, Package, PackageLocator, RetrievableFile};

mod item;
pub use item::{
    Item, ItemData, ItemId, ItemMap, ItemSource, ItemType, ItemVariant, ResolvableItemId,
};

mod component;
pub use component::Component;

mod concept;
pub use concept::{Concept, ConceptValue};

mod attribute;
pub use attribute::Attribute;

mod primitive_type;
pub use primitive_type::PrimitiveType;

mod type_;
use thiserror::Error;
pub use type_::{Enum, Type, TypeInner};

mod message;
pub use message::Message;

mod value;
use url::Url;
pub use value::{ResolvableValue, ScalarValue, Value};

mod printer;
pub use printer::Printer;

mod util;

pub type Schema = HashMap<&'static str, &'static str>;
pub fn schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();
    SCHEMA.get_or_init(|| HashMap::from_iter(ambient_schema::FILES.iter().copied()))
}

#[derive(Error, Debug)]
pub enum PackageAddError {
    #[error(
        "The manifest in {include_source} does not have an ID, and no ID override was provided"
    )]
    MissingId { include_source: RetrievableFile },
    #[error("Failed to parse manifest from `{manifest_path}`")]
    ManifestParseError {
        manifest_path: RetrievableFile,
        source: ambient_package::ManifestParseError,
    },
    #[error("Failed to add dependency `{dependency_name}` for {locator}: {source}")]
    FailedToAddDependency {
        dependency_name: SnakeCaseIdentifier,
        locator: PackageLocator,
        source: Box<PackageAddError>,
    },
    #[error(
        "Include `{include_name}` = {include_path:?} for `{include_source}` must have an extension"
    )]
    IncludeMissingExtension {
        include_name: SnakeCaseIdentifier,
        include_path: PathBuf,
        include_source: RetrievableFile,
    },
    #[error("Failed to parse included manifest from {include_source}")]
    IncludeParseError {
        include_source: RetrievableFile,
        source: ambient_package::ManifestParseError,
    },
    #[error("The package `{package_id}` does not have a version `{version}`")]
    InvalidPackageVersion {
        package_id: PackageId,
        version: String,
    },
    #[error("{0}")]
    PackageConflictError(Box<PackageConflictError>),
    #[error("{0}")]
    GetError(#[from] GetError),
    #[error("{0}")]
    ParentJoinError(#[from] ParentJoinError),
    #[error("Dependency `{dependency_name}` for {locator} has no supported sources specified (are you trying to deploy a package with a local dependency?)")]
    NoSupportedSources {
        locator: PackageLocator,
        dependency_name: SnakeCaseIdentifier,
    },
    #[error("{0}")]
    BuildMetadataError(ambient_package::BuildMetadataError),
    #[error("{0}")]
    IdentifierCaseError(ambient_package::IdentifierCaseOwnedError),
}

impl From<PackageConflictError> for Box<PackageAddError> {
    fn from(val: PackageConflictError) -> Self {
        Box::new(PackageAddError::PackageConflictError(Box::new(val)))
    }
}
impl From<GetError> for Box<PackageAddError> {
    fn from(val: GetError) -> Self {
        Box::new(PackageAddError::GetError(val))
    }
}
impl From<ParentJoinError> for Box<PackageAddError> {
    fn from(val: ParentJoinError) -> Self {
        Box::new(PackageAddError::ParentJoinError(val))
    }
}
impl From<ambient_package::BuildMetadataError> for Box<PackageAddError> {
    fn from(val: ambient_package::BuildMetadataError) -> Self {
        Box::new(PackageAddError::BuildMetadataError(val))
    }
}
impl From<ambient_package::IdentifierCaseOwnedError> for Box<PackageAddError> {
    fn from(val: ambient_package::IdentifierCaseOwnedError) -> Self {
        Box::new(PackageAddError::IdentifierCaseError(val))
    }
}

#[derive(Debug)]
pub struct PackageConflictError {
    existing_package: PackageLocator,
    existing_package_dependent: Option<PackageLocator>,
    new_package: PackageLocator,
    new_package_dependent: Option<PackageLocator>,
}
impl std::error::Error for PackageConflictError {}
impl fmt::Display for PackageConflictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PackageConflictError {
            existing_package,
            existing_package_dependent,
            new_package,
            new_package_dependent,
        } = self;

        fn imported_by(dependent_locator: Option<&PackageLocator>) -> String {
            match dependent_locator {
                Some(locator) => {
                    format!("\n      imported by {}", locator)
                }
                None => String::new(),
            }
        }

        write!(
            f,
            "Package conflict found:\n  - {existing_package}{}\n\n  - {new_package}{}\n\nThe system does not currently support multiple versions of the same package in the dependency tree.",
            imported_by(existing_package_dependent.as_ref()),
            imported_by(new_package_dependent.as_ref())
        )
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Semantic {
    pub items: ItemMap,
    pub root_scope_id: ItemId<Scope>,
    pub packages: HashMap<PackageLocator, ItemId<Package>>,
    /// Used to determine if there are any existing packages with this ID
    pub id_to_locator: HashMap<Identifier, PackageLocator>,
    pub ambient_package_id: ItemId<Package>,
    pub standard_definitions: StandardDefinitions,
    ignore_local_dependencies: bool,
}
impl Semantic {
    /// For debugging: `path` dependencies will be ignored when adding packages
    const ALWAYS_IGNORE_LOCAL_DEPENDENCIES: bool = false;

    pub async fn new(ignore_local_dependencies: bool) -> anyhow::Result<Self> {
        let mut items = ItemMap::default();
        let (root_scope_id, standard_definitions) = create_root_scope(&mut items)?;

        let ignore_local_dependencies =
            ignore_local_dependencies || Self::ALWAYS_IGNORE_LOCAL_DEPENDENCIES;

        let mut semantic = Self {
            items,
            root_scope_id,
            packages: HashMap::new(),
            id_to_locator: HashMap::new(),
            ambient_package_id: ItemId::empty_you_should_really_initialize_this(),
            standard_definitions,
            ignore_local_dependencies,
        };

        semantic.ambient_package_id = semantic
            .add_package(
                RetrievableFile::Ambient(PathBuf::from("ambient.toml")),
                None,
            )
            .await?;

        Ok(semantic)
    }

    #[cfg_attr(not(target_os = "unknown"), async_recursion)]
    #[cfg_attr(target_os = "unknown", async_recursion(?Send))]
    pub async fn add_package(
        &mut self,
        retrievable_manifest: RetrievableFile,
        // Used to indicate which package had this as a dependency first,
        // improving error diagnostics
        dependent_package_id: Option<ItemId<Package>>,
    ) -> Result<ItemId<Package>, Box<PackageAddError>> {
        let manifest = Manifest::parse(&retrievable_manifest.get().await?).map_err(|source| {
            Box::new(PackageAddError::ManifestParseError {
                manifest_path: retrievable_manifest.clone(),
                source,
            })
        })?;

        // If and only if this is the root Ambient core manifest, override its ID.
        // Otherwise, leave it alone.
        let id_override = match &retrievable_manifest {
            RetrievableFile::Ambient(path) if path == Path::new("ambient.toml") => Some(
                Identifier::from(SnakeCaseIdentifier::new("ambient_core").unwrap()),
            ),
            _ => None,
        };

        let locator = PackageLocator::from_manifest(
            &manifest,
            retrievable_manifest.clone(),
            id_override.clone(),
        )
        .ok_or_else(|| PackageAddError::MissingId {
            include_source: retrievable_manifest.clone(),
        })?;

        if let Some(id) = self.packages.get(&locator) {
            return Ok(*id);
        }

        if let Some(existing) = self.id_to_locator.get(&locator.id) {
            let existing_package = self.items.get(self.packages[existing]);

            let get_locator = |package_id: Option<ItemId<Package>>| {
                package_id.map(|p| self.items.get(p).locator.clone())
            };

            return Err(PackageConflictError {
                existing_package: existing.clone(),
                existing_package_dependent: get_locator(existing_package.dependent_package_id),
                new_package: locator,
                new_package_dependent: get_locator(dependent_package_id),
            }
            .into());
        }

        let build_metadata = retrievable_manifest
            .parent_join(Path::new(BuildMetadata::FILENAME))?
            .get()
            .await
            .ok()
            .map(|s| BuildMetadata::parse(&s))
            .transpose()?;

        let scope_id = self
            .add_scope_from_manifest_with_includes(
                None,
                &manifest,
                retrievable_manifest.clone(),
                id_override,
            )
            .await?;

        let manifest_dependencies = manifest.dependencies.clone();
        let package = Package {
            data: ItemData {
                parent_id: None,
                id: locator.id.clone(),
                source: match retrievable_manifest {
                    RetrievableFile::Ambient(_) => ItemSource::Ambient,
                    RetrievableFile::Path(_)
                    | RetrievableFile::Url(_)
                    | RetrievableFile::Deployment(_) => ItemSource::User,
                },
            },
            locator: locator.clone(),
            source: retrievable_manifest.clone(),
            manifest,
            build_metadata,
            dependencies: HashMap::new(),
            scope_id,
            dependent_package_id,
            resolved: false,
        };

        let id = self.items.add(package);

        // Add the dependencies after the fact so that we can use the package id
        let mut dependencies = HashMap::new();
        for (dependency_name, dependency) in manifest_dependencies {
            let Some(source) = package_dependency_to_retrievable_file(
                &retrievable_manifest,
                self.ignore_local_dependencies,
                &dependency,
            )
            .await?
            else {
                return Err(Box::new(PackageAddError::NoSupportedSources {
                    locator,
                    dependency_name,
                }));
            };

            let dependency_id = self.add_package(source, Some(id)).await.map_err(|source| {
                PackageAddError::FailedToAddDependency {
                    dependency_name: dependency_name.clone(),
                    locator: locator.clone(),
                    source,
                }
            })?;

            dependencies.insert(
                dependency_name.clone(),
                Dependency {
                    id: dependency_id,
                    enabled: dependency.enabled,
                },
            );
        }

        {
            let scope = self.items.get_mut(scope_id);

            // If this is not the Ambient package, import the Ambient package
            if !matches!(retrievable_manifest, RetrievableFile::Ambient(_)) {
                let id = SnakeCaseIdentifier::new("ambient_core").unwrap();
                scope.imports.insert(id, self.ambient_package_id);
            }

            for (name, dependency) in &dependencies {
                scope.imports.insert(name.clone(), dependency.id);
            }
        }

        self.items.get_mut(id).dependencies = dependencies;

        self.id_to_locator
            .insert(locator.id.clone(), locator.clone());
        self.packages.insert(locator, id);

        Ok(id)
    }

    pub fn resolve_all(&mut self) -> anyhow::Result<()> {
        let package_ids = self.packages.values().copied().collect::<Vec<_>>();
        for package_id in package_ids {
            self.resolve(package_id)?;
        }

        Ok(())
    }

    pub fn root_scope(&self) -> &Scope {
        self.items.get(self.root_scope_id)
    }

    pub fn get_scope_id_by_name(&self, name: &SnakeCaseIdentifier) -> Option<ItemId<Scope>> {
        self.root_scope().scopes.get(name).copied()
    }
}
impl Semantic {
    /// Resolve the item with the given id by cloning it, avoiding borrowing issues.
    pub(crate) fn resolve<T: Resolve>(&mut self, id: ItemId<T>) -> anyhow::Result<&mut T> {
        let item = self.items.get(id);
        if !item.already_resolved() {
            let item = item.clone();
            let new_item = item.resolve(self, id)?;
            self.items.insert(id, new_item);
        }
        Ok(self.items.get_mut(id))
    }

    /// Walks upwards from `start_scope` to find the first type located at `path`.
    fn get_contextual<Output: Item>(
        &self,
        start_scope: ItemId<Scope>,
        getter: impl Fn(ItemId<Scope>) -> Option<ItemId<Output>>,
    ) -> Option<ItemId<Output>> {
        let mut context_scope_id = Some(start_scope);
        while let Some(scope_id) = context_scope_id {
            if let Some(id) = getter(scope_id) {
                return Some(id);
            }
            context_scope_id = self.items.get(scope_id).data().parent_id;
        }
        if let Some(id) = getter(self.root_scope_id) {
            return Some(id);
        }
        None
    }

    /// Walks upwards from `start_scope` to find the first type located at `path`.
    pub(crate) fn get_contextual_type_id(
        &self,
        start_scope: ItemId<Scope>,
        component_type: &ComponentType,
    ) -> Option<ItemId<Type>> {
        self.get_contextual(start_scope, |scope_id| match component_type {
            ComponentType::Item(id) => get_type_id(&self.items, scope_id, id.as_path()),
            ComponentType::Contained {
                type_,
                element_type,
            } => get_type_id(&self.items, scope_id, element_type.as_path()).map(|id| match type_ {
                ambient_package::ContainerType::Vec => self.items.get_vec_id(id),
                ambient_package::ContainerType::Option => self.items.get_option_id(id),
            }),
        })
    }

    /// Walks upwards from `start_scope` to find the first attribute located at `path`.
    pub(crate) fn get_contextual_attribute_id<'a>(
        &self,
        start_scope: ItemId<Scope>,
        path: ItemPath<'a>,
    ) -> Result<ItemId<Attribute>, ContextGetError<'a>> {
        self.get_contextual(start_scope, |scope_id| {
            get_attribute_id(&self.items, scope_id, path)
        })
        .ok_or(ContextGetError::NotFound {
            path,
            type_: ItemType::Attribute,
        })
    }

    /// Walks upwards from `start_scope` to find the first concept located at `path`.
    pub(crate) fn get_contextual_concept_id<'a>(
        &self,
        start_scope: ItemId<Scope>,
        path: ItemPath<'a>,
    ) -> Result<ItemId<Concept>, ContextGetError<'a>> {
        self.get_contextual(start_scope, |scope_id| {
            get_concept_id(&self.items, scope_id, path)
        })
        .ok_or(ContextGetError::NotFound {
            path,
            type_: ItemType::Concept,
        })
    }

    /// Walks upwards from `start_scope` to find the first component located at `path`.
    pub(crate) fn get_contextual_component_id<'a>(
        &self,
        start_scope: ItemId<Scope>,
        path: ItemPath<'a>,
    ) -> Result<ItemId<Component>, ContextGetError<'a>> {
        self.get_contextual(start_scope, |scope_id| {
            get_component_id(&self.items, scope_id, path)
        })
        .ok_or(ContextGetError::NotFound {
            path,
            type_: ItemType::Component,
        })
    }
}
impl Semantic {
    #[cfg_attr(not(target_os = "unknown"), async_recursion)]
    #[cfg_attr(target_os = "unknown", async_recursion(?Send))]
    async fn add_scope_from_manifest_with_includes(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        manifest: &Manifest,
        source: RetrievableFile,
        id_override: Option<Identifier>,
    ) -> Result<ItemId<Scope>, Box<PackageAddError>> {
        let includes = manifest.includes.clone();
        let scope_id = self.add_scope_from_manifest_without_includes(
            parent_id,
            manifest,
            source.clone(),
            id_override,
        )?;

        let mut include_names: Vec<_> = includes.keys().collect();
        include_names.sort();

        for include_name in include_names {
            let include_path = &includes[include_name];

            if include_path.extension().is_none() {
                return Err(Box::new(PackageAddError::IncludeMissingExtension {
                    include_name: include_name.clone(),
                    include_path: include_path.clone(),
                    include_source: source.clone(),
                }));
            }

            let include_source = source.parent_join(include_path)?;
            let include_manifest =
                Manifest::parse(&include_source.get().await?).map_err(|err| {
                    PackageAddError::IncludeParseError {
                        include_source: source.clone(),
                        source: err,
                    }
                })?;
            let include_scope_id = self
                .add_scope_from_manifest_with_includes(
                    Some(scope_id),
                    &include_manifest,
                    include_source,
                    Some(include_name.clone().into()),
                )
                .await?;

            self.items
                .get_mut(scope_id)
                .scopes
                .insert(include_name.clone(), include_scope_id);
        }

        Ok(scope_id)
    }

    fn add_scope_from_manifest_without_includes(
        &mut self,
        parent_id: Option<ItemId<Scope>>,
        manifest: &Manifest,
        source: RetrievableFile,
        id_override: Option<Identifier>,
    ) -> Result<ItemId<Scope>, Box<PackageAddError>> {
        let item_source = match source {
            RetrievableFile::Ambient(_) => ItemSource::Ambient,
            _ => ItemSource::User,
        };
        let scope_id = self.items.add(Scope::new(ItemData {
            parent_id,
            id: manifest
                .package
                .id
                .clone()
                .map(Identifier::from)
                .or(id_override)
                .ok_or_else(|| PackageAddError::MissingId {
                    include_source: source.clone(),
                })?,
            source: item_source,
        }));

        let make_item_data = |parent_id: ItemId<Scope>, item_id: &Identifier| -> ItemData {
            ItemData {
                parent_id: Some(parent_id),
                id: item_id.clone(),
                source: item_source,
            }
        };

        let items = &mut self.items;
        for (path, component) in manifest.components.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let scope_id = items.get_or_create_scope_mut(scope_id, scope_path);
            let value = items.add(Component::from_package(
                make_item_data(scope_id, item),
                component,
            ));

            items
                .get_mut(scope_id)
                .components
                .insert(item.as_snake().map_err(|e| e.to_owned())?.clone(), value);
        }

        for (path, concept) in manifest.concepts.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let scope_id = items.get_or_create_scope_mut(scope_id, scope_path);
            let value = items.add(Concept::from_package(
                make_item_data(scope_id, item),
                concept,
            ));

            items
                .get_mut(scope_id)
                .concepts
                .insert(item.as_pascal().map_err(|e| e.to_owned())?.clone(), value);
        }

        for (path, message) in manifest.messages.iter() {
            let path = path.as_path();
            let (scope_path, item) = path.scope_and_item();

            let scope_id = items.get_or_create_scope_mut(scope_id, scope_path);
            let value = items.add(Message::from_package(
                make_item_data(scope_id, item),
                message,
            ));

            items
                .get_mut(scope_id)
                .messages
                .insert(item.as_pascal().map_err(|e| e.to_owned())?.clone(), value);
        }

        for (segment, enum_ty) in manifest.enums.iter() {
            let enum_id = items.add(Type::from_package_enum(
                make_item_data(scope_id, &Identifier::from(segment.clone())),
                enum_ty,
            ));
            items
                .get_mut(scope_id)
                .types
                .insert(segment.clone(), enum_id);
        }

        Ok(scope_id)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StandardDefinitions {
    pub attributes: StandardAttributes,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StandardAttributes {
    pub debuggable: ItemId<Attribute>,
    pub networked: ItemId<Attribute>,
    pub resource: ItemId<Attribute>,
    pub maybe_resource: ItemId<Attribute>,
    pub store: ItemId<Attribute>,
    pub enum_: ItemId<Attribute>,
}

pub fn create_root_scope(
    items: &mut ItemMap,
) -> anyhow::Result<(ItemId<Scope>, StandardDefinitions)> {
    let root_scope = items.add(Scope::new(ItemData {
        parent_id: None,
        id: SnakeCaseIdentifier::default().into(),
        source: ItemSource::System,
    }));

    macro_rules! define_primitive_types {
        ($(($value:ident, $_type:ty)),*) => {
            [
                $((stringify!($value), PrimitiveType::$value)),*
            ]
        };
    }

    for (id, pt) in primitive_component_definitions!(define_primitive_types) {
        let id = PascalCaseIdentifier::new(id)
            .map_err(anyhow::Error::msg)
            .context("standard value was not valid snake-case")?;

        let ty = Type::new(
            ItemData {
                parent_id: Some(root_scope),
                id: id.clone().into(),
                source: ItemSource::System,
            },
            TypeInner::Primitive(pt),
        );
        let item_id = items.add(ty);
        items.get_mut(root_scope).types.insert(id, item_id);
    }

    fn make_attribute(
        items: &mut ItemMap,
        root_scope: ItemId<Scope>,
        name: &str,
    ) -> anyhow::Result<ItemId<Attribute>> {
        let id = PascalCaseIdentifier::new(name)
            .map_err(|e| anyhow::Error::msg(e.to_string()))
            .context("standard value was not valid snake-case")?;
        let item_id = items.add(Attribute {
            data: ItemData {
                parent_id: Some(root_scope),
                id: id.clone().into(),
                source: ItemSource::System,
            },
        });
        items.get_mut(root_scope).attributes.insert(id, item_id);
        Ok(item_id)
    }

    let attributes = StandardAttributes {
        debuggable: make_attribute(items, root_scope, "Debuggable")?,
        networked: make_attribute(items, root_scope, "Networked")?,
        resource: make_attribute(items, root_scope, "Resource")?,
        maybe_resource: make_attribute(items, root_scope, "MaybeResource")?,
        store: make_attribute(items, root_scope, "Store")?,
        enum_: make_attribute(items, root_scope, "Enum")?,
    };

    let standard_definitions = StandardDefinitions { attributes };
    Ok((root_scope, standard_definitions))
}

pub async fn package_dependency_to_retrievable_file(
    retrievable_manifest: &RetrievableFile,
    ignore_local_dependencies: bool,
    dependency: &ambient_package::Dependency,
) -> Result<Option<RetrievableFile>, PackageAddError> {
    let path = dependency
        .path
        .as_ref()
        .filter(|_| !ignore_local_dependencies);

    // path takes precedence over other remote access
    let retrievable_file = match (path, dependency.id_version(), &dependency.deployment) {
        (None, None, None) => None,
        (Some(path), _, _) => Some(retrievable_manifest.parent_join(&path.join("ambient.toml"))?),
        (_, Some((id, version)), _) => {
            let package_version_url = Url::parse(&format!(
                "{API_URL}/packages/versions/{id}/{version}"
            ))
            .map_err(|_| PackageAddError::InvalidPackageVersion {
                package_id: id.clone(),
                version: version.to_owned(),
            })?;

            let deployment = util::retrieve_url(&package_version_url)
                .await
                .map_err(|_| PackageAddError::InvalidPackageVersion {
                    package_id: id.clone(),
                    version: version.to_owned(),
                })?;

            Some(RetrievableFile::Deployment(RetrievableDeployment {
                id: deployment,
                path: PathBuf::from("ambient.toml"),
            }))
        }
        (_, _, Some(deployment)) => Some(RetrievableFile::Deployment(RetrievableDeployment {
            id: deployment.clone(),
            path: PathBuf::from("ambient.toml"),
        })),
    };
    Ok(retrievable_file)
}

/// This item supports being resolved by cloning.
pub(crate) trait Resolve: Item {
    fn resolve(self, items: &mut Semantic, self_id: ItemId<Self>) -> anyhow::Result<Self>;
    fn already_resolved(&self) -> bool;
}

fn get_type_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Type>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .types
        .get(item.as_pascal().ok()?)
        .copied()
}

fn get_attribute_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Attribute>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .attributes
        .get(item.as_pascal().ok()?)
        .copied()
}

fn get_concept_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Concept>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .concepts
        .get(item.as_pascal().ok()?)
        .copied()
}

fn get_component_id(
    items: &ItemMap,
    self_scope_id: ItemId<Scope>,
    path: ItemPath,
) -> Option<ItemId<Component>> {
    let (scope, item) = path.scope_and_item();
    items
        .get_scope(self_scope_id, scope)
        .ok()?
        .components
        .get(item.as_snake().ok()?)
        .copied()
}

#[derive(Error, Debug)]
pub enum ContextGetError<'a> {
    #[error("Failed to find {path} ({type_})")]
    NotFound { path: ItemPath<'a>, type_: ItemType },
}
impl ContextGetError<'_> {
    pub fn into_owned(self) -> ContextGetOwnedError {
        self.into()
    }
}
#[derive(Error, Debug)]
pub enum ContextGetOwnedError {
    #[error("Failed to find {path} ({type_})")]
    NotFound { path: ItemPathBuf, type_: ItemType },
}
impl From<ContextGetError<'_>> for ContextGetOwnedError {
    fn from(error: ContextGetError) -> Self {
        match error {
            ContextGetError::NotFound { path, type_ } => Self::NotFound {
                path: path.to_owned(),
                type_,
            },
        }
    }
}
