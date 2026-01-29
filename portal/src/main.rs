use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// Data Structures
// ============================================================================

/// Main oh-my-opencode configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhMyOpenCodeConfig {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_auth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagents: Option<HashMap<String, serde_json::Value>>,
    pub agents: HashMap<String, AgentConfig>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Per-agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Provider config format - matches oh-my-opencode structure
/// Example: { "agents": { "sisyphus": { "model": "github-copilot/claude-opus-4.5" } } }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    #[serde(default)]
    pub agents: HashMap<String, AgentModelConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelConfig {
    pub model: String,
}

// ============================================================================
// Default Model Mappings
// ============================================================================

struct ModelMapping {
    base: &'static str,
    copilot: &'static str,
    openrouter_provider: &'static str,
    openrouter_model: &'static str,
}

const MODEL_MAPPINGS: &[ModelMapping] = &[
    ModelMapping {
        base: "claude-opus-4.5",
        copilot: "claude-opus-4.5",
        openrouter_provider: "anthropic",
        openrouter_model: "claude-opus-4.5",
    },
    ModelMapping {
        base: "claude-sonnet-4.5",
        copilot: "claude-sonnet-4.5",
        openrouter_provider: "anthropic",
        openrouter_model: "claude-sonnet-4.5",
    },
    ModelMapping {
        base: "claude-sonnet-4",
        copilot: "claude-sonnet-4",
        openrouter_provider: "anthropic",
        openrouter_model: "claude-sonnet-4",
    },
    ModelMapping {
        base: "gpt-5.2",
        copilot: "gpt-5.2",
        openrouter_provider: "openai",
        openrouter_model: "gpt-5.2",
    },
    ModelMapping {
        base: "gpt-4.1",
        copilot: "gpt-4.1",
        openrouter_provider: "openai",
        openrouter_model: "gpt-4.1",
    },
    ModelMapping {
        base: "o3",
        copilot: "o3",
        openrouter_provider: "openai",
        openrouter_model: "o3",
    },
    ModelMapping {
        base: "o4-mini",
        copilot: "o4-mini",
        openrouter_provider: "openai",
        openrouter_model: "o4-mini",
    },
    ModelMapping {
        base: "gemini-3-flash",
        copilot: "gemini-3-flash",
        openrouter_provider: "google",
        openrouter_model: "gemini-3-flash-preview",
    },
    ModelMapping {
        base: "gemini-3-pro",
        copilot: "gemini-3-pro",
        openrouter_provider: "google",
        openrouter_model: "gemini-3-pro-preview",
    },
];

// ============================================================================
// CLI
// ============================================================================

#[derive(Parser)]
#[command(name = "portal")]
#[command(about = "Quick switching between oh-my-opencode model providers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to oh-my-opencode.json config file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Show what would change without writing
    #[arg(long, global = true)]
    dry_run: bool,

    /// Force create backup even if one exists
    #[arg(long, global = true)]
    backup: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Switch to a provider (copilot, openrouter, or custom)
    Switch {
        /// Provider name (e.g., copilot, openrouter, work-openrouter)
        provider: String,
    },
    /// Show current provider and model configuration
    Status,
    /// List available providers from ~/.config/portal/
    List,
    /// Revert to a backup
    Revert {
        /// Path to backup file (defaults to latest)
        backup_path: Option<PathBuf>,
    },
}

// ============================================================================
// Config Operations
// ============================================================================

fn get_portal_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".config")
        })
        .join("portal")
}

fn get_config_path(custom: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = custom {
        return Ok(path);
    }

    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("Could not determine home directory")
                .join(".config")
        });
    Ok(config_dir.join("opencode").join("oh-my-opencode.json"))
}

fn get_provider_config_path(provider: &str) -> PathBuf {
    get_portal_dir().join(format!("{}.json", provider))
}

fn read_config(path: &PathBuf) -> Result<OhMyOpenCodeConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))
}

fn read_provider_config(provider: &str) -> Result<Option<ProviderConfig>> {
    let path = get_provider_config_path(provider);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read provider config: {}", path.display()))?;
    let config: ProviderConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse provider config: {}", path.display()))?;
    Ok(Some(config))
}

fn has_existing_backup(config_path: &PathBuf) -> bool {
    let parent = config_path.parent().unwrap_or(config_path);
    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("oh-my-opencode.json.bak.") {
                return true;
            }
        }
    }
    false
}

fn find_latest_backup(config_path: &PathBuf) -> Option<PathBuf> {
    let parent = config_path.parent()?;
    let mut backups: Vec<_> = fs::read_dir(parent)
        .ok()?
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("oh-my-opencode.json.bak.")
        })
        .collect();

    backups.sort_by_key(|e| e.file_name());
    backups.last().map(|e| e.path())
}

fn write_config(path: &PathBuf, config: &OhMyOpenCodeConfig, force_backup: bool) -> Result<()> {
    let should_backup = force_backup || !has_existing_backup(path);

    if should_backup && path.exists() {
        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S-%3fZ");
        let backup_path = path.with_extension(format!("json.bak.{}", timestamp));
        fs::copy(path, &backup_path)
            .with_context(|| format!("Failed to create backup at: {}", backup_path.display()))?;
        eprintln!("Backup created: {}", backup_path.display());
    }

    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, format!("{}\n", content))
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;
    Ok(())
}

// ============================================================================
// Provider Switching Logic
// ============================================================================

fn detect_current_provider(config: &OhMyOpenCodeConfig) -> Option<String> {
    let first_model = config.agents.values().next()?.model.as_str();
    let parts: Vec<&str> = first_model.split('/').collect();
    parts.first().map(|s| s.to_string())
}

fn extract_base_model(model: &str) -> String {
    let parts: Vec<&str> = model.split('/').collect();
    match parts.as_slice() {
        [_, model] => model.to_string(),
        [_, _, model] => model.to_string(),
        [_, _, _, rest @ ..] => rest.join("/"),
        _ => model.to_string(),
    }
}

fn find_mapping(base_model: &str) -> Option<&'static ModelMapping> {
    MODEL_MAPPINGS.iter().find(|m| {
        m.base == base_model || m.copilot == base_model || m.openrouter_model == base_model
    })
}

fn transform_to_builtin_provider(base_model: &str, provider: &str) -> Option<String> {
    let mapping = find_mapping(base_model)?;

    match provider {
        "copilot" | "github-copilot" => Some(format!("github-copilot/{}", mapping.copilot)),
        "openrouter" => Some(format!(
            "openrouter/{}/{}",
            mapping.openrouter_provider, mapping.openrouter_model
        )),
        _ => None,
    }
}

fn infer_openrouter_model(base_model: &str) -> String {
    let provider = if base_model.starts_with("claude") {
        "anthropic"
    } else if base_model.starts_with("gpt")
        || base_model.starts_with("o1")
        || base_model.starts_with("o3")
        || base_model.starts_with("o4")
    {
        "openai"
    } else if base_model.starts_with("gemini") {
        "google"
    } else {
        "unknown"
    };
    format!("openrouter/{}/{}", provider, base_model)
}

fn switch_to_provider(
    config: &mut OhMyOpenCodeConfig,
    provider: &str,
    provider_config: Option<&ProviderConfig>,
) -> Result<()> {
    for (agent_name, agent_config) in config.agents.iter_mut() {
        // Check if provider config has explicit mapping for this agent
        if let Some(pc) = provider_config {
            if let Some(agent_override) = pc.agents.get(agent_name) {
                agent_config.model = agent_override.model.clone();
                continue;
            }
        }

        // Fall back to built-in transformations
        let base = extract_base_model(&agent_config.model);
        let canonical_base = find_mapping(&base).map(|m| m.base).unwrap_or(&base);

        if let Some(new_model) = transform_to_builtin_provider(canonical_base, provider) {
            agent_config.model = new_model;
        } else {
            // Custom provider without explicit config - best effort
            match provider {
                p if p.contains("openrouter") => {
                    agent_config.model = infer_openrouter_model(canonical_base);
                }
                p if p.contains("copilot") => {
                    agent_config.model = format!("github-copilot/{}", canonical_base);
                }
                _ => {
                    eprintln!(
                        "Warning: No mapping for agent '{}' with provider '{}', keeping current model",
                        agent_name, provider
                    );
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Commands
// ============================================================================

fn cmd_switch(cli: &Cli, provider: &str) -> Result<()> {
    let config_path = get_config_path(cli.config.clone())?;

    if !config_path.exists() {
        return Err(anyhow!(
            "Config file not found: {}\n\nMake sure oh-my-opencode is configured.",
            config_path.display()
        ));
    }

    let mut config = read_config(&config_path)?;
    let provider_config = read_provider_config(provider)?;

    switch_to_provider(&mut config, provider, provider_config.as_ref())?;

    if cli.dry_run {
        println!("Dry run - would switch to '{}':", provider);
        println!();
        for (name, agent) in &config.agents {
            println!("  {}: {}", name, agent.model);
        }
    } else {
        write_config(&config_path, &config, cli.backup)?;
        println!("Switched to '{}' provider.", provider);
    }

    Ok(())
}

fn cmd_status(cli: &Cli) -> Result<()> {
    let config_path = get_config_path(cli.config.clone())?;

    if !config_path.exists() {
        return Err(anyhow!(
            "Config file not found: {}\n\nMake sure oh-my-opencode is configured.",
            config_path.display()
        ));
    }

    let config = read_config(&config_path)?;
    let current = detect_current_provider(&config);

    println!("Config: {}", config_path.display());
    println!();
    println!(
        "Provider: {}",
        current.unwrap_or_else(|| "Unknown".to_string())
    );
    println!();
    println!("Agents:");
    for (name, agent) in &config.agents {
        println!("  {}: {}", name, agent.model);
    }

    Ok(())
}

fn cmd_list() -> Result<()> {
    println!("Built-in providers:");
    println!("  copilot     - GitHub Copilot (github-copilot/model)");
    println!("  openrouter  - OpenRouter (openrouter/provider/model)");
    println!();

    let portal_dir = get_portal_dir();
    if portal_dir.exists() {
        let custom_providers: Vec<_> = fs::read_dir(&portal_dir)?
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .map(|e| e.path().file_stem().unwrap().to_string_lossy().to_string())
            .collect();

        if !custom_providers.is_empty() {
            println!("Custom providers (from {}):", portal_dir.display());
            for p in custom_providers {
                println!("  {}", p);
            }
            println!();
        }
    }

    println!("Usage: portal switch <provider>");
    Ok(())
}

fn cmd_revert(cli: &Cli, backup_path: Option<PathBuf>) -> Result<()> {
    let config_path = get_config_path(cli.config.clone())?;

    let backup = match backup_path {
        Some(p) => {
            if !p.exists() {
                return Err(anyhow!("Backup file not found: {}", p.display()));
            }
            p
        }
        None => find_latest_backup(&config_path).ok_or_else(|| anyhow!("No backup files found"))?,
    };

    if cli.dry_run {
        println!("Dry run - would revert to: {}", backup.display());
        return Ok(());
    }

    fs::copy(&backup, &config_path)
        .with_context(|| format!("Failed to restore from backup: {}", backup.display()))?;

    println!("Reverted to: {}", backup.display());
    Ok(())
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Switch { provider } => cmd_switch(&cli, provider),
        Commands::Status => cmd_status(&cli),
        Commands::List => cmd_list(),
        Commands::Revert { backup_path } => cmd_revert(&cli, backup_path.clone()),
    }
}
