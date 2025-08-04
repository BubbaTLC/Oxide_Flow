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
