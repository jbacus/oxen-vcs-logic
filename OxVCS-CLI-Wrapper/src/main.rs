use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxenvcs-cli")]
#[command(about = "Oxen.ai CLI wrapper for Logic Pro version control", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Oxen repository
    Init {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    /// Stage changes
    Add {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    /// Create a commit
    Commit {
        #[arg(short, long)]
        message: String,
    },
    /// Show commit history
    Log,
    /// Restore to a previous commit
    Restore {
        #[arg(value_name = "COMMIT_ID")]
        commit_id: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            println!("Initializing repository at: {:?}", path);
            // TODO: Call liboxen init
            Ok(())
        }
        Commands::Add { path } => {
            println!("Staging: {:?}", path);
            // TODO: Call liboxen add
            Ok(())
        }
        Commands::Commit { message } => {
            println!("Committing: {}", message);
            // TODO: Call liboxen commit
            Ok(())
        }
        Commands::Log => {
            println!("Showing commit history");
            // TODO: Call liboxen log
            Ok(())
        }
        Commands::Restore { commit_id } => {
            println!("Restoring to: {}", commit_id);
            // TODO: Call liboxen restore
            Ok(())
        }
    }
}
