//! Cli interface

use crate::engine::Engine;
use crate::language_servers::LanguageServer;
use clap::Parser;

fn get_pwd() -> String {
    // NOTE this probably should not happen, if it does just panic cause its hopeless
    std::env::current_dir()
        .expect("Unable to get CWD")
        .to_string_lossy()
        .to_string()
}

// Tool to streamline containerized LSP tooling for any editor by providing path manipulation
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
#[clap(disable_help_flag = true)]
pub struct Cli {
    /// Show debugging information
    #[arg(short, long)]
    pub debug: bool,

    /// Engine preference, to force certain engine just omit others
    #[arg(short, long, value_name = "ENGINES", use_value_delimiter = true, default_values = ["podman", "docker"])]
    pub engine: Vec<Engine>,

    /// Override host path, defaults to CWD
    #[arg(short, long, default_value_t = get_pwd())]
    pub host_path: String,

    /// Override container path, defaults to image's `WorkingDir`
    #[arg(short, long)]
    pub container_path: Option<String>,

    /// Start container for specific language server
    pub language_server: Option<LanguageServer>,

    /// Override the container image (required if language_server is not set)
    #[arg(long, required_unless_present = "language_server")]
    pub image: Option<String>,

    /// Override the command in the container including the entrypoint (required if language_server is not set)
    #[arg(last = true, required_unless_present = "language_server")]
    pub cmd: Vec<String>,

    /// List all LSPs and their configuration (mostly useful for debugging)
    #[arg(long, exclusive = true)]
    pub list: bool,

    /// Print this help screen
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
