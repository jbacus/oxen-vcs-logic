use clap::{Parser, Subcommand};
use colored::Colorize;
use oxenvcs_cli::{logger, progress, success, vlog, CommitMetadata, OxenRepository};
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

        #[arg(
            long,
            help = "Initialize for Logic Pro project (auto-detect and configure)"
        )]
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

        #[arg(
            long,
            help = "Tags for categorization (comma-separated, e.g., 'mixing,draft')"
        )]
        tags: Option<String>,
    },

    /// Show commit history
    #[command(long_about = "Show commit history

USAGE:
    oxenvcs-cli log [OPTIONS]

DESCRIPTION:
    Displays the commit history for the repository, showing commit IDs, authors,
    timestamps, and messages. Audio metadata (BPM, key, etc.) is displayed if
    present in the commit message.

    Filter commits by metadata, tags, or date range to find specific versions.

EXAMPLES:
    # Show all commits
    oxenvcs-cli log

    # Show only the last 5 commits
    oxenvcs-cli log --limit 5

    # Show commits with specific BPM
    oxenvcs-cli log --bpm 128

    # Show commits with specific tag
    oxenvcs-cli log --tag mixing

    # Show commits since a date
    oxenvcs-cli log --since \"2025-01-01\"

    # Combine filters
    oxenvcs-cli log --bpm 120 --tag vocals --limit 10")]
    Log {
        #[arg(short, long, help = "Maximum number of commits to display")]
        limit: Option<usize>,

        #[arg(long, help = "Filter by BPM (e.g., 120, 128)")]
        bpm: Option<f32>,

        #[arg(long, help = "Filter by tag (e.g., 'mixing', 'vocals')")]
        tag: Option<String>,

        #[arg(long, help = "Filter by key signature (e.g., 'C Major')")]
        key: Option<String>,

        #[arg(long, help = "Show commits since date (YYYY-MM-DD)")]
        since: Option<String>,
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
        #[arg(
            value_name = "COMMIT_ID",
            help = "Commit ID to restore to (from 'log' command)"
        )]
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

    /// Show detailed information about a commit
    #[command(long_about = "Show detailed information about a commit

USAGE:
    oxenvcs-cli show <COMMIT_ID>

DESCRIPTION:
    Displays comprehensive information about a specific commit, including:
      • Full commit message
      • Audio metadata (BPM, sample rate, key signature, tags)
      • Author and timestamp
      • Files changed
      • Commit statistics

EXAMPLES:
    # Show details of a recent commit
    oxenvcs-cli show abc123f

    # Show details with full hash
    oxenvcs-cli show abc123def456789012345678901234567890")]
    Show {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID to show details for")]
        commit_id: String,
    },

    /// Show changes between commits or working directory
    #[command(long_about = "Show changes between commits or working directory

USAGE:
    oxenvcs-cli diff [COMMIT_ID]

DESCRIPTION:
    Shows file-level changes in the repository:
      • Without arguments: shows changes in working directory vs last commit
      • With commit ID: shows changes between that commit and working directory
      • With two IDs: shows changes between two commits

    Displays:
      • Modified files with size changes
      • Added files
      • Deleted files
      • Total size impact

EXAMPLES:
    # Show uncommitted changes
    oxenvcs-cli diff

    # Show changes since specific commit
    oxenvcs-cli diff abc123f

    # Compare two commits (future enhancement)
    # oxenvcs-cli diff abc123f def456a")]
    Diff {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID to compare against (optional)")]
        commit_id: Option<String>,
    },

    /// Compare metadata between two Logic Pro project versions
    #[command(name = "metadata-diff")]
    #[command(long_about = "Compare metadata between two Logic Pro project versions

USAGE:
    oxenvcs-cli metadata-diff <PROJECT_A> <PROJECT_B>
    oxenvcs-cli metadata-diff <PROJECT_A> <PROJECT_B> --output json

DESCRIPTION:
    Analyzes and compares the metadata of two Logic Pro project versions,
    generating a detailed report of what changed. Detects:
      • Global changes (tempo, sample rate, key signature)
      • Track additions, removals, and modifications
      • EQ changes (frequency, gain, Q factor)
      • Compressor changes (threshold, ratio, attack, release)
      • Volume and pan adjustments
      • Automation curve changes
      • Plugin parameter changes

    The output is a human-readable report showing exactly what changed
    between versions, making it easy to understand project evolution.

EXAMPLES:
    # Compare two project versions
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx

    # Output as JSON for programmatic use
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx --output json

    # Compare with colored output
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx --color

    # Verbose mode with technical details
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx --verbose")]
    MetadataDiff {
        #[arg(value_name = "PROJECT_A", help = "First Logic Pro project (.logicx)")]
        project_a: PathBuf,

        #[arg(value_name = "PROJECT_B", help = "Second Logic Pro project (.logicx)")]
        project_b: PathBuf,

        #[arg(
            long,
            value_name = "FORMAT",
            default_value = "text",
            help = "Output format (text or json)"
        )]
        output: String,

        #[arg(long, help = "Use colored output (default: auto-detect)")]
        color: bool,

        #[arg(long, short, help = "Include technical details in output")]
        verbose: bool,
    },
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
                // Multi-step initialization with progress feedback
                let pb = progress::spinner("Validating Logic Pro project structure...");

                vlog!("Initializing Logic Pro project repository...");
                let _repo = OxenRepository::init_for_logic_project(&path).await?;

                progress::finish_success(&pb, "Logic Pro project repository initialized");
                println!();
                progress::success(&format!("Repository created at: {}", path.display()));
                progress::info("Next steps:");
                println!("  1. cd {}", path.display());
                println!("  2. oxenvcs-cli add --all");
                println!("  3. oxenvcs-cli commit -m \"Initial commit\"");
            } else {
                let pb = progress::spinner(&format!("Initializing Oxen repository at {}...", path.display()));

                vlog!("Initializing generic Oxen repository...");
                let _repo = OxenRepository::init(&path).await?;

                progress::finish_success(
                    &pb,
                    &format!("Oxen repository initialized at: {}", path.display())
                );
            }
            Ok(())
        }

        Commands::Add { paths, all } => {
            let repo = OxenRepository::new(".");

            if all {
                let pb = progress::spinner("Staging all changes...");
                repo.stage_all().await?;
                progress::finish_success(&pb, "All changes staged");
                println!();
                progress::info("Next step: oxenvcs-cli commit -m \"Your message\"");
            } else {
                if paths.is_empty() {
                    progress::error("Please provide paths to stage or use --all");
                    std::process::exit(1);
                }
                let pb = progress::spinner(&format!("Staging {} file(s)...", paths.len()));
                repo.stage_changes(paths).await?;
                progress::finish_success(&pb, "Files staged");
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
            let pb = progress::spinner("Preparing commit...");
            let repo = OxenRepository::new(".");

            let mut metadata = CommitMetadata::new(message.clone());

            if let Some(bpm) = bpm {
                metadata = metadata.with_bpm(bpm);
            }

            if let Some(sr) = sample_rate {
                metadata = metadata.with_sample_rate(sr);
            }

            if let Some(ref key_val) = key {
                metadata = metadata.with_key_signature(key_val.clone());
            }

            if let Some(ref tags_str) = tags {
                for tag in tags_str.split(',') {
                    metadata = metadata.with_tag(tag.trim());
                }
            }

            pb.set_message("Creating commit...");
            let commit_id = repo.create_commit(metadata).await?;

            progress::finish_success(&pb, &format!("Commit created: {}", commit_id));

            // Show commit details
            println!();
            progress::info("Commit Details:");
            println!("  Message: {}", message);
            if let Some(ref bpm_val) = bpm {
                println!("  BPM: {}", bpm_val);
            }
            if let Some(ref sr_val) = sample_rate {
                println!("  Sample Rate: {} Hz", sr_val);
            }
            if let Some(ref key_val) = key {
                println!("  Key: {}", key_val);
            }
            if let Some(ref tags_val) = tags {
                println!("  Tags: {}", tags_val);
            }

            Ok(())
        }

        Commands::Log { limit, bpm, tag, key, since } => {
            let repo = OxenRepository::new(".");

            let mut commits = repo.get_history(None).await?;

            if commits.is_empty() {
                println!();
                progress::info("No commits yet");
                println!();
                progress::info("Create your first commit:");
                println!("  oxenvcs-cli add --all");
                println!("  oxenvcs-cli commit -m \"Initial commit\"");
                return Ok(());
            }

            // Apply filters
            let total_before_filter = commits.len();
            let mut filters_applied = vec![];

            if let Some(bpm_filter) = bpm {
                commits.retain(|c| {
                    c.message.lines().any(|line| {
                        line.contains("BPM:") && line.contains(&bpm_filter.to_string())
                    })
                });
                filters_applied.push(format!("BPM = {}", bpm_filter));
            }

            if let Some(tag_filter) = &tag {
                commits.retain(|c| {
                    c.message.lines().any(|line| {
                        line.contains("Tags:") && line.to_lowercase().contains(&tag_filter.to_lowercase())
                    })
                });
                filters_applied.push(format!("tag = {}", tag_filter));
            }

            if let Some(key_filter) = &key {
                commits.retain(|c| {
                    c.message.lines().any(|line| {
                        line.contains("Key:") && line.to_lowercase().contains(&key_filter.to_lowercase())
                    })
                });
                filters_applied.push(format!("key = {}", key_filter));
            }

            if let Some(_since_filter) = &since {
                // TODO: Implement date filtering when commit timestamps are available
                progress::warning("Date filtering not yet implemented (commit timestamps needed)");
            }

            // Apply limit after filtering
            if let Some(lim) = limit {
                commits.truncate(lim);
            }

            // Show results
            println!();
            println!("┌─ Commit History ────────────────────────────────────────┐");
            if !filters_applied.is_empty() {
                println!("│ Filters: {}                                              │", filters_applied.join(", "));
                println!("│ Found {} of {} commit(s)                                 │", commits.len(), total_before_filter);
            } else if let Some(lim) = limit {
                println!("│ Showing last {} of {} commit(s)                          │", commits.len(), total_before_filter);
            } else {
                println!("│ Showing all {} commit(s)                                 │", commits.len());
            }
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            if commits.is_empty() {
                progress::info("No commits match the specified filters");
                return Ok(());
            }

            for (idx, commit) in commits.iter().enumerate() {
                let short_id = &commit.id[..7.min(commit.id.len())];

                // Visual timeline with bullets
                println!("{} {} - {}", "●".cyan(), short_id.bright_yellow(), "now".bright_black());

                // Commit message (indented)
                let lines: Vec<&str> = commit.message.lines().collect();
                if let Some(first_line) = lines.first() {
                    println!("  │ {}", first_line.bright_white());
                }

                // Additional metadata if present in message
                for line in lines.iter().skip(1) {
                    if !line.trim().is_empty() {
                        if line.contains("BPM:") || line.contains("Sample Rate:") || line.contains("Key:") || line.contains("Tags:") {
                            println!("  │ {}", line.trim().bright_black());
                        } else {
                            println!("  │ {}", line.trim());
                        }
                    }
                }

                // Add spacing between commits (except last one)
                if idx < commits.len() - 1 {
                    println!("  │");
                }
            }

            println!();
            progress::info(&format!("Showing {} commit(s)", commits.len()));

            Ok(())
        }

        Commands::Restore { commit_id } => {
            let pb = progress::spinner(&format!("Restoring to commit {}...", &commit_id[..7.min(commit_id.len())]));
            let repo = OxenRepository::new(".");

            pb.set_message("Checking out files...");
            repo.restore(&commit_id).await?;

            progress::finish_success(&pb, &format!("Restored to commit {}", &commit_id[..7.min(commit_id.len())]));
            println!();
            progress::warning("Your working directory has been updated to match this commit");
            progress::info("To create a new commit from here, use:");
            println!("  oxenvcs-cli add --all");
            println!("  oxenvcs-cli commit -m \"Your message\"");

            Ok(())
        }

        Commands::Status => {
            let repo = OxenRepository::new(".");

            let status = repo.status().await?;

            // Header
            println!();
            println!("┌─ Repository Status ─────────────────────────────────────┐");
            println!("│                                                          │");

            let total_changes = status.staged.len() + status.modified.len() + status.untracked.len();

            if total_changes == 0 {
                println!("│  ✓ Working directory clean                              │");
            } else {
                println!("│  Changes: {} staged, {} modified, {} untracked",
                    status.staged.len().to_string().green(),
                    status.modified.len().to_string().yellow(),
                    status.untracked.len().to_string().cyan(),
                );
            }

            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            if !status.staged.is_empty() {
                println!("{} Staged files ({}):", "●".green(), status.staged.len());
                for path in &status.staged {
                    println!("  {} {}", "+".green(), path.display());
                }
                println!();
            }

            if !status.modified.is_empty() {
                println!("{} Modified files ({}):", "◆".yellow(), status.modified.len());
                for path in &status.modified {
                    println!("  {} {}", "M".yellow(), path.display());
                }
                println!();
            }

            if !status.untracked.is_empty() {
                println!("{} Untracked files ({}):", "?".cyan(), status.untracked.len());
                for path in &status.untracked {
                    println!("  {} {}", "?".cyan(), path.display());
                }
                println!();
            }

            // Next steps suggestion
            if total_changes > 0 {
                if status.staged.is_empty() && (!status.modified.is_empty() || !status.untracked.is_empty()) {
                    progress::info("Next step: oxenvcs-cli add --all");
                } else if !status.staged.is_empty() {
                    progress::info("Next step: oxenvcs-cli commit -m \"Your message\"");
                }
            }

            Ok(())
        }

        Commands::Show { commit_id } => {
            let repo = OxenRepository::new(".");

            // Get all commits to find the one we want
            let commits = repo.get_history(None).await?;

            let commit = commits.iter().find(|c| {
                c.id.starts_with(&commit_id) || c.id == commit_id
            });

            if let Some(commit) = commit {
                println!();
                println!("┌─ Commit Details ────────────────────────────────────────┐");
                println!("│                                                          │");
                println!("│  Commit: {}                                      │", commit.id.bright_yellow());
                println!("│                                                          │");
                println!("└──────────────────────────────────────────────────────────┘");
                println!();

                // Parse commit message and metadata
                let lines: Vec<&str> = commit.message.lines().collect();

                // First line is the commit message
                if let Some(first_line) = lines.first() {
                    println!("{}", "Message:".bright_white().bold());
                    println!("  {}", first_line);
                    println!();
                }

                // Extract metadata
                let mut metadata_found = false;
                for line in lines.iter().skip(1) {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if trimmed.starts_with("BPM:") || trimmed.starts_with("Sample Rate:") ||
                           trimmed.starts_with("Key:") || trimmed.starts_with("Tags:") {
                            if !metadata_found {
                                println!("{}", "Metadata:".bright_white().bold());
                                metadata_found = true;
                            }
                            println!("  {}", trimmed.bright_black());
                        } else {
                            // Additional commit message content
                            println!("  {}", trimmed);
                        }
                    }
                }

                println!();
                progress::info(&format!("Use 'oxenvcs-cli restore {}' to restore to this commit", &commit.id[..7.min(commit.id.len())]));
            } else {
                progress::error(&format!("Commit not found: {}", commit_id));
                std::process::exit(1);
            }

            Ok(())
        }

        Commands::Diff { commit_id } => {
            let repo = OxenRepository::new(".");

            println!();
            if let Some(cid) = &commit_id {
                println!("┌─ Changes Since Commit {} ─────────────────────┐", &cid[..7.min(cid.len())].bright_yellow());
            } else {
                println!("┌─ Uncommitted Changes ───────────────────────────────────┐");
            }
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            // Get current status
            let status = repo.status().await?;

            let total_files = status.staged.len() + status.modified.len() + status.untracked.len();

            if total_files == 0 {
                progress::info("No changes in working directory");
                return Ok(());
            }

            // Modified files
            if !status.modified.is_empty() {
                println!("{} Modified files ({}):", "◆".yellow(), status.modified.len());
                for path in &status.modified {
                    // Try to get file size info
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let size = metadata.len();
                        println!("  {} {} {}", "~".yellow(), path.display(), format!("({} bytes)", size).bright_black());
                    } else {
                        println!("  {} {}", "~".yellow(), path.display());
                    }
                }
                println!();
            }

            // Untracked (new) files
            if !status.untracked.is_empty() {
                println!("{} Added files ({}):", "◆".green(), status.untracked.len());
                for path in &status.untracked {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let size = metadata.len();
                        let size_mb = size as f64 / 1_048_576.0;
                        if size_mb >= 1.0 {
                            println!("  {} {} {}", "+".green(), path.display(), format!("({:.1} MB)", size_mb).bright_black());
                        } else {
                            println!("  {} {} {}", "+".green(), path.display(), format!("({} bytes)", size).bright_black());
                        }
                    } else {
                        println!("  {} {}", "+".green(), path.display());
                    }
                }
                println!();
            }

            // Summary
            let total_modified = status.modified.len();
            let total_added = status.untracked.len();

            progress::info(&format!(
                "Total changes: {} modified, {} added",
                total_modified.to_string().yellow(),
                total_added.to_string().green()
            ));

            if !status.staged.is_empty() {
                println!();
                progress::info("Some changes are already staged. Use 'oxenvcs-cli status' for details");
            }

            Ok(())
        }

        Commands::MetadataDiff {
            project_a,
            project_b,
            output,
            color,
            verbose: verbose_flag,
        } => {
            use oxenvcs_cli::{LogicParser, MetadataDiffer};

            vlog!("Parsing project A: {}", project_a.display());

            // Validate paths
            if !LogicParser::is_valid_project(&project_a) {
                anyhow::bail!("Invalid Logic Pro project: {}", project_a.display());
            }

            if !LogicParser::is_valid_project(&project_b) {
                anyhow::bail!("Invalid Logic Pro project: {}", project_b.display());
            }

            // Parse both projects
            let data_a = LogicParser::parse(&project_a)
                .map_err(|e| anyhow::anyhow!("Failed to parse project A: {}", e))?;

            vlog!("Parsing project B: {}", project_b.display());
            let data_b = LogicParser::parse(&project_b)
                .map_err(|e| anyhow::anyhow!("Failed to parse project B: {}", e))?;

            // Generate diff
            vlog!("Computing metadata diff");
            let diff = MetadataDiffer::compare(&data_a, &data_b);

            // Output result
            match output.as_str() {
                "json" => {
                    let json = MetadataDiffer::to_json(&diff)?;
                    println!("{}", json);
                }
                "text" | _ => {
                    // Determine color usage
                    let use_color = if color {
                        true
                    } else {
                        // Auto-detect TTY
                        atty::is(atty::Stream::Stdout)
                    };

                    let report = MetadataDiffer::generate_report_with_options(
                        &diff,
                        use_color,
                        verbose_flag || cli.verbose,
                    );
                    println!("{}", report);
                }
            }

            success!("Metadata diff completed");
            Ok(())
        }
    }
}
