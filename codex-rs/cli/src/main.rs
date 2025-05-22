use clap::Parser;
use codex_cli::LandlockCommand;
use codex_cli::create_sandbox_policy;
use codex_cli::proto;
use codex_cli::seatbelt;
use codex_cli::LandlockCommand;
use codex_cli::SeatbeltCommand;
use codex_core::mcp_config; // Added for MCP preset management
use codex_exec::Cli as ExecCli;
use codex_tui::Cli as TuiCli;
use std::io::{self, Write}; // Added for printing to stdout

use crate::proto::ProtoCli;

/// Codex CLI
///
/// If no subcommand is specified, options will be forwarded to the interactive CLI.
#[derive(Debug, Parser)]
#[clap(
    author,
    version,
    // If a sub‑command is given, ignore requirements of the default args.
    subcommand_negates_reqs = true
)]
struct MultitoolCli {
    #[clap(flatten)]
    interactive: TuiCli,

    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    /// Run Codex non-interactively.
    #[clap(visible_alias = "e")]
    Exec(ExecCli),

    /// Experimental: run Codex as an MCP server.
    Mcp,

    /// Run the Protocol stream via stdin/stdout
    #[clap(visible_alias = "p")]
    Proto(ProtoCli),

    /// Internal debugging commands.
    Debug(DebugArgs),

    /// Manage configuration settings.
    Config(ConfigCli),
}

#[derive(Debug, Parser)]
struct DebugArgs {
    #[command(subcommand)]
    cmd: DebugCommand,
}

#[derive(Debug, clap::Subcommand)]
enum DebugCommand {
    /// Run a command under Seatbelt (macOS only).
    Seatbelt(SeatbeltCommand),

    /// Run a command under Landlock+seccomp (Linux only).
    Landlock(LandlockCommand),
}

#[derive(Debug, Parser)]
struct ReplProto {}

#[derive(Debug, Parser)]
struct ConfigCli {
    #[clap(subcommand)]
    command: ConfigSubcommand,
}

#[derive(Debug, clap::Subcommand)]
enum ConfigSubcommand {
    /// Manage MCP server presets
    #[clap(subcommand, name = "mcp")]
    Mcp(McpPresetSubcommand),
}

#[derive(Debug, clap::Subcommand)]
enum McpPresetSubcommand {
    /// Add or update an MCP server preset. If a preset with the given label exists, it will be updated.
    Add {
        /// Unique label for the preset
        #[clap(long)]
        label: String,
        /// URL of the MCP server
        #[clap(long)]
        url: String,
    },
    /// Remove an MCP server preset
    Remove {
        /// Label of the preset to remove
        #[clap(long)]
        label: String,
    },
    /// Enable an MCP server preset
    Enable {
        /// Label of the preset to enable
        #[clap(long)]
        label: String,
    },
    /// Disable an MCP server preset
    Disable {
        /// Label of the preset to disable
        #[clap(long)]
        label: String,
    },
    /// List all MCP server presets
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = MultitoolCli::parse();

    match cli.subcommand {
        None => {
            codex_tui::run_main(cli.interactive)?;
        }
        Some(Subcommand::Exec(exec_cli)) => {
            codex_exec::run_main(exec_cli).await?;
        }
        Some(Subcommand::Mcp) => {
            codex_mcp_server::run_main().await?;
        }
        Some(Subcommand::Proto(proto_cli)) => {
            proto::run_main(proto_cli).await?;
        }
        Some(Subcommand::Debug(debug_args)) => match debug_args.cmd {
            DebugCommand::Seatbelt(SeatbeltCommand {
                command,
                sandbox,
                full_auto,
            }) => {
                let sandbox_policy = create_sandbox_policy(full_auto, sandbox);
                seatbelt::run_seatbelt(command, sandbox_policy).await?;
            }
            #[cfg(unix)]
            DebugCommand::Landlock(LandlockCommand {
                command,
                sandbox,
                full_auto,
            }) => {
                let sandbox_policy = create_sandbox_policy(full_auto, sandbox);
                codex_cli::landlock::run_landlock(command, sandbox_policy)?;
            }
            #[cfg(not(unix))]
            DebugCommand::Landlock(_) => {
                anyhow::bail!("Landlock is only supported on Linux.");
            }
        },
        Some(Subcommand::Config(config_cli)) => match config_cli.command {
            ConfigSubcommand::Mcp(mcp_command) => match mcp_command {
                McpPresetSubcommand::Add { label, url } => {
                    match mcp_config::add_mcp_server_preset(label.clone(), url) {
                        Ok(_) => println!("MCP preset '{}' added/updated successfully.", label),
                        Err(e) => eprintln!("Error adding/updating MCP preset: {}", e),
                    }
                }
                McpPresetSubcommand::Remove { label } => {
                    match mcp_config::remove_mcp_server_preset(label.clone()) {
                        Ok(_) => println!("MCP preset '{}' removed successfully.", label),
                        Err(e) => eprintln!("Error removing MCP preset: {}", e),
                    }
                }
                McpPresetSubcommand::Enable { label } => {
                    match mcp_config::enable_mcp_server_preset(label.clone(), true) {
                        Ok(_) => println!("MCP preset '{}' enabled successfully.", label),
                        Err(e) => eprintln!("Error enabling MCP preset: {}", e),
                    }
                }
                McpPresetSubcommand::Disable { label } => {
                    match mcp_config::enable_mcp_server_preset(label.clone(), false) {
                        Ok(_) => println!("MCP preset '{}' disabled successfully.", label),
                        Err(e) => eprintln!("Error disabling MCP preset: {}", e),
                    }
                }
                McpPresetSubcommand::List => {
                    match mcp_config::list_mcp_server_presets() {
                        Ok(presets) => {
                            if presets.is_empty() {
                                println!("No MCP server presets configured.");
                            } else {
                                // Determine column widths
                                let label_width = presets.iter().map(|p| p.label.len()).max().unwrap_or(5).max(5); // "Label"
                                let url_width = presets.iter().map(|p| p.url.len()).max().unwrap_or(3).max(3); // "URL"
                                let status_width = 8; // "Enabled" / "Disabled"

                                // Print header
                                println!(
                                    "{:<label_width$} | {:<url_width$} | {:<status_width$}",
                                    "Label", "URL", "Status",
                                    label_width = label_width,
                                    url_width = url_width,
                                    status_width = status_width
                                );
                                println!(
                                    "{:-<label_width$}-+-{:-<url_width$}-+-{:-<status_width$}",
                                    "", "", "",
                                    label_width = label_width,
                                    url_width = url_width,
                                    status_width = status_width
                                );

                                // Print presets
                                for preset in presets {
                                    println!(
                                        "{:<label_width$} | {:<url_width$} | {:<status_width$}",
                                        preset.label,
                                        preset.url,
                                        if preset.is_enabled { "Enabled" } else { "Disabled" },
                                        label_width = label_width,
                                        url_width = url_width,
                                        status_width = status_width
                                    );
                                }
                            }
                        }
                        Err(e) => eprintln!("Error listing MCP presets: {}", e),
                    }
                }
            },
        },
    }

    Ok(())
}
