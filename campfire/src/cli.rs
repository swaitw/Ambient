use clap::Parser;

use crate::{doc, golden_images, install, join, package, release, web};

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true, trailing_var_arg = true)]
pub enum Cli {
    /// Generate documentation for Ambient
    #[command(subcommand)]
    Doc(doc::Doc),
    /// Package-related functionality
    #[command(subcommand)]
    Package(package::Package),
    /// Running golden image tests
    GoldenImages(golden_images::GoldenImages),
    /// Release-related functionality
    #[command(subcommand)]
    Release(release::Release),
    /// Helper to install specific versions of Ambient
    Install(install::Install),
    /// Helper to join a server by various means
    Join(join::Join),

    #[command(subcommand)]
    /// Web-related functionality
    Web(web::Web),

    // Helper aliases for subcommands
    /// Clean all build artifacts for all packages.
    Clean,
    /// Run a package. Alias for `package run`.
    Run(package::Run),
    /// Serve a package. Alias for `package serve`.
    Serve(package::Run),
}

// https://rust-cli-recommendations.sunshowers.io/handling-arguments.html
