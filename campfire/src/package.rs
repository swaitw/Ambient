use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Args, Parser};
use itertools::Itertools;

use crate::util::{all_directories_in, run_ambient};

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
pub enum Package {
    /// Clean all package build artifacts
    Clean,
    /// Run an package
    Run(Run),
    /// Serve an package
    Serve(Run),
    /// List all packages
    List,
    /// Run all the packages in order
    RunAll(RunParams),
    /// Check all the packages
    CheckAll(CheckAllParams),
    /// Build all standard packages
    BuildAll,
    /// Publish all standard packages
    DeployAll {
        #[arg(long)]
        token: String,
        #[arg(long)]
        include_examples: bool,
    },
    /// Regenerate IDs for all packages
    RegenerateIds,
}

#[derive(Parser, Clone)]
#[clap(trailing_var_arg = true)]
/// Run a package
pub struct Run {
    /// The name of the package to run
    pub package: String,
    #[command(flatten)]
    pub params: RunParams,
}

#[derive(Args, Clone, Debug)]
#[clap(trailing_var_arg = true)]
/// Run a package
pub struct RunParams {
    /// Whether or not to run Ambient in release mode
    #[arg(short, long, default_value_t = false)]
    pub release: bool,
    /// The args to pass through to `ambient`
    pub args: Vec<String>,
}

#[derive(Args, Clone, Debug)]
#[clap(trailing_var_arg = true)]
/// Check all packages for compilation, not build, errors
pub struct CheckAllParams {
    /// Whether or not to delete the target folder before runs.
    /// Used by the CI to avoid running out of space.
    #[arg(short, long, default_value_t = false)]
    pub delete_target: bool,
}

pub fn main(args: &Package) -> anyhow::Result<()> {
    match args {
        Package::Clean => clean(),
        Package::Run(args) => run(args),
        Package::Serve(args) => serve(args),
        Package::List => list(),
        Package::RunAll(params) => run_all(params),
        Package::CheckAll(params) => check_all(params),
        Package::BuildAll => build_all(),
        Package::DeployAll {
            token,
            include_examples,
        } => deploy_all(token, *include_examples),
        Package::RegenerateIds => regenerate_ids(),
    }
}

pub fn clean() -> anyhow::Result<()> {
    tracing::info!("Cleaning examples...");
    for path in get_all_packages(true, true, true)? {
        let build_path = path.join("build");
        if !build_path.exists() {
            continue;
        }

        std::fs::remove_dir_all(&build_path)?;
        tracing::info!("Removed build directory for {}.", path.display());
    }
    tracing::info!("Done cleaning examples.");
    Ok(())
}

pub fn run(args: &Run) -> anyhow::Result<()> {
    let Run { package, params } = args;
    let path = find_package(package)?;

    tracing::info!("Running example {} (params: {params:?})...", path.display());
    run_package("run", &path, params)
}

pub fn serve(args: &Run) -> anyhow::Result<()> {
    let Run { package, params } = args;
    let path = find_package(package)?;

    tracing::info!("Serving example {} (params: {params:?})...", path.display());
    run_package("serve", &path, params)
}

fn find_package(package: &String) -> anyhow::Result<PathBuf> {
    get_all_packages(true, true, false)?
        .into_iter()
        .find(|p| p.ends_with(package))
        .ok_or_else(|| anyhow::anyhow!("no example found with name {}", package))
}

fn list() -> anyhow::Result<()> {
    for path in get_all_packages(true, true, true)? {
        println!("{}", path.display());
    }

    Ok(())
}

fn run_all(params: &RunParams) -> anyhow::Result<()> {
    for path in get_all_packages(true, true, false)? {
        tracing::info!("Running example {} (params: {params:?})...", path.display());
        run_package("run", &path, params)?;
    }

    Ok(())
}

fn check_all(params: &CheckAllParams) -> anyhow::Result<()> {
    // Rust
    {
        let root_path = Path::new("guest/rust");
        tracing::info!("Checking Rust guest code...");

        for features in ["", "client", "server", "client,server"] {
            if params.delete_target {
                tracing::info!("Deleting target directory...");
                let target_path = root_path.join("target");
                if target_path.exists() {
                    std::fs::remove_dir_all(&target_path)?;
                }
            }

            tracing::info!("Checking Rust guest code with features `{}`...", features);

            let mut command = std::process::Command::new("cargo");
            command.current_dir(root_path);
            command.args(["clippy"]);
            command.env("RUSTFLAGS", "-Dwarnings");

            if !features.is_empty() {
                command.args(["--features", features]);
            }
            command.args(["--", "-A", "clippy::collapsible-if"]);

            if !command.spawn()?.wait()?.success() {
                anyhow::bail!(
                    "Failed to check Rust guest code with features {}.",
                    features
                );
            }
        }

        tracing::info!("Checked Rust guest code.");
    }

    Ok(())
}

pub fn build_all() -> anyhow::Result<()> {
    let package_paths = get_all_packages(true, true, true)?;

    for path in &package_paths {
        run_ambient(
            &["build", &path.to_string_lossy(), "--clean-build"],
            true,
            false,
        )?;
    }

    Ok(())
}

pub fn deploy_all(token: &str, include_examples: bool) -> anyhow::Result<()> {
    let paths = get_all_packages(include_examples, false, true)?;
    let paths = paths.iter().map(|p| p.to_string_lossy()).collect_vec();

    let mut args = vec!["deploy"];
    for (idx, path) in paths.iter().enumerate() {
        if idx != 0 {
            args.push("--extra-packages");
        }
        args.push(path);
    }
    args.push("--token");
    args.push(token);
    args.push("--clean-build");

    run_ambient(&args, true, true)
}

fn run_package(run_cmd: &str, path: &Path, params: &RunParams) -> anyhow::Result<()> {
    let mut args = vec![run_cmd];
    let path = path.to_string_lossy();
    args.push(&path);
    if !params.args.is_empty() {
        args.extend(params.args.iter().map(|s| s.as_str()));
    }
    run_ambient(&args, params.release, false)
}

pub fn get_all_packages(
    include_examples: bool,
    include_testcases: bool,
    include_mods: bool,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut package_paths = vec![];
    for category in all_directories_in(Path::new("guest/rust/packages"))? {
        for package in all_directories_in(&category.path())? {
            let package_path = package.path();
            package_paths.push(package_path.clone());

            if include_mods {
                let mods_path = package_path.join("mods");
                if mods_path.is_dir() {
                    for mod_path in all_directories_in(&mods_path)? {
                        package_paths.push(mod_path.path());
                    }
                }
            }

            {
                let core_path = package_path.join("core");
                if core_path.is_dir() {
                    if core_path.join("ambient.toml").is_file() {
                        package_paths.push(core_path);
                    } else {
                        for core_package in all_directories_in(&core_path)? {
                            package_paths.push(core_package.path());
                        }
                    }
                }
            }

            {
                let schema_path = package_path.join("schema");
                if schema_path.is_dir() {
                    package_paths.push(schema_path);
                }
            }
        }
    }
    if include_examples {
        package_paths.append(&mut get_all_examples(include_testcases)?);
    }

    package_paths.sort();
    Ok(package_paths)
}

fn get_all_examples(include_testcases: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut examples = Vec::new();

    for guest in all_directories_in(Path::new("guest")).context("Failed to find guest directory")? {
        let examples_path = guest.path().join("examples");
        let dirs = match all_directories_in(&examples_path) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to query examples directory at {examples_path:?}: {e}");
                continue;
            }
        };

        for category in dirs {
            for example in all_directories_in(&category.path())? {
                let example_path = example.path();
                examples.push(example_path.clone());

                // Hacky workaround for dependencies example
                {
                    let deps_path = example_path.join("deps");
                    if deps_path.is_dir() {
                        for deps_package in all_directories_in(&deps_path)? {
                            examples.push(deps_package.path());
                        }
                    }
                }
            }
        }

        if include_testcases {
            let testcases_path = guest.path().join("testcases");
            if testcases_path.exists() {
                for entry in all_directories_in(&testcases_path)? {
                    examples.push(entry.path());
                }
            }
        }
    }

    examples.sort_by_key(|path| path.clone());

    Ok(examples)
}

fn regenerate_ids() -> anyhow::Result<()> {
    for path in get_all_packages(true, true, true)? {
        println!("Regenerating ID for {path:?}");
        run_ambient(
            &["package", "regenerate-id", &path.to_string_lossy()],
            true,
            false,
        )?;
    }

    Ok(())
}
