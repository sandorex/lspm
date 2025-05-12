use anyhow::{Context, Result, anyhow};
use clap::Parser;
use language_servers::get_language_server_defaults;
use std::io::{BufRead, BufReader, Write};

mod cli;
mod engine;
mod language_servers;

// TODO remove options and just keep replace pattern
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
    let mut args = cli::Cli::parse();

    if args.list {
        // TODO
        todo!("listing not implemented yet");
    }

    // TODO search and check for each engine
    let engine = args.engine.first().unwrap();
    assert!(engine.is_available());

    // if language server is picked get the defaults and apply them
    if let Some(language_server) = args.language_server {
        let defaults = get_language_server_defaults(language_server);

        // only apply them if not override in cli
        if args.image.is_none() {
            args.image = Some(defaults.image.to_string())
        }

        if args.cmd.is_empty() {
            args.cmd.extend(defaults.cmd.iter().map(|x| x.to_string()));
        }
    }

    assert!(args.image.is_some());
    assert!(!args.cmd.is_empty());

    if args.container_path.is_none() {
        args.container_path = match engine.get_image_workdir(&args.image.as_ref().unwrap())? {
            Some(x) => Some(x.clone()),
            None => {
                return Err(anyhow!(
                    "Image {:?} does not have workdir defined and container_path is not set",
                    args.image.as_ref().unwrap()
                ));
            }
        };
    }

    assert!(args.container_path.is_some());
    assert!(
        engine
            .image_exists(args.image.as_ref().unwrap())
            .is_ok_and(|x| x == true)
    );

    let options = Options::new(&args.host_path, args.container_path.as_ref().unwrap());

    let mut child = engine.start_container(
        "",
        args.image.as_ref().unwrap(),
        args.container_path.as_ref().unwrap(),
        args.cmd.iter().map(|x| x.as_str()).collect(),
    )?;

    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = BufReader::new(child.stdout.take().unwrap());

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

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

        child.wait().expect("Error waiting for child");
    });

    Ok(())
}
