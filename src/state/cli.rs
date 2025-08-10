use crate::cli::{StateAction, WorkerAction};
use crate::state::backend::{BackendConfig, SerializationFormat};
use crate::state::manager::{StateManager, StateManagerConfig};
use crate::state::types::{PipelineState, PipelineStatus};
use anyhow::Result;
use chrono::Utc;
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

/// Handle state management CLI commands
pub async fn handle_state_command(action: StateAction) -> Result<()> {
    let config = StateManagerConfig {
        backend: BackendConfig::File {
            base_path: PathBuf::from(".oxiflow/state"),
            format: SerializationFormat::Json,
            atomic_writes: true,
            lock_timeout_ms: 30000,
        },
        ..Default::default()
    };
    let state_manager = StateManager::new(config).await?;

    match action {
        StateAction::Show {
            pipeline,
            json,
            yaml,
            verbose,
        } => show_state(&state_manager, &pipeline, json, yaml, verbose).await,

        StateAction::List {
            active,
            failed,
            completed,
            json,
            verbose,
        } => list_states(&state_manager, active, failed, completed, json, verbose).await,

        StateAction::Cleanup {
            stale,
            older_than_days,
            dry_run,
            force,
        } => cleanup_states(&state_manager, stale, older_than_days, dry_run, force).await,

        StateAction::Export {
            pipeline,
            output,
            format,
        } => export_state(&state_manager, &pipeline, &output, &format).await,

        StateAction::Import {
            pipeline,
            input,
            force,
        } => import_state(&state_manager, &pipeline, &input, force).await,
    }
}

/// Handle worker management CLI commands
pub async fn handle_worker_command(action: WorkerAction) -> Result<()> {
    let config = StateManagerConfig {
        backend: BackendConfig::File {
            base_path: PathBuf::from(".oxiflow/state"),
            format: SerializationFormat::Json,
            atomic_writes: true,
            lock_timeout_ms: 30000,
        },
        ..Default::default()
    };
    let state_manager = StateManager::new(config).await?;

    match action {
        WorkerAction::List {
            pipeline,
            json,
            verbose,
        } => list_workers(&state_manager, pipeline.as_deref(), json, verbose).await,

        WorkerAction::Stop { worker_id, force } => {
            stop_worker(&state_manager, &worker_id, force).await
        }
    }
}

/// Show the state of a specific pipeline
async fn show_state(
    state_manager: &StateManager,
    pipeline: &str,
    json: bool,
    yaml: bool,
    verbose: bool,
) -> Result<()> {
    match state_manager.load_state(pipeline).await {
        Ok(state) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&state)?);
            } else if yaml {
                println!("{}", serde_yaml::to_string(&state)?);
            } else {
                print_state_human(&state, verbose);
            }
        }
        Err(_) => {
            println!("❌ No state found for pipeline: {}", pipeline);
            std::process::exit(1);
        }
    }
    Ok(())
}

/// List all pipeline states with optional filtering
async fn list_states(
    state_manager: &StateManager,
    active: bool,
    failed: bool,
    completed: bool,
    json: bool,
    verbose: bool,
) -> Result<()> {
    let pipeline_ids = state_manager.list_pipelines().await?;
    let mut states = Vec::new();

    for pipeline_id in pipeline_ids {
        if let Ok(state) = state_manager.load_state(&pipeline_id).await {
            // Apply filters
            let include = if active || failed || completed {
                match &state.status {
                    PipelineStatus::Running { .. } => active,
                    PipelineStatus::Failed { .. } => failed,
                    PipelineStatus::Completed { .. } => completed,
                    PipelineStatus::Paused { .. } => active,
                    PipelineStatus::Pending => active,
                }
            } else {
                true // No filter, include all
            };

            if include {
                states.push(state);
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&states)?);
    } else {
        print_states_table(&states, verbose);
    }

    Ok(())
}

/// Clean up old or stale pipeline states
async fn cleanup_states(
    state_manager: &StateManager,
    stale: bool,
    older_than_days: Option<u32>,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let pipeline_ids = state_manager.list_pipelines().await?;
    let mut to_clean = Vec::new();

    for pipeline_id in pipeline_ids {
        if let Ok(state) = state_manager.load_state(&pipeline_id).await {
            let mut should_clean = false;

            if stale {
                // Check if state is stale (no recent heartbeat and not active)
                let stale_threshold = Utc::now() - chrono::Duration::minutes(30);
                if state.last_heartbeat < stale_threshold
                    && !matches!(state.status, PipelineStatus::Running { .. })
                {
                    should_clean = true;
                }
            }

            if let Some(days) = older_than_days {
                let threshold = Utc::now() - chrono::Duration::days(days as i64);
                if state.metadata.created_at < threshold {
                    should_clean = true;
                }
            }

            if should_clean {
                to_clean.push(state);
            }
        }
    }

    if to_clean.is_empty() {
        println!("✅ No states to clean up");
        return Ok(());
    }

    println!("🧹 Found {} states to clean up:", to_clean.len());
    for state in &to_clean {
        print_state_summary(&state);
    }

    if dry_run {
        println!("\n🔍 Dry run - no states were actually removed");
        return Ok(());
    }

    if !force {
        print!("\n❓ Are you sure you want to delete these states? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ Cleanup cancelled");
            return Ok(());
        }
    }

    for state in &to_clean {
        match state_manager.delete_state(&state.pipeline_id).await {
            Ok(_) => println!("✅ Cleaned up state for: {}", state.pipeline_id),
            Err(e) => println!("❌ Failed to clean up {}: {}", state.pipeline_id, e),
        }
    }

    println!("🎉 Cleanup completed");
    Ok(())
}

/// Export a pipeline state to a file
async fn export_state(
    state_manager: &StateManager,
    pipeline: &str,
    output: &str,
    format: &str,
) -> Result<()> {
    let state = state_manager.load_state(pipeline).await?;

    let content = match format.to_lowercase().as_str() {
        "json" => serde_json::to_string_pretty(&state)?,
        "yaml" => serde_yaml::to_string(&state)?,
        _ => {
            anyhow::bail!("Unsupported format: {}. Use 'json' or 'yaml'", format);
        }
    };

    fs::write(output, content)?;
    println!("✅ Exported state for {} to {}", pipeline, output);
    Ok(())
}

/// Import a pipeline state from a file
async fn import_state(
    state_manager: &StateManager,
    pipeline: &str,
    input: &str,
    force: bool,
) -> Result<()> {
    if !Path::new(input).exists() {
        anyhow::bail!("Input file does not exist: {}", input);
    }

    // Check if state already exists
    if !force && state_manager.load_state(pipeline).await.is_ok() {
        anyhow::bail!(
            "State already exists for pipeline: {}. Use --force to overwrite",
            pipeline
        );
    }

    let content = fs::read_to_string(input)?;

    // Try to parse as JSON first, then YAML
    let state: PipelineState = serde_json::from_str(&content)
        .or_else(|_| serde_yaml::from_str(&content))
        .map_err(|e| anyhow::anyhow!("Failed to parse state file: {}", e))?;

    // Ensure the pipeline ID matches
    if state.pipeline_id != pipeline {
        anyhow::bail!(
            "Pipeline ID mismatch: expected '{}', found '{}'",
            pipeline,
            state.pipeline_id
        );
    }

    state_manager.save_state(&state).await?;
    println!("✅ Imported state for {} from {}", pipeline, input);
    Ok(())
}

/// List all active workers
async fn list_workers(
    state_manager: &StateManager,
    pipeline_filter: Option<&str>,
    json: bool,
    verbose: bool,
) -> Result<()> {
    let pipeline_ids = state_manager.list_pipelines().await?;
    let mut workers = Vec::new();

    for pipeline_id in pipeline_ids {
        // Skip if pipeline filter doesn't match
        if let Some(filter) = pipeline_filter {
            if !pipeline_id.contains(filter) {
                continue;
            }
        }

        if let Ok(state) = state_manager.load_state(&pipeline_id).await {
            if let Some(worker_id) = &state.worker_id {
                // Check if worker is still active (recent heartbeat)
                let active_threshold = Utc::now() - chrono::Duration::minutes(5);
                let is_active = state.last_heartbeat > active_threshold;

                workers.push(serde_json::json!({
                    "worker_id": worker_id,
                    "pipeline_id": pipeline_id,
                    "status": format!("{:?}", state.status),
                    "last_heartbeat": state.last_heartbeat,
                    "active": is_active,
                    "current_step": state.current_step,
                }));
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&workers)?);
    } else {
        print_workers_table(&workers, verbose);
    }

    Ok(())
}

/// Stop a specific worker
async fn stop_worker(state_manager: &StateManager, worker_id: &str, force: bool) -> Result<()> {
    // Find the pipeline with this worker
    let pipeline_ids = state_manager.list_pipelines().await?;
    let mut found = false;

    for pipeline_id in pipeline_ids {
        if let Ok(state) = state_manager.load_state(&pipeline_id).await {
            if let Some(state_worker_id) = &state.worker_id {
                if state_worker_id == worker_id {
                    found = true;

                    if !force {
                        print!(
                            "❓ Are you sure you want to stop worker {}? (y/N): ",
                            worker_id
                        );
                        use std::io::{self, Write};
                        io::stdout().flush()?;

                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;

                        if !input.trim().to_lowercase().starts_with('y') {
                            println!("❌ Stop cancelled");
                            return Ok(());
                        }
                    }

                    // Update state to paused
                    state_manager
                        .update_state_locked(&pipeline_id, |state| {
                            state.status = PipelineStatus::Paused {
                                paused_at: Utc::now(),
                            };
                            state.metadata.updated_at = Utc::now();
                        })
                        .await?;

                    println!(
                        "✅ Worker {} stopped for pipeline {}",
                        worker_id, pipeline_id
                    );
                    break;
                }
            }
        }
    }

    if !found {
        println!("❌ Worker not found: {}", worker_id);
        std::process::exit(1);
    }

    Ok(())
}

/// Print a pipeline state in human-readable format
fn print_state_human(state: &PipelineState, verbose: bool) {
    println!("📊 Pipeline State: {}", state.pipeline_id);
    println!("🔄 Run ID: {}", state.run_id);
    println!("📈 Status: {:?}", state.status);
    println!(
        "🕒 Started: {}",
        state.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "💓 Last Heartbeat: {}",
        state.last_heartbeat.format("%Y-%m-%d %H:%M:%S UTC")
    );

    if verbose {
        println!("📝 Current Step: {}", state.current_step);
        println!("✅ Records Processed: {}", state.records_processed);
        println!("❌ Records Failed: {}", state.records_failed);
        println!("💾 Data Size: {} bytes", state.data_size_processed);

        if !state.step_states.is_empty() {
            println!("\n🔧 Step States:");
            for (step_id, step_state) in &state.step_states {
                println!("  • {}: {:?}", step_id, step_state.status);
            }
        }

        if !state.errors.is_empty() {
            println!("\n❌ Errors ({}):", state.errors.len());
            for error in &state.errors {
                println!("  • {:?}: {}", error.error_type, error.message);
            }
        }
    }
}

/// Print a summary line for a state
fn print_state_summary(state: &PipelineState) {
    println!(
        "  {} | {} | {:?} | {}",
        state.pipeline_id,
        state.run_id[..8].to_string() + "...",
        state.status,
        state.started_at.format("%Y-%m-%d %H:%M")
    );
}

/// Print states in a table format
fn print_states_table(states: &[PipelineState], verbose: bool) {
    if states.is_empty() {
        println!("📭 No pipeline states found");
        return;
    }

    println!("📊 Pipeline States ({}):", states.len());
    println!("{:-<80}", "");

    if verbose {
        println!(
            "{:<20} {:<12} {:<15} {:<20} {:<10}",
            "Pipeline", "Run ID", "Status", "Started", "Progress"
        );
        println!("{:-<80}", "");

        for state in states {
            let status_str = match &state.status {
                PipelineStatus::Running { .. } => "Running",
                PipelineStatus::Completed { .. } => "Completed",
                PipelineStatus::Failed { .. } => "Failed",
                PipelineStatus::Paused { .. } => "Paused",
                PipelineStatus::Pending => "Pending",
            };

            println!(
                "{:<20} {:<12} {:<15} {:<20} {}/{}",
                state.pipeline_id,
                &state.run_id[..8],
                status_str,
                state.started_at.format("%m-%d %H:%M"),
                state.records_processed,
                state.records_processed + state.records_failed
            );
        }
    } else {
        println!("{:<20} {:<15} {:<20}", "Pipeline", "Status", "Started");
        println!("{:-<60}", "");

        for state in states {
            let status_str = match &state.status {
                PipelineStatus::Running { .. } => "🟢 Running",
                PipelineStatus::Completed { .. } => "✅ Completed",
                PipelineStatus::Failed { .. } => "❌ Failed",
                PipelineStatus::Paused { .. } => "⏸️  Paused",
                PipelineStatus::Pending => "⏳ Pending",
            };

            println!(
                "{:<20} {:<15} {}",
                state.pipeline_id,
                status_str,
                state.started_at.format("%Y-%m-%d %H:%M")
            );
        }
    }
}

/// Print workers in a table format
fn print_workers_table(workers: &[serde_json::Value], verbose: bool) {
    if workers.is_empty() {
        println!("👥 No active workers found");
        return;
    }

    println!("👥 Active Workers ({}):", workers.len());
    println!("{:-<80}", "");

    if verbose {
        println!(
            "{:<15} {:<20} {:<15} {:<20} {:<10}",
            "Worker ID", "Pipeline", "Status", "Last Heartbeat", "Step"
        );
        println!("{:-<80}", "");

        for worker in workers {
            println!(
                "{:<15} {:<20} {:<15} {:<20} {}",
                worker["worker_id"].as_str().unwrap_or(""),
                worker["pipeline_id"].as_str().unwrap_or(""),
                worker["status"].as_str().unwrap_or(""),
                worker["last_heartbeat"]
                    .as_str()
                    .map(|s| &s[11..19])
                    .unwrap_or(""),
                worker["current_step"].as_str().unwrap_or("")
            );
        }
    } else {
        println!("{:<15} {:<20} {:<10}", "Worker ID", "Pipeline", "Active");
        println!("{:-<50}", "");

        for worker in workers {
            let active_icon = if worker["active"].as_bool().unwrap_or(false) {
                "🟢"
            } else {
                "🔴"
            };

            println!(
                "{:<15} {:<20} {}",
                worker["worker_id"].as_str().unwrap_or(""),
                worker["pipeline_id"].as_str().unwrap_or(""),
                active_icon
            );
        }
    }
}
