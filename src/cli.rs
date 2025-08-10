use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new Oxide Flow project
    Init {
        /// Project name (optional, will prompt if not provided)
        #[arg(short, long)]
        name: Option<String>,

        /// Target directory (defaults to current directory)
        #[arg(short, long)]
        directory: Option<String>,
    },
    /// Run a pipeline from a YAML file
    Run {
        /// Pipeline name to run (finds in configured pipeline directory)
        #[arg(default_value = "pipeline")]
        pipeline: String,

        /// Path to configuration file
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Manage pipelines (list, add, test, info)
    Pipeline {
        #[command(subcommand)]
        action: PipelineAction,
    },
    /// Manage pipeline state (show, list, cleanup, export/import)
    State {
        #[command(subcommand)]
        action: StateAction,
    },
    /// Manage workers (list, stop)
    Worker {
        #[command(subcommand)]
        action: WorkerAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PipelineAction {
    /// List available pipelines
    List {
        /// Filter by tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Filter by keyword in name/description
        #[arg(short, long)]
        filter: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Create a new pipeline from a template
    Add {
        /// Name of the new pipeline
        name: String,

        /// Template to use (default: "basic")
        #[arg(short, long, default_value = "basic")]
        template: String,

        /// Pipeline description
        #[arg(short, long)]
        description: Option<String>,

        /// Pipeline author
        #[arg(short, long)]
        author: Option<String>,
    },
    /// Test/validate a pipeline
    Test {
        /// Name of the pipeline to test
        name: String,

        /// Validate only, don't execute
        #[arg(long)]
        dry_run: bool,

        /// Show detailed validation information
        #[arg(short, long)]
        verbose: bool,

        /// Attempt to fix common issues
        #[arg(long)]
        fix: bool,

        /// Validate against schemas only
        #[arg(long)]
        schema: bool,
    },
    /// Show detailed pipeline information
    Info {
        /// Name of the pipeline
        name: String,

        /// Show configuration schema for all steps
        #[arg(long)]
        schema: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Output in YAML format
        #[arg(long)]
        yaml: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum StateAction {
    /// View current pipeline state
    Show {
        /// Pipeline name
        pipeline: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Output in YAML format
        #[arg(long)]
        yaml: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// List all pipeline states
    List {
        /// Filter by active pipelines only
        #[arg(long)]
        active: bool,

        /// Filter by failed pipelines only
        #[arg(long)]
        failed: bool,

        /// Filter by completed pipelines only
        #[arg(long)]
        completed: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Clean up old/stale states
    Cleanup {
        /// Remove only stale states (no active workers)
        #[arg(long)]
        stale: bool,

        /// Remove states older than this many days
        #[arg(long)]
        older_than_days: Option<u32>,

        /// Dry run - show what would be cleaned up
        #[arg(long)]
        dry_run: bool,

        /// Force cleanup without confirmation
        #[arg(short, long)]
        force: bool,
    },
    /// Export state to JSON/YAML file
    Export {
        /// Pipeline name
        pipeline: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Export format (json, yaml)
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Import state from JSON/YAML file
    Import {
        /// Pipeline name
        pipeline: String,

        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Force import even if state exists
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkerAction {
    /// List active workers
    List {
        /// Filter by pipeline name
        #[arg(short, long)]
        pipeline: Option<String>,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Stop a specific worker
    Stop {
        /// Worker ID to stop
        worker_id: String,

        /// Force stop without confirmation
        #[arg(short, long)]
        force: bool,
    },
}
