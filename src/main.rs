use oxide_flow::{cli::{Cli, Commands}, project, pipeline::Pipeline, Oxi, types::OxiData};
use oxide_flow::oxis::json::oxi::{ParseJson, FormatJson};
use oxide_flow::oxis::csv::oxi::{ParseCsv, FormatCsv};
use oxide_flow::oxis::file::oxi::{ReadFile, WriteFile};
use oxide_flow::oxis::flatten::flatten::Flatten;
use oxide_flow::oxis::read_stdin::ReadStdIn;
use oxide_flow::oxis::write_stdout::WriteStdOut;
use clap::Parser;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Enable verbose output if requested
    if cli.verbose {
        println!("Verbose mode enabled");
    }

    // Handle commands
    match cli.command {
        Commands::Init { name, directory } => {
            match project::init_project(name, directory) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Failed to initialize project: {}", e);
                    std::process::exit(1);
                }
            }
        },
        Commands::Run { pipeline, config: _ } => {
            println!("Running pipeline from: {}", pipeline);
            match run_pipeline_from_yaml(&pipeline).await {
                Ok(_) => println!("✅ Pipeline execution completed successfully!"),
                Err(e) => {
                    eprintln!("❌ Pipeline execution failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Run a pipeline from a YAML file
async fn run_pipeline_from_yaml(pipeline_path: &str) -> anyhow::Result<()> {
    // Load pipeline
    let pipeline = Pipeline::load_from_file(pipeline_path)?;
    
    println!("Running pipeline: {}", pipeline.name());
    if let Some(desc) = pipeline.description() {
        println!("Description: {}", desc);
    }
    println!("Steps: {}", pipeline.step_count());
    
    // Track step outputs for potential referencing
    let mut step_outputs: HashMap<String, OxiData> = HashMap::new();
    let mut current_data = OxiData::Empty;
    
    // Execute each step in the pipeline
    for (step_index, step) in pipeline.pipeline.iter().enumerate() {
        println!("Step {}: Executing oxi '{}'", step_index + 1, step.name);
        
        // Convert step config to OxiConfig
        let oxi_config = step.to_oxi_config();
        
                // Execute the appropriate Oxi
        let step_result = match step.name.as_str() {
            "read_file" => {
                let oxi = ReadFile;
                oxi.process(current_data, &oxi_config).await?
            },
            "write_file" => {
                let oxi = WriteFile;
                oxi.process(current_data, &oxi_config).await?
            },
            "read_stdin" => {
                let oxi = ReadStdIn;
                oxi.process(current_data, &oxi_config).await?
            },
            "write_stdout" => {
                let oxi = WriteStdOut;
                oxi.process(current_data, &oxi_config).await?
            },
            "parse_json" => {
                let oxi = ParseJson;
                oxi.process(current_data, &oxi_config).await?
            },
            "format_json" => {
                let oxi = FormatJson;
                oxi.process(current_data, &oxi_config).await?
            },
            "parse_csv" => {
                let oxi = ParseCsv;
                oxi.process(current_data, &oxi_config).await?
            },
            "format_csv" => {
                let oxi = FormatCsv;
                oxi.process(current_data, &oxi_config).await?
            },
            "flatten" => {
                let oxi = Flatten;
                oxi.process(current_data, &oxi_config).await?
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown oxi type: {}", step.name));
            }
        };
        
        // Store step output for potential future reference
        let step_id = step.get_id();
        step_outputs.insert(step_id.to_string(), step_result.clone());
        
        // Update current data for next step
        current_data = step_result;
        
        match &current_data {
            OxiData::Text(text) => {
                let preview = if text.len() > 200 {
                    format!("{}... ({} characters)", &text[..200], text.len())
                } else {
                    text.clone()
                };
                println!("  Result: Text data - {}", preview);
            },
            OxiData::Json(_) => {
                println!("  Result: JSON data");
            },
            OxiData::Binary(data) => {
                println!("  Result: Binary data ({} bytes)", data.len());
            },
            OxiData::Empty => {
                println!("  Result: Empty data");
            },
        }
    }
    
    Ok(())
}