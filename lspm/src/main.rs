use anyhow::{Context, Result, anyhow};
use clap::Parser;
use std::{
    io::{BufRead, BufReader, Write},
    process::Command,
};

mod cli;

struct Options {
    /// Pattern to replace in host->container communication (from, to)
    replace_pattern: (String, String),
}

impl Options {
    pub fn new(host_path: &str, container_path: &str) -> Self {
        // TODO these do not support encoded paths so no unicode?
        // NOTE i am using the simplest and dumbest way, just finding uri beginning with `"file://`
        Self {
            replace_pattern: (
                format!(r#""file://{}"#, host_path),
                format!(r#""file://{}"#, container_path),
            ),
        }
    }
}

fn transform(line: &mut String, options: &Options, flip: bool) {
    if flip {
        *line = line.replace(&options.replace_pattern.1, &options.replace_pattern.0);
    } else {
        *line = line.replace(&options.replace_pattern.0, &options.replace_pattern.1);
    }
}

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    let options = Options::new(
        &args.host_path,
        args.container_path
            .as_ref()
            .expect("container_path getting not implemented"),
    );

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    // TODO start container
    let mut cmd_handle = Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| anyhow!("Error spawning command"))?;

    let mut child_stdin = cmd_handle.stdin.take().unwrap();
    let mut child_stdout = BufReader::new(cmd_handle.stdout.take().unwrap());

    std::thread::scope(|s| {
        // main stdin -> child stdin
        s.spawn(|| {
            let mut main_stdin = stdin.lock();
            let mut line = String::with_capacity(256);

            loop {
                match main_stdin.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        transform(&mut line, &options, false);
                        if args.debug {
                            eprintln!("h > c {:?}", line);
                        }
                        child_stdin.write(line.as_bytes()).unwrap();
                        line.clear();
                    }
                    Err(err) => panic!("Error reading a line from stdin: {}", err),
                }
            }
        });

        // child stdout -> main stdout
        s.spawn(|| {
            let mut main_stdout = stdout.lock();
            let mut line = String::with_capacity(256);

            loop {
                match child_stdout.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        transform(&mut line, &options, true);
                        if args.debug {
                            eprintln!("h < c {:?}", line);
                        }
                        main_stdout.write(line.as_bytes()).unwrap();
                        line.clear();
                    }
                    Err(err) => panic!("Error reading a line from stdin: {}", err),
                }
            }
        });

        cmd_handle.wait().expect("Error waiting for child");
    });

    Ok(())
}
