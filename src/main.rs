use oxide_flow::{cli::Cli, Oxi, types::{OxiConfig, OxiData}};
use oxide_flow::oxis::json::oxi::{ParseJson, FormatJson};
use oxide_flow::oxis::csv::oxi::{ParseCsv, FormatCsv};
use oxide_flow::oxis::flatten::flatten::Flatten;
use oxide_flow::oxis::read_stdin::ReadStdIn;
use oxide_flow::oxis::write_stdout::WriteStdOut;
use oxide_flow::config::{Config, PipelineContext};
use clap::Parser;
use serde_yaml;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Enable verbose output if requested
    if cli.verbose {
        println!("Verbose mode enabled");
    }

    // Check if config path is provided
    if let Some(config_path) = &cli.config {
        println!("Using config from: {}", config_path);
        match run_pipeline_from_config(config_path, &cli).await {
            Ok(_) => println!("Pipeline execution completed successfully!"),
            Err(e) => {
                eprintln!("Pipeline execution failed: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // No config provided, show usage
        println!("No configuration file provided.");
        println!("Usage: oxide_flow --config <path_to_config.yaml>");
        println!();
        println!("Example config file structure:");
        println!("```yaml");
        println!("version: \"1.0\"");
        println!("pipelines:");
        println!("  default:");
        println!("    name: \"JSON to CSV Pipeline\"");
        println!("    description: \"Convert JSON data to CSV format\"");
        println!("    oxis:");
        println!("      - oxi: \"read_file\"");
        println!("        config:");
        println!("          path: \"input.json\"");
        println!("      - oxi: \"parse_json\"");
        println!("      - oxi: \"format_csv\"");
        println!("      - oxi: \"write_file\"");
        println!("        config:");
        println!("          path: \"output.csv\"");
        println!("```");
        std::process::exit(1);
    }
}

/// Run a pipeline from a configuration file
async fn run_pipeline_from_config(config_path: &str, cli: &Cli) -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load(config_path)?;
    
    // Determine which pipeline to run
    let pipeline_name = if !cli.oxis.is_empty() {
        // Use the first argument as pipeline name if provided
        cli.oxis[0].as_str()
    } else {
        "default"
    };
    
    let pipeline = config.get_pipeline(pipeline_name)
        .ok_or_else(|| anyhow::anyhow!("Pipeline '{}' not found in config", pipeline_name))?;
    
    if cli.verbose {
        println!("Running pipeline: {}", pipeline.name);
        if let Some(desc) = &pipeline.description {
            println!("Description: {}", desc);
        }
        println!("Steps: {}", pipeline.oxis.len());
    }
    
    // Create pipeline context for managing step outputs and metadata
    let mut context = PipelineContext::new();
    let mut current_data = OxiData::Empty;
    
    // Execute each step in the pipeline
    for (step_index, step) in pipeline.oxis.iter().enumerate() {
        if cli.verbose {
            println!("Step {}: Executing oxi '{}'", step_index + 1, step.oxi);
        }
        
        // Convert step config to OxiConfig
        let oxi_config = OxiConfig::from_yaml(step.config.clone());
        
        // Resolve any dynamic references in the config
        let resolved_config = context.resolve_config_references(&oxi_config)?;
        
        // Execute the appropriate Oxi
        let step_result = match step.oxi.as_str() {
            "read_stdin" => {
                let oxi = ReadStdIn;
                oxi.process(current_data, &resolved_config).await?
            },
            "write_stdout" => {
                let oxi = WriteStdOut;
                oxi.process(current_data, &resolved_config).await?
            },
            "parse_json" => {
                let oxi = ParseJson;
                oxi.process(current_data, &resolved_config).await?
            },
            "format_json" => {
                let oxi = FormatJson;
                oxi.process(current_data, &resolved_config).await?
            },
            "parse_csv" => {
                let oxi = ParseCsv;
                oxi.process(current_data, &resolved_config).await?
            },
            "format_csv" => {
                let oxi = FormatCsv;
                oxi.process(current_data, &resolved_config).await?
            },
            "flatten" => {
                let oxi = Flatten;
                oxi.process(current_data, &resolved_config).await?
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown oxi type: {}", step.oxi));
            }
        };
        
        // Store step output in context for future reference
        let default_step_id = format!("step_{}", step_index);
        let step_id = step.alias.as_ref()
            .unwrap_or(&default_step_id);
        
        // Convert OxiData to serde_yaml::Value for storage
        let output_value = match &step_result {
            OxiData::Text(text) => serde_yaml::Value::String(text.clone()),
            OxiData::Json(value) => {
                // Convert JSON to YAML for storage in context
                let yaml_str = serde_yaml::to_string(&value)?;
                serde_yaml::from_str(&yaml_str)?
            },
            OxiData::Binary(data) => {
                use base64::{Engine as _, engine::general_purpose};
                serde_yaml::Value::String(general_purpose::STANDARD.encode(data))
            },
            OxiData::Empty => serde_yaml::Value::Null,
        };
        
        context.add_step_output(step_id, output_value);
        
        // Update current data for next step
        current_data = step_result;
        
        if cli.verbose {
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
    }
    
    Ok(())
}