use clap::{Parser, Subcommand};
use oxenvcs_cli::{CommitMetadata, OxenRepository};
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
    /// Initialize a new Oxen repository for a Logic Pro project
    Init {
        #[arg(value_name = "PATH")]
        path: PathBuf,

        #[arg(long, help = "Initialize for Logic Pro project (auto-detect and configure)")]
        logic: bool,
    },

    /// Stage changes
    Add {
        #[arg(value_name = "PATHS")]
        paths: Vec<PathBuf>,

        #[arg(long, help = "Stage all changes")]
        all: bool,
    },

    /// Create a commit
    Commit {
        #[arg(short, long, help = "Commit message")]
        message: String,

        #[arg(long, help = "Beats per minute")]
        bpm: Option<f32>,

        #[arg(long, help = "Sample rate (Hz)")]
        sample_rate: Option<u32>,

        #[arg(long, help = "Key signature (e.g., 'C Major')")]
        key: Option<String>,

        #[arg(long, help = "Tags (comma-separated)")]
        tags: Option<String>,
    },

    /// Show commit history
    Log {
        #[arg(short, long, help = "Number of commits to show")]
        limit: Option<usize>,
    },

    /// Restore to a previous commit
    Restore {
        #[arg(value_name = "COMMIT_ID")]
        commit_id: String,
    },

    /// Show repository status
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, logic } => {
            if logic {
                let repo = OxenRepository::init_for_logic_project(&path).await?;
                println!("✓ Successfully initialized Logic Pro project repository");
            } else {
                let repo = OxenRepository::init(&path).await?;
                println!("✓ Successfully initialized Oxen repository at: {}", path.display());
            }
            Ok(())
        }

        Commands::Add { paths, all } => {
            let repo = OxenRepository::new(".");

            if all {
                repo.stage_all().await?;
            } else {
                if paths.is_empty() {
                    eprintln!("Error: Please provide paths to stage or use --all");
                    std::process::exit(1);
                }
                repo.stage_changes(paths).await?;
            }

            Ok(())
        }

        Commands::Commit {
            message,
            bpm,
            sample_rate,
            key,
            tags,
        } => {
            let repo = OxenRepository::new(".");

            let mut metadata = CommitMetadata::new(message);

            if let Some(bpm) = bpm {
                metadata = metadata.with_bpm(bpm);
            }

            if let Some(sr) = sample_rate {
                metadata = metadata.with_sample_rate(sr);
            }

            if let Some(key) = key {
                metadata = metadata.with_key_signature(key);
            }

            if let Some(tags_str) = tags {
                for tag in tags_str.split(',') {
                    metadata = metadata.with_tag(tag.trim());
                }
            }

            let commit_id = repo.create_commit(metadata).await?;
            println!("✓ Commit created: {}", commit_id);

            Ok(())
        }

        Commands::Log { limit } => {
            let repo = OxenRepository::new(".");

            let commits = repo.get_history(limit).await?;

            if commits.is_empty() {
                println!("No commits yet");
                return Ok(());
            }

            println!("\nCommit History:\n");

            for commit in commits {
                println!("Commit: {}", commit.id);
                println!("Author: {}", commit.author);
                println!("Date:   {}", commit.timestamp);
                println!("\n    {}\n", commit.message.lines().collect::<Vec<_>>().join("\n    "));
                println!("{}", "─".repeat(80));
            }

            Ok(())
        }

        Commands::Restore { commit_id } => {
            let repo = OxenRepository::new(".");

            repo.restore(&commit_id).await?;

            Ok(())
        }

        Commands::Status => {
            let repo = OxenRepository::new(".");

            let status = repo.status().await?;

            println!("\nRepository Status:\n");

            if !status.staged_files.is_empty() {
                println!("Staged files:");
                for entry in &status.staged_files {
                    println!("  + {}", entry.filename);
                }
                println!();
            }

            if !status.modified_files.is_empty() {
                println!("Modified files:");
                for entry in &status.modified_files {
                    println!("  M {}", entry.filename);
                }
                println!();
            }

            if !status.untracked_files.is_empty() {
                println!("Untracked files:");
                for path in &status.untracked_files {
                    println!("  ? {}", path.display());
                }
                println!();
            }

            if status.staged_files.is_empty()
                && status.modified_files.is_empty()
                && status.untracked_files.is_empty()
            {
                println!("Working directory clean");
            }

            Ok(())
        }
    }
}
