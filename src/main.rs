use clap::Parser;
use oxide_flow::{
    cli::{Cli, Commands},
    config_resolver::ConfigResolver,
    pipeline::Pipeline,
    project::{self, ProjectConfig},
    types::OxiData,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Enable verbose output if requested
    if cli.verbose {
        println!("Verbose mode enabled");
    }

    // Handle commands
    match cli.command {
        Commands::Init { name, directory } => match project::init_project(name, directory) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to initialize project: {e}");
                std::process::exit(1);
            }
        },
        Commands::Run {
            pipeline,
            config: _,
        } => match run_pipeline_by_name(&pipeline).await {
            Ok(_) => println!("✅ Pipeline execution completed successfully!"),
            Err(e) => {
                eprintln!("❌ Pipeline execution failed: {e}");
                std::process::exit(1);
            }
        },
    }
}

/// Run a pipeline by name using project configuration for discovery
async fn run_pipeline_by_name(pipeline_name: &str) -> anyhow::Result<()> {
    // Load project configuration
    let project_config = ProjectConfig::load()
        .map_err(|e| anyhow::anyhow!("Failed to load project configuration: {}", e))?;

    // Find the pipeline file
    let pipeline_path = project_config.find_pipeline(pipeline_name)?;

    println!(
        "🔍 Running pipeline '{}' from: {}",
        pipeline_name,
        pipeline_path.display()
    );

    // Run the pipeline
    run_pipeline_from_yaml(pipeline_path.to_str().unwrap()).await
}

/// Run a pipeline from a YAML file with enhanced error handling
async fn run_pipeline_from_yaml(pipeline_path: &str) -> anyhow::Result<()> {
    // Load pipeline
    let pipeline = Pipeline::load_from_file(pipeline_path)?;

    println!("Running pipeline: {}", pipeline.name());
    if let Some(desc) = pipeline.description() {
        println!("Description: {desc}");
    }
    println!("Steps: {}", pipeline.step_count());

    // Create configuration resolver for dynamic references
    let resolver = ConfigResolver::default();

    // Use enhanced execution with error handling
    let result = pipeline
        .execute_with_retries(OxiData::Empty, &resolver)
        .await;

    if result.success {
        if let Some(final_data) = result.final_data {
            // Display final result
            match &final_data {
                OxiData::Text(text) => {
                    let preview = if text.len() > 200 {
                        format!("{}... ({} characters)", &text[..200], text.len())
                    } else {
                        text.clone()
                    };
                    println!("Final Result: Text data - {preview}");
                }
                OxiData::Json(_) => {
                    println!("Final Result: JSON data");
                }
                OxiData::Binary(data) => {
                    println!("Final Result: Binary data ({} bytes)", data.len());
                }
                OxiData::Empty => {
                    println!("Final Result: Empty data");
                }
            }
        }
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Pipeline execution failed with {} failed steps",
            result.steps_failed
        ))
    }
}
