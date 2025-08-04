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
}
