use std::{collections::HashMap, fmt::Display, path::PathBuf};

use indexmap::IndexMap;
use rand::Rng;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use thiserror::Error;

use crate::{
    Component, Concept, Enum, ItemPathBuf, Message, PascalCaseIdentifier, SnakeCaseIdentifier,
};

#[derive(Error, Debug, PartialEq)]
pub enum ManifestParseError {
    #[error("manifest was not valid TOML: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("manifest contains a project and/or an ember section; projects/embers have been renamed to packages")]
    ProjectEmberRenamedToPackageError,
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Serialize)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    #[serde(alias = "component")]
    pub components: IndexMap<ItemPathBuf, Component>,
    #[serde(default)]
    #[serde(alias = "concept")]
    pub concepts: IndexMap<ItemPathBuf, Concept>,
    #[serde(default)]
    #[serde(alias = "message")]
    pub messages: IndexMap<ItemPathBuf, Message>,
    #[serde(default)]
    #[serde(alias = "enum")]
    pub enums: IndexMap<PascalCaseIdentifier, Enum>,
    #[serde(default)]
    pub includes: HashMap<SnakeCaseIdentifier, PathBuf>,
    #[serde(default)]
    pub dependencies: IndexMap<SnakeCaseIdentifier, Dependency>,
    #[serde(default)]
    pub hosting: Hosting,
}
impl Manifest {
    pub fn parse(manifest: &str) -> Result<Self, ManifestParseError> {
        let raw = toml::from_str::<toml::Table>(manifest)?;
        if raw.contains_key("project") || raw.contains_key("ember") {
            return Err(ManifestParseError::ProjectEmberRenamedToPackageError);
        }

        Ok(toml::from_str(manifest)?)
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string_pretty(self).unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Default, Serialize)]
#[serde(transparent)]
/// A checksummed package ID. Guaranteed to be a valid `SnakeCaseIdentifier` as well.
pub struct PackageId(pub(crate) String);
impl<'de> Deserialize<'de> for PackageId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PackageId::new(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}
impl PackageId {
    const DATA_LENGTH: usize = 12;
    const CHECKSUM_LENGTH: usize = 8;
    const TOTAL_LENGTH: usize = Self::DATA_LENGTH + Self::CHECKSUM_LENGTH;
    // to ensure that the first character is always alphabetic we have to make sure that the highest 5 bits of the
    // first byte are at most 25 (as Base32 encodes every 5 bits as 1 character => 0-25 as A-Z and 26-31 as digits)
    // so the max value looks like this:
    // 11001    = 25 (Base32 'Z') on highest 5 bits
    //      111 = max for the lowest 3 since it can be anything
    #[allow(clippy::unusual_byte_groupings)]
    const MAX_VALUE_FOR_FIRST_BYTE: u8 = 0b11001_111;

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Attempts to create a new package ID from a string.
    pub fn new(id: &str) -> Result<Self, String> {
        Self::validate(id)?;
        Ok(Self(id.to_string()))
    }

    /// Generates a new package ID.
    pub fn generate() -> Self {
        let mut data: [u8; Self::DATA_LENGTH] = rand::random();
        data[0] = rand::thread_rng().gen_range(0..=Self::MAX_VALUE_FOR_FIRST_BYTE);
        let checksum: [u8; Self::CHECKSUM_LENGTH] = sha2::Sha256::digest(data)
            [0..Self::CHECKSUM_LENGTH]
            .try_into()
            .unwrap();

        let mut bytes = [0u8; Self::TOTAL_LENGTH];
        bytes[0..Self::DATA_LENGTH].copy_from_slice(&data);
        bytes[Self::DATA_LENGTH..].copy_from_slice(&checksum);

        let output = data_encoding::BASE32_NOPAD
            .encode(&bytes)
            .to_ascii_lowercase();

        assert!(output.chars().next().unwrap().is_ascii_alphabetic());
        Self(output)
    }

    /// Validate that a package ID is correct.
    pub fn validate(id: &str) -> Result<(), String> {
        let cmd =
            "Use `ambient package regenerate-id` to regenerate the package ID with the new format.";

        let bytes = data_encoding::BASE32_NOPAD
            .decode(id.to_ascii_uppercase().as_bytes())
            .map_err(|e| format!("Package ID contained invalid characters: {e}. {cmd}"))?;

        let data = &bytes[0..Self::DATA_LENGTH];
        let checksum = &bytes[Self::DATA_LENGTH..];

        let expected_checksum = &sha2::Sha256::digest(data)[0..Self::CHECKSUM_LENGTH];
        if checksum != expected_checksum {
            return Err(format!(
                "Package ID contained invalid checksum: expected {:?}, got {:?}. {cmd}",
                expected_checksum, checksum
            ));
        }

        Ok(())
    }
}
impl Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<PackageId> for SnakeCaseIdentifier {
    fn from(id: PackageId) -> Self {
        SnakeCaseIdentifier(id.0)
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Package {
    /// The ID can be optional if and only if the package is `ambient_core` or an include.
    #[serde(default)]
    pub id: Option<PackageId>,
    pub name: String,
    pub version: Version,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub ambient_version: Option<VersionReq>,
    #[serde(default)]
    pub authors: Vec<String>,
    pub content: PackageContent,
    #[serde(default = "return_true")]
    pub public: bool,
}
impl Default for Package {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            version: Version::parse("0.0.0").unwrap(),
            description: Default::default(),
            repository: Default::default(),
            ambient_version: Default::default(),
            authors: Default::default(),
            content: Default::default(),
            public: true,
        }
    }
}

fn return_true() -> bool {
    true
}

// ----- NOTE: Update docs/reference/package.md when changing this ----

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PackageContent {
    Playable {
        #[serde(default)]
        example: bool,
    },
    /// Assets are something that you can use as a dependency in your package
    Asset {
        #[serde(default)]
        models: bool,
        #[serde(default)]
        animations: bool,
        #[serde(default)]
        textures: bool,
        #[serde(default)]
        materials: bool,
        #[serde(default)]
        audio: bool,
        #[serde(default)]
        fonts: bool,
        #[serde(default)]
        code: bool,
        #[serde(default)]
        schema: bool,
    },
    Tool,
    Mod {
        /// List of package ids that this mod is applicable to
        #[serde(default)]
        for_playables: Vec<String>,
    },
}
impl Default for PackageContent {
    fn default() -> Self {
        Self::Playable { example: false }
    }
}

// -----------------------------------------------------------------

#[derive(Deserialize, Clone, Debug, PartialEq, Default, Serialize)]
pub struct Build {
    #[serde(default)]
    pub rust: BuildRust,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct BuildRust {
    #[serde(rename = "feature-multibuild")]
    pub feature_multibuild: Vec<String>,
}
impl Default for BuildRust {
    fn default() -> Self {
        Self {
            feature_multibuild: vec!["client".to_string(), "server".to_string()],
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Serialize)]
pub struct Dependency {
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub id: Option<PackageId>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub deployment: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}
impl Dependency {
    pub fn has_remote_dependency(&self) -> bool {
        self.deployment.is_some() || self.version.is_some()
    }

    pub fn id_version(&self) -> Option<(&PackageId, &str)> {
        self.id.as_ref().zip(self.version.as_deref())
    }
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Serialize)]
pub struct Hosting {
    /// The region to host in
    #[serde(default)]
    pub region: Region,
    /// The maximum number of players that can be connected at once (0 = unlimited)
    #[serde(default)]
    pub max_players: usize,
}

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Serialize)]
pub enum Region {
    /// Automatically select the best region based on the player's location
    #[default]
    Auto,
    /// Always use the EU region
    EU,
    /// Always use the US region
    US,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use indexmap::IndexMap;

    use crate::{
        Build, BuildRust, Component, ComponentType, Components, Concept, ConceptValue,
        ContainerType, Dependency, Enum, Identifier, ItemPathBuf, Manifest, ManifestParseError,
        Package, PackageId, PascalCaseIdentifier, SnakeCaseIdentifier,
    };
    use semver::Version;

    fn i(s: &str) -> Identifier {
        Identifier::new(s).unwrap()
    }

    fn sci(s: &str) -> SnakeCaseIdentifier {
        SnakeCaseIdentifier::new(s).unwrap()
    }

    fn pci(s: &str) -> PascalCaseIdentifier {
        PascalCaseIdentifier::new(s).unwrap()
    }

    fn ipb(s: &str) -> ItemPathBuf {
        ItemPathBuf::new(s).unwrap()
    }

    #[test]
    fn can_parse_minimal_toml() {
        const TOML: &str = r#"
        [package]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "Test"
        version = "0.0.1"
        content = { type = "Playable" }
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("lktsfudbjw2qikhyumt573ozxhadkiwm".to_string())),
                    name: "Test".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            })
        );
    }

    #[test]
    fn will_fail_on_legacy_project_toml() {
        const TOML: &str = r#"
        [project]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "Test"
        version = "0.0.1"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Err(ManifestParseError::ProjectEmberRenamedToPackageError)
        )
    }

    #[test]
    fn can_parse_tictactoe_toml() {
        const TOML: &str = r#"
        [package]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "Tic Tac Toe"
        version = "0.0.1"
        content = { type = "Playable" }

        [components]
        cell = { type = "i32", name = "Cell", description = "The ID of the cell this player is in", attributes = ["store"] }

        [concepts.Cell]
        name = "Cell"
        description = "A cell object"
        [concepts.Cell.components.required]
        cell = {}
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("lktsfudbjw2qikhyumt573ozxhadkiwm".to_string())),
                    name: "Tic Tac Toe".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([(
                    ipb("cell"),
                    Component {
                        name: Some("Cell".to_string()),
                        description: Some("The ID of the cell this player is in".to_string()),
                        type_: ComponentType::Item(i("i32").into()),
                        attributes: vec![i("store").into()],
                        default: None,
                    }
                )]),
                concepts: IndexMap::from_iter([(
                    ipb("Cell"),
                    Concept {
                        name: Some("Cell".to_string()),
                        description: Some("A cell object".to_string()),
                        extends: vec![],
                        components: Components {
                            required: IndexMap::from_iter([(ipb("cell"), ConceptValue::default())]),
                            optional: Default::default()
                        }
                    }
                )]),
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_rust_build_settings() {
        const TOML: &str = r#"
        [package]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "Tic Tac Toe"
        version = "0.0.1"
        content = { type = "Playable" }
        ambient_version = "0.3.0-nightly-2023-08-31"

        [build.rust]
        feature-multibuild = ["client"]
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("lktsfudbjw2qikhyumt573ozxhadkiwm".to_string())),
                    name: "Tic Tac Toe".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ambient_version: Some(
                        semver::VersionReq::parse("0.3.0-nightly-2023-08-31").unwrap()
                    ),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string()]
                    }
                },
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_concepts_with_documented_namespace_from_manifest() {
        use toml::Value;

        const TOML: &str = r#"
        [package]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "My Package"
        version = "0.0.1"
        content = { type = "Playable" }

        [components]
        "core::transform::rotation" = { type = "quat", name = "Rotation", description = "" }
        "core::transform::scale" = { type = "vec3", name = "Scale", description = "" }
        "core::transform::spherical_billboard" = { type = "empty", name = "Spherical billboard", description = "" }
        "core::transform::translation" = { type = "vec3", name = "Translation", description = "" }

        [concepts."ns::Transformable"]
        name = "Transformable"
        description = "Can be translated, rotated and scaled."

        [concepts."ns::Transformable".components.required]
        # This is intentionally out of order to ensure that order is preserved
        "core::transform::translation" = { suggested = [0, 0, 0] }
        "core::transform::scale" = { suggested = [1, 1, 1] }
        "core::transform::rotation" = { suggested = [0, 0, 0, 1] }

        [concepts."ns::Transformable".components.optional]
        "core::transform::inv_local_to_world" = { description = "If specified, will be automatically updated" }
        "#;

        let manifest = Manifest::parse(TOML).unwrap();
        assert_eq!(
            manifest,
            Manifest {
                package: Package {
                    id: Some(PackageId("lktsfudbjw2qikhyumt573ozxhadkiwm".to_string())),
                    name: "My Package".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ipb("core::transform::rotation"),
                        Component {
                            name: Some("Rotation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("quat").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("core::transform::scale"),
                        Component {
                            name: Some("Scale".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("core::transform::spherical_billboard"),
                        Component {
                            name: Some("Spherical billboard".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("empty").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("core::transform::translation"),
                        Component {
                            name: Some("Translation".to_string()),
                            description: Some("".to_string()),
                            type_: ComponentType::Item(i("vec3").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                ]),
                concepts: IndexMap::from_iter([(
                    ipb("ns::Transformable"),
                    Concept {
                        name: Some("Transformable".to_string()),
                        description: Some("Can be translated, rotated and scaled.".to_string()),
                        extends: vec![],
                        components: Components {
                            required: IndexMap::from_iter([
                                (
                                    ipb("core::transform::translation"),
                                    ConceptValue {
                                        suggested: Some(Value::Array(vec![
                                            Value::Integer(0),
                                            Value::Integer(0),
                                            Value::Integer(0)
                                        ])),
                                        ..Default::default()
                                    }
                                ),
                                (
                                    ipb("core::transform::scale"),
                                    ConceptValue {
                                        suggested: Some(Value::Array(vec![
                                            Value::Integer(1),
                                            Value::Integer(1),
                                            Value::Integer(1)
                                        ])),
                                        ..Default::default()
                                    }
                                ),
                                (
                                    ipb("core::transform::rotation"),
                                    ConceptValue {
                                        suggested: Some(Value::Array(vec![
                                            Value::Integer(0),
                                            Value::Integer(0),
                                            Value::Integer(0),
                                            Value::Integer(1)
                                        ])),
                                        ..Default::default()
                                    }
                                ),
                            ]),
                            optional: IndexMap::from_iter([(
                                ipb("core::transform::inv_local_to_world"),
                                ConceptValue {
                                    description: Some(
                                        "If specified, will be automatically updated".to_string()
                                    ),
                                    ..Default::default()
                                },
                            )])
                        }
                    }
                )]),
                ..Default::default()
            }
        );

        assert_eq!(
            manifest
                .concepts
                .first()
                .unwrap()
                .1
                .components
                .required
                .keys()
                .collect::<Vec<_>>(),
            vec![
                &ipb("core::transform::translation"),
                &ipb("core::transform::scale"),
                &ipb("core::transform::rotation"),
            ]
        );
    }

    #[test]
    fn can_parse_enums() {
        const TOML: &str = r#"
        [package]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "Tic Tac Toe"
        version = "0.0.1"
        content = { type = "Playable" }

        [enums.CellState]
        description = "The current cell state"
        [enums.CellState.members]
        Taken = "The cell is taken"
        Free = "The cell is free"
        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("lktsfudbjw2qikhyumt573ozxhadkiwm".to_string())),
                    name: "Tic Tac Toe".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ..Default::default()
                },
                enums: IndexMap::from_iter([(
                    pci("CellState"),
                    Enum {
                        description: Some("The current cell state".to_string()),
                        members: IndexMap::from_iter([
                            (pci("Taken"), "The cell is taken".to_string()),
                            (pci("Free"), "The cell is free".to_string()),
                        ])
                    }
                )]),
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_container_types() {
        const TOML: &str = r#"
        [package]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "Test"
        version = "0.0.1"
        content = { type = "Playable" }

        [components]
        test = { type = "I32", name = "Test", description = "Test" }
        vec_test = { type = { container_type = "Vec", element_type = "I32" }, name = "Test", description = "Test" }
        option_test = { type = { container_type = "Option", element_type = "I32" }, name = "Test", description = "Test" }

        "#;

        assert_eq!(
            Manifest::parse(TOML),
            Ok(Manifest {
                package: Package {
                    id: Some(PackageId("lktsfudbjw2qikhyumt573ozxhadkiwm".to_string())),
                    name: "Test".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ..Default::default()
                },
                build: Build {
                    rust: BuildRust {
                        feature_multibuild: vec!["client".to_string(), "server".to_string()]
                    }
                },
                components: IndexMap::from_iter([
                    (
                        ipb("test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Item(i("I32").into()),
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("vec_test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Vec,
                                element_type: i("I32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                    ),
                    (
                        ipb("option_test"),
                        Component {
                            name: Some("Test".to_string()),
                            description: Some("Test".to_string()),
                            type_: ComponentType::Contained {
                                type_: ContainerType::Option,
                                element_type: i("I32").into()
                            },
                            attributes: vec![],
                            default: None,
                        }
                    )
                ]),
                ..Default::default()
            })
        )
    }

    #[test]
    fn can_parse_dependencies() {
        const TOML: &str = r#"
        [package]
        id = "lktsfudbjw2qikhyumt573ozxhadkiwm"
        name = "dependencies"
        version = "0.0.1"
        content = { type = "Playable" }

        [dependencies]
        deps_assets = { path = "deps/assets" }
        deps_code = { path = "deps/code" }
        deps_ignore_me = { path = "deps/ignore_me", enabled = false }
        deps_remote_deployment = { deployment = "jhsdfu574S" }
        deps_remote_deployment_pkg_ver = { id = "cezekiuth6khuiykw66bmepsggaoztyv", version = "0.1.0" }

        "#;

        let manifest = Manifest::parse(TOML).unwrap();
        assert_eq!(
            manifest
                .dependencies
                .get(&sci("deps_remote_deployment_pkg_ver"))
                .unwrap()
                .id_version(),
            Some((
                &PackageId("cezekiuth6khuiykw66bmepsggaoztyv".to_owned()),
                "0.1.0"
            ))
        );

        assert_eq!(
            manifest,
            Manifest {
                package: Package {
                    id: Some(PackageId("lktsfudbjw2qikhyumt573ozxhadkiwm".to_string())),
                    name: "dependencies".to_string(),
                    version: Version::parse("0.0.1").unwrap(),
                    ..Default::default()
                },
                dependencies: IndexMap::from_iter([
                    (
                        sci("deps_assets"),
                        Dependency {
                            path: Some(PathBuf::from("deps/assets")),
                            id: None,
                            version: None,
                            deployment: None,
                            enabled: None,
                        }
                    ),
                    (
                        sci("deps_code"),
                        Dependency {
                            path: Some(PathBuf::from("deps/code")),
                            id: None,
                            version: None,
                            deployment: None,
                            enabled: None,
                        }
                    ),
                    (
                        sci("deps_ignore_me"),
                        Dependency {
                            path: Some(PathBuf::from("deps/ignore_me")),
                            id: None,
                            version: None,
                            deployment: None,
                            enabled: Some(false),
                        }
                    ),
                    (
                        sci("deps_remote_deployment"),
                        Dependency {
                            path: None,
                            id: None,
                            version: None,
                            deployment: Some("jhsdfu574S".to_owned()),
                            enabled: None,
                        }
                    ),
                    (
                        sci("deps_remote_deployment_pkg_ver"),
                        Dependency {
                            path: None,
                            id: Some(PackageId("cezekiuth6khuiykw66bmepsggaoztyv".to_owned())),
                            version: Some("0.1.0".to_owned()),
                            deployment: None,
                            enabled: None,
                        }
                    ),
                ]),
                ..Default::default()
            }
        )
    }
}
