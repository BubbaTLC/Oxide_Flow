use clap::Parser;
use oxide_flow::{
    cli::{Cli, Commands, PipelineAction},
    config_resolver::ConfigResolver,
    pipeline::Pipeline,
    pipeline_manager::PipelineManager,
    project::{self, ProjectConfig},
    state::cli::{handle_state_command, handle_worker_command},
    types::{Data, OxiData},
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
        Commands::Pipeline { action } => match handle_pipeline_command(action).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("❌ Pipeline command failed: {e}");
                std::process::exit(1);
            }
        },
        Commands::State { action } => match handle_state_command(action).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("❌ State command failed: {e}");
                std::process::exit(1);
            }
        },
        Commands::Worker { action } => match handle_worker_command(action).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("❌ Worker command failed: {e}");
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

    // Run the pipeline with state tracking
    run_pipeline_from_yaml_with_state(pipeline_path.to_str().unwrap(), &project_config).await
}

/// Run a pipeline from a YAML file with state tracking support
async fn run_pipeline_from_yaml_with_state(
    pipeline_path: &str,
    project_config: &ProjectConfig,
) -> anyhow::Result<()> {
    // Load pipeline
    let pipeline = Pipeline::load_from_file(pipeline_path)?;

    println!("Running pipeline: {}", pipeline.name());
    if let Some(desc) = pipeline.description() {
        println!("Description: {desc}");
    }
    println!("Steps: {}", pipeline.step_count());

    // Create configuration resolver for dynamic references
    let resolver = ConfigResolver::default();

    // Create state manager if configured
    let state_manager = if project_config.state_manager.is_some() {
        match oxide_flow::state::manager::StateManager::new(
            project_config.create_state_manager_config(),
        )
        .await
        {
            Ok(manager) => {
                println!("📊 State tracking enabled");
                Some(manager)
            }
            Err(e) => {
                println!("⚠️  Failed to initialize state tracking: {e}");
                None
            }
        }
    } else {
        None
    };

    // Use enhanced execution with optional state tracking
    let result = pipeline
        .execute_with_state_tracking(OxiData::empty(), &resolver, state_manager)
        .await;

    if result.success {
        if let Some(final_data) = result.final_data {
            // Display final result
            match &final_data.data {
                Data::Text(text) => {
                    let preview = if text.len() > 200 {
                        format!("{}... ({} characters)", &text[..200], text.len())
                    } else {
                        text.clone()
                    };
                    println!("Final Result: Text data - {preview}");
                }
                Data::Json(_) => {
                    println!("Final Result: JSON data");
                }
                Data::Binary(data) => {
                    println!("Final Result: Binary data ({} bytes)", data.len());
                }
                Data::Empty => {
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

/// Handle pipeline management commands
async fn handle_pipeline_command(action: PipelineAction) -> anyhow::Result<()> {
    match action {
        PipelineAction::List {
            tags,
            filter,
            verbose,
        } => {
            let manager = PipelineManager::new()?;
            let mut pipelines = manager.discover_pipelines()?;

            // Apply tag filter if provided
            if let Some(tag_filter) = tags {
                pipelines = manager.filter_by_tags(&pipelines, &tag_filter);
            }

            // Apply keyword filter if provided
            if let Some(keyword_filter) = filter {
                pipelines = manager.filter_by_keyword(&pipelines, &keyword_filter);
            }

            // Display results
            let output = manager.format_pipeline_table(&pipelines, verbose);
            println!("{output}");

            Ok(())
        }
        PipelineAction::Add {
            name,
            template,
            description,
            author,
        } => {
            let manager = PipelineManager::new()?;

            // Create pipeline with provided parameters
            println!("📝 Creating new pipeline: {name}");
            println!("  Template: {template}");

            manager.create_pipeline(&name, &template, description.as_deref(), author.as_deref())?;
            println!("✅ Pipeline '{name}' created successfully!");

            Ok(())
        }
        PipelineAction::Test {
            name,
            dry_run,
            verbose,
            fix,
            schema,
        } => {
            let manager = PipelineManager::new()?;

            match manager.test_pipeline(&name, dry_run, verbose, fix, schema) {
                Ok(result) => {
                    let output = manager.format_validation_result(&result, verbose);
                    println!("{output}");

                    if !result.is_valid() {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Pipeline testing failed: {e}");
                    std::process::exit(1);
                }
            }

            Ok(())
        }
        PipelineAction::Info {
            name,
            schema,
            json,
            yaml,
        } => {
            // Use pipeline manager to find and display pipeline info
            let manager = PipelineManager::new()?;
            let pipelines = manager.discover_pipelines()?;

            // Find the pipeline by name (check both display name and filename)
            if let Some(pipeline) = pipelines.iter().find(|p| {
                p.name == name
                    || p.file_path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .map(|stem| stem == name)
                        .unwrap_or(false)
            }) {
                if json {
                    // Output as JSON
                    let json_output = serde_json::to_string_pretty(pipeline)?;
                    println!("{json_output}");
                } else if yaml {
                    // Output as YAML
                    let yaml_output = serde_yaml::to_string(pipeline)?;
                    println!("{yaml_output}");
                } else {
                    // Standard formatted output
                    println!("📋 Pipeline Information: {}\n", pipeline.name);

                    println!("📝 Metadata:");
                    if let Some(description) = &pipeline.description {
                        println!("   Description: {description}");
                    }
                    if let Some(version) = &pipeline.version {
                        println!("   Version: {version}");
                    }
                    if let Some(author) = &pipeline.author {
                        println!("   Author: {author}");
                    }
                    if let Some(tags) = &pipeline.tags {
                        println!("   Tags: {}", tags.join(", "));
                    }
                    if let Some(created) = &pipeline.created {
                        println!("   Created: {created}");
                    }
                    println!("   Location: {}", pipeline.file_path.display());

                    println!("\n⚙️  Configuration:");
                    if pipeline.step_names.is_empty() {
                        println!("   Steps: {} total", pipeline.step_count);
                    } else {
                        println!(
                            "   Steps: {} ({})",
                            pipeline.step_count,
                            pipeline.step_names.join(" → ")
                        );
                    }

                    if schema {
                        println!("\n🔧 Schema information will be implemented in Phase 4");
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Pipeline '{}' not found", name));
            }

            Ok(())
        }
    }
}
