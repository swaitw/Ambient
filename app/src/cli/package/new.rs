use std::path::Path;

use ambient_native_std::{ambient_version, asset_cache::AssetCache};
use anyhow::Context;
use clap::{Parser, ValueEnum};
use convert_case::{Case, Casing};

use super::{build, PackageArgs};

#[derive(Parser, Clone, Debug)]
/// Create a new Ambient package
pub struct New {
    #[command(flatten)]
    pub package: PackageArgs,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(long)]
    api_path: Option<String>,

    /// This package is being created in an existing Rust workspace,
    /// and does not need to have extra files generated for it.
    #[arg(long)]
    in_workspace: bool,

    #[arg(long, value_enum, default_value_t)]
    rust: RustTemplate,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum RustTemplate {
    /// No template.
    None,
    /// An empty client/server.
    Empty,
    /// An empty client with a camera looking at a quad on the server.
    #[default]
    Quad,
}

pub(crate) async fn handle(args: &New, assets: &AssetCache) -> anyhow::Result<()> {
    let package_path = args.package.package_path()?;

    let Some(package_path) = &package_path.fs_path else {
        anyhow::bail!("Cannot create package in a remote directory.");
    };

    let name = args.name.as_deref();
    let api_path = args.api_path.as_deref();
    let in_workspace = args.in_workspace;

    // Build the identifier.
    let name = name.unwrap_or(
        package_path
            .file_name()
            .and_then(|s| s.to_str())
            .context("Package path has no terminating segment")?,
    );

    if package_path.is_dir() && std::fs::read_dir(package_path)?.next().is_some() {
        anyhow::bail!("package path {package_path:?} is not empty");
    }

    let id = ambient_package::PackageId::generate();
    let snake_case_name = name
        .to_case(Case::Snake)
        .replace(|c: char| !(c.is_ascii_alphanumeric() || c == '_'), "");

    // Build a list of files to write to disk, then write them all at once.
    macro_rules! template_file {
        ($path:expr) => {
            include_str!(concat!("new_package_template/", $path))
        };
    }

    macro_rules! template_path_and_file {
        ($path:expr) => {
            (Path::new($path), template_file!($path))
        };
    }

    let ambient_toml = include_str!("new_package_template/ambient.toml")
        .replace("{{id}}", id.as_str())
        .replace("{{name}}", name)
        .replace(
            "{{ambient_version}}",
            &format!("{}", ambient_native_std::ambient_version().version),
        );

    let cargo_toml = build_cargo_toml(package_path, api_path, snake_case_name);

    let mut files_to_write = vec![
        // root
        (Path::new("ambient.toml"), ambient_toml.as_str()),
    ];

    // Add all common Rust files.
    match args.rust {
        RustTemplate::None => {}
        RustTemplate::Empty | RustTemplate::Quad => {
            files_to_write.push((Path::new("Cargo.toml"), cargo_toml.as_str()));
            files_to_write.push(template_path_and_file!("build.rs"));

            if !in_workspace {
                files_to_write.extend_from_slice(&[
                    template_path_and_file!("rust-toolchain.toml"),
                    template_path_and_file!(".cargo/config.toml"),
                    template_path_and_file!(".vscode/settings.json"),
                ]);
            }
        }
    }

    // Add specific Rust files.
    match args.rust {
        RustTemplate::None => {}
        RustTemplate::Empty => {
            files_to_write.extend_from_slice(&[
                template_path_and_file!("src/client.rs"),
                (
                    Path::new("src/server.rs"),
                    template_file!("src/server_empty.rs"),
                ),
            ]);
        }
        RustTemplate::Quad => {
            files_to_write.extend_from_slice(&[
                template_path_and_file!("src/client.rs"),
                (
                    Path::new("src/server.rs"),
                    template_file!("src/server_quad.rs"),
                ),
            ]);
        }
    }

    if !in_workspace {
        files_to_write.extend_from_slice(&[
            template_path_and_file!(".gitignore"),
            template_path_and_file!(".vscode/launch.json"),
        ]);
    }

    for (path, contents) in files_to_write {
        let path = package_path.join(path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        std::fs::write(&path, contents).with_context(|| format!("Failed to create {path:?}"))?;
    }

    tracing::info!("Package \"{name}\" created; doing first build");

    // Build the new package to ensure that the user can use it immediately, and to have the proc-macro
    // ready for rust-analyzer to use
    build::handle_inner(&args.package, assets, false).await?;

    tracing::info!("Package \"{name}\" built successfully - ready to go at {package_path:?}");

    Ok(())
}

fn build_cargo_toml(
    package_path: &Path,
    api_path: Option<&str>,
    snake_case_name: String,
) -> String {
    let ((api_path, package_projection_path), in_guest_rust) =
        get_ambient_package_locations(package_path, api_path);

    let mut template_cargo_toml = include_str!("new_package_template/Cargo.toml")
        .replace("{{id}}", &snake_case_name)
        .replace(
            "ambient_api = { path = \"../../../../guest/rust/api\" }",
            &api_path,
        )
        .replace(
            r#"ambient_package_projection = { path = "../../../../guest/rust/api_core/package_projection" }"#,
            &package_projection_path
        );

    if in_guest_rust {
        template_cargo_toml = template_cargo_toml.replace(
            r#"version = "0.0.1""#,
            "rust-version = { workspace = true }\nversion = { workspace = true }",
        );
    }

    template_cargo_toml
}

fn get_ambient_package_locations(
    package_path: &Path,
    api_path: Option<&str>,
) -> ((String, String), bool) {
    if package_path
        .iter()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| w == ["guest", "rust"])
    {
        return (
            (
                r"ambient_api = { workspace = true }".to_string(),
                r"ambient_package_projection = { workspace = true }".to_string(),
            ),
            true,
        );
    }

    let version = ambient_version();
    let paths = if let Some(api_path) = api_path {
        tracing::info!("Ambient path: {}", api_path);

        let api_path = Path::new(api_path);
        let api_path = if api_path.is_relative() {
            pathdiff::diff_paths(
                api_path
                    .canonicalize()
                    .expect("failed to canonicalize API path; does it exist?"),
                package_path,
            )
            .expect("failed to compute relative path to API path")
        } else {
            api_path.to_owned()
        };

        let package_projection_path = ambient_std::path::normalize(
            &api_path
                .join("..")
                .join("api_core")
                .join("package_projection"),
        );

        (
            format!("ambient_api = {{ path = {:?} }}", api_path.display()),
            format!(
                "ambient_package_projection = {{ path = {:?} }}",
                package_projection_path.display()
            ),
        )
    } else if version.is_released_version() {
        tracing::info!("Ambient version: {}", version.version);
        (
            format!("ambient_api = \"{}\"", version.version),
            format!("ambient_package_projection = \"{}\"", version.version),
        )
    } else if let Some(tag) = version.tag() {
        tracing::info!("Ambient tag: {}", tag);

        let location =
            format!("{{ git = \"https://github.com/AmbientRun/Ambient.git\", tag = \"{tag}\" }}");
        (
            format!("ambient_api = {location}"),
            format!("ambient_package_projection = {location}"),
        )
    } else if !version.revision.is_empty() {
        tracing::info!("Ambient revision: {}", version.revision);

        let location = format!(
            "{{ git = \"https://github.com/AmbientRun/Ambient.git\", rev = \"{}\" }}",
            version.revision
        );
        (
            format!("ambient_api = {location}"),
            format!("ambient_package_projection = {location}"),
        )
    } else {
        tracing::info!("Ambient version: {}", version.version);
        (
            format!("ambient_api = \"{}\"", version.version),
            format!("ambient_package_projection = \"{}\"", version.version),
        )
    };

    (paths, false)
}
