//! Contains all LSPs supported officially

/// All officially supported LSPs
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
#[clap(rename_all = "kebab-case")]
pub enum LanguageServer {
    /// https://github.com/bash-lsp/bash-language-server
    BashLanguageServer,
}

pub struct LanguageServerDefaultSettings<'a> {
    /// Default container for lsp
    pub image: &'a str,

    /// Command to run in the container (first element is the entrypoint)
    pub cmd: Vec<&'a str>,
}

pub fn get_language_server_defaults(
    language_server: LanguageServer,
) -> LanguageServerDefaultSettings<'static> {
    match language_server {
        LanguageServer::BashLanguageServer => LanguageServerDefaultSettings {
            image: "docker.io/lspcontainers/bash-language-server",
            cmd: vec!["bash-language-server", "start"],
        },
    }
}
