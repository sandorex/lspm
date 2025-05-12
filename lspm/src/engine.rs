//! Everything to do with container engines like podman, docker

use anyhow::{Context, Result, anyhow};
use std::process::Command;

#[derive(clap::ValueEnum, Clone, PartialEq, Eq, Debug)]
pub enum Engine {
    Podman,
    Docker,
}

impl Engine {
    fn command(&self) -> Command {
        match self {
            Self::Podman => Command::new("podman"),
            Self::Docker => Command::new("docker"),
        }
    }

    /// Checks if engine is installed by checking if its in path
    pub fn is_available(&self) -> bool {
        let output = self.command().args(["--version"]).output();

        match output {
            Ok(x) => {
                if x.status.success() {
                    true
                } else {
                    panic!("Failed to run engine --version, this is a bug")
                }
            }
            // if its missing then it an err
            Err(_) => false,
        }
    }

    /// Pull image interactively
    pub fn pull_image(&self, image: &str) -> Result<()> {
        let output = self
            .command()
            .args(["pull", image])
            .output()
            .with_context(|| anyhow!("Failed to pull image {image:?}"))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Error trying to pull image {image:?} (exit code: {:?})",
                output.status.code()
            ))
        }
    }

    /// Check if image exists locally
    pub fn image_exists(&self, image: &str) -> Result<bool> {
        let output = self
            .command()
            .args(["image", "exists", image])
            .output()
            .with_context(|| anyhow!("Failed to check for image {image:?}"))?;

        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            x => Err(anyhow!(
                "Error checking for image {image:?} (code: {:?})",
                x
            )),
        }
    }

    pub fn start_container(
        &self,
        _name: &str,
        image: &str,
        workdir: &str,
        cmd: Vec<&str>,
    ) -> Result<std::process::Child> {
        let entrypoint = cmd.first().unwrap();

        self.command()
            .args([
                "run",
                "-i",   // enable stdin/stdout
                "--rm", // rm automatically
                // "--security-opt label=disable", // TODO idk if its necessary?
                // "--userns keep-id", // this as well?
                // TODO just use random name for now
                // "--name", // name the container
                // name,
                format!("--volume=.:{workdir}").as_str(),
                "--workdir", // force workdir to known value just in case
                workdir,
                "--entrypoint",
                entrypoint,
                image,
            ])
            // append all the args but entrypoint
            .args(&cmd[1..])
            .spawn()
            .with_context(|| anyhow!("Failed to spawn {image:?} with cmd {cmd:?}"))
    }

    pub fn get_image_workdir(&self, image: &str) -> Result<Option<String>> {
        let output = self
            .command()
            .args([
                "image",
                "inspect",
                "--format={{ .Config.WorkingDir }}",
                image,
            ])
            .output()
            .with_context(|| anyhow!("Failed to inspect image {image:?}"))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stdout = stdout.trim();

            if stdout.is_empty() {
                Ok(None)
            } else {
                Ok(Some(stdout.to_string()))
            }
        } else {
            Err(anyhow!(
                "Error getting workdir from image {image:?} (code: {:?})",
                output.status.code()
            ))
        }
    }
}
