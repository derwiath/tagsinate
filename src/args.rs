use clap::{arg, command, value_parser, ArgAction};
use std::path::PathBuf;

pub struct Args {
    pub config_file: PathBuf,
    pub pretend: bool,
}

pub fn parse() -> Args {
    Args::new()
}

impl Args {
    fn new() -> Args {
        let matches = command!()
            .arg(
                arg!(
                    -c --config <FILE> "Name of tagsinate config file"
                )
                .required(false)
                .value_parser(value_parser!(PathBuf))
                .default_value(".tagsinate-config.json"),
            )
            .arg(
                arg!(
                    -p --pretend "Print commands to stdout without executing them"
                )
                .action(ArgAction::SetTrue),
            )
            .get_matches();
        let config_file = matches.get_one::<PathBuf>("config").unwrap();
        let pretend = matches.get_one::<bool>("pretend").unwrap();
        Args {
            config_file: config_file.clone(),
            pretend: *pretend,
        }
    }
}
