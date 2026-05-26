mod app;
mod input;

use anyhow::{Result, bail};
use std::path::PathBuf;

const HELP_TEXT: &str = concat!(
    "pf3-edit ",
    env!("CARGO_PKG_VERSION"),
    "

USAGE:
  pf3-edit [OPTIONS] [FILE]

OPTIONS:
  -h, --help        Show this help text
  -V, --version     Show the version
  --debug-keys  Print raw key events until Esc is pressed
"
);

enum CliCommand {
    Run { path: Option<PathBuf> },
    Help,
    Version,
    DebugKeys,
}

fn parse_cli() -> Result<CliCommand> {
    let mut path: Option<PathBuf> = None;
    let mut debug_keys = false;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => return Ok(CliCommand::Help),
            "-V" | "--version" => return Ok(CliCommand::Version),
            "--debug-keys" => debug_keys = true,
            _ if arg.starts_with('-') => bail!("unknown option: {arg}"),
            _ => {
                if path.is_some() {
                    bail!("expected at most one file path");
                }
                path = Some(PathBuf::from(arg));
            }
        }
    }

    if debug_keys {
        if path.is_some() {
            bail!("--debug-keys does not accept a file path");
        }
        return Ok(CliCommand::DebugKeys);
    }

    Ok(CliCommand::Run { path })
}

fn main() -> Result<()> {
    match parse_cli()? {
        CliCommand::Help => {
            println!("{HELP_TEXT}");
            Ok(())
        }
        CliCommand::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        CliCommand::DebugKeys => app::run_debug_keys(),
        CliCommand::Run { path } => app::run(path.as_deref()),
    }
}
