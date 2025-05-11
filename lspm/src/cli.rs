//! Cli interface

use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Copy, Debug)]
#[clap(rename_all = "kebab-case")]
pub enum LanguageServer {
    BashLanguageServer,
}

fn get_pwd() -> String {
    std::env::current_dir()
        .expect("Unable to get CWD")
        .to_string_lossy()
        .to_string()
}

// Tool to streamline containerized LSP tooling for any editor by providing path manipulation
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long)]
    pub debug: bool,

    /// Override host path, defaults to CWD
    #[arg(long, default_value_t = get_pwd())]
    pub host_path: String,

    /// Override container path, defaults to image's `WorkingDir`
    #[arg(long)]
    pub container_path: Option<String>,

    /// Start container for specific language server
    pub language_server: Option<LanguageServer>,

    /// Start a container from custom image
    #[arg(long, requires = "cmd", conflicts_with = "language_server")]
    pub image: Option<String>,

    /// Command to run in the containr, including the entrypoint
    #[arg(last = true, conflicts_with = "language_server")]
    pub cmd: Vec<String>,
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
