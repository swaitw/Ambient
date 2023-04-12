use clap::Parser;

mod doc;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum Cli {
    /// Generate documentation for Ambient
    Doc {},
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Doc { .. } => {
            doc::main()?;
        }
    }

    Ok(())
}
