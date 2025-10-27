use clap::{Parser, Subcommand};
use colored::Colorize;
use oxenvcs_cli::{logger, CommitMetadata, OxenRepository, vlog, info, success, error};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxenvcs-cli")]
#[command(version)]
#[command(about = "Oxen.ai CLI wrapper for Logic Pro version control")]
#[command(long_about = "Oxen.ai CLI wrapper for Logic Pro version control

This tool provides Git-like version control specifically designed for Logic Pro
projects (.logicx). It integrates with Oxen.ai to efficiently track and manage
your music production projects with support for audio-specific metadata.

FEATURES:
  • Automatic detection and setup for Logic Pro projects
  • Audio metadata tracking (BPM, sample rate, key signature)
  • Draft branch workflow for auto-commits
  • Efficient handling of large audio files
  • Ignore patterns for cache and temporary files

BASIC WORKFLOW:
  1. Initialize: oxenvcs-cli init --logic .
  2. Make changes in Logic Pro
  3. Stage: oxenvcs-cli add --all
  4. Commit: oxenvcs-cli commit -m \"Added drum track\" --bpm 120
  5. View history: oxenvcs-cli log

For more information, visit: https://github.com/your-repo")]
struct Cli {
    /// Enable verbose debug output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Oxen repository for a Logic Pro project
    #[command(long_about = "Initialize a new Oxen repository for a Logic Pro project

USAGE:
    oxenvcs-cli init --logic <PATH>
    oxenvcs-cli init <PATH>

DESCRIPTION:
    Creates a new Oxen repository at the specified path. When used with --logic,
    it will:
      • Detect and validate the Logic Pro project structure
      • Create an .oxenignore file with Logic-specific patterns
      • Set up a draft branch workflow for auto-commits
      • Initialize tracking for projectData, Alternatives, and Resources

    The PATH can be:
      • Current directory: .
      • Relative path: Demo_Project.logicx
      • Absolute path: /Users/you/Music/Logic/MyProject.logicx

EXAMPLES:
    # Initialize from inside a Logic project directory
    cd Demo_Project.logicx
    oxenvcs-cli init --logic .

    # Initialize from parent directory
    oxenvcs-cli init --logic Demo_Project.logicx

    # Initialize a generic Oxen repository (not Logic-specific)
    oxenvcs-cli init /path/to/folder")]
    Init {
        #[arg(value_name = "PATH", help = "Path to the project directory")]
        path: PathBuf,

        #[arg(long, help = "Initialize for Logic Pro project (auto-detect and configure)")]
        logic: bool,
    },

    /// Stage changes to be committed
    #[command(long_about = "Stage changes to be committed

USAGE:
    oxenvcs-cli add --all
    oxenvcs-cli add <PATHS>...

DESCRIPTION:
    Stages changes in the working directory for the next commit. Similar to 'git add'.
    Use --all to stage all changes, or specify individual files/directories.

EXAMPLES:
    # Stage all changes in the repository
    oxenvcs-cli add --all

    # Stage specific files
    oxenvcs-cli add projectData Alternatives/Take001

    # Stage a directory
    oxenvcs-cli add Resources/")]
    Add {
        #[arg(value_name = "PATHS", help = "Files or directories to stage")]
        paths: Vec<PathBuf>,

        #[arg(long, short, help = "Stage all changes in the repository")]
        all: bool,
    },

    /// Create a commit with optional audio metadata
    #[command(long_about = "Create a commit with optional audio metadata

USAGE:
    oxenvcs-cli commit -m <MESSAGE> [OPTIONS]

DESCRIPTION:
    Creates a new commit with the currently staged changes. You can attach
    audio production metadata to help track the evolution of your project.

    Metadata includes:
      • BPM (tempo)
      • Sample rate (Hz)
      • Key signature
      • Tags for categorization

EXAMPLES:
    # Simple commit
    oxenvcs-cli commit -m \"Initial project setup\"

    # Commit with audio metadata
    oxenvcs-cli commit -m \"Added bass line\" --bpm 120 --key \"A Minor\"

    # Commit with multiple tags
    oxenvcs-cli commit -m \"Final mix\" --sample-rate 44100 --tags \"mixing,mastered\"

    # Full metadata commit
    oxenvcs-cli commit -m \"Verse 2 complete\" \\
        --bpm 128 \\
        --sample-rate 48000 \\
        --key \"C Major\" \\
        --tags \"verse,arrangement\"")]
    Commit {
        #[arg(short, long, help = "Commit message describing the changes")]
        message: String,

        #[arg(long, help = "Beats per minute (tempo) of the project")]
        bpm: Option<f32>,

        #[arg(long, help = "Sample rate in Hz (e.g., 44100, 48000, 96000)")]
        sample_rate: Option<u32>,

        #[arg(long, help = "Key signature (e.g., 'C Major', 'A Minor', 'F# Minor')")]
        key: Option<String>,

        #[arg(long, help = "Tags for categorization (comma-separated, e.g., 'mixing,draft')")]
        tags: Option<String>,
    },

    /// Show commit history
    #[command(long_about = "Show commit history

USAGE:
    oxenvcs-cli log [--limit <N>]

DESCRIPTION:
    Displays the commit history for the repository, showing commit IDs, authors,
    timestamps, and messages. Audio metadata (BPM, key, etc.) is displayed if
    present in the commit message.

EXAMPLES:
    # Show all commits
    oxenvcs-cli log

    # Show only the last 5 commits
    oxenvcs-cli log --limit 5

    # Show the last 10 commits
    oxenvcs-cli log -l 10")]
    Log {
        #[arg(short, long, help = "Maximum number of commits to display")]
        limit: Option<usize>,
    },

    /// Restore project to a previous commit
    #[command(long_about = "Restore project to a previous commit

USAGE:
    oxenvcs-cli restore <COMMIT_ID>

DESCRIPTION:
    Restores the project to the state at the specified commit. This checks out
    the files from that commit, allowing you to return to a previous version.

    WARNING: Make sure to commit any current changes before restoring, or they
    will be lost.

    You can find commit IDs using the 'log' command.

EXAMPLES:
    # Find commit IDs
    oxenvcs-cli log --limit 5

    # Restore to a specific commit
    oxenvcs-cli restore abc123def

    # Restore to a commit (full hash)
    oxenvcs-cli restore abc123def456789012345678901234567890")]
    Restore {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID to restore to (from 'log' command)")]
        commit_id: String,
    },

    /// Show repository status
    #[command(long_about = "Show repository status

USAGE:
    oxenvcs-cli status

DESCRIPTION:
    Displays the current state of the working directory and staging area:
      • Staged files (ready to commit)
      • Modified files (changed but not staged)
      • Untracked files (new files not yet added)

    This is similar to 'git status' and helps you see what changes are pending.

EXAMPLES:
    # Check repository status
    oxenvcs-cli status")]
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Enable verbose logging if requested
    logger::set_verbose(cli.verbose);

    match cli.command {
        Commands::Init { path, logic } => {
            vlog!("Starting initialization for path: {}", path.display());
            vlog!("Logic Pro mode: {}", logic);

            if logic {
                vlog!("Initializing Logic Pro project repository...");
                let repo = OxenRepository::init_for_logic_project(&path).await?;
                success!("Successfully initialized Logic Pro project repository");
            } else {
                vlog!("Initializing generic Oxen repository...");
                let repo = OxenRepository::init(&path).await?;
                success!("Successfully initialized Oxen repository at: {}", path.display());
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
