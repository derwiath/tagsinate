use clap::{App, Arg};
use std::path::{Path, PathBuf};

pub struct Config {
    pub config_file: PathBuf,
}

pub fn parse_args() -> Config {
    Config::new()
}

struct CargoPackage {
    name: &'static str,
    version: &'static str,
    authors: Vec<&'static str>,
}

impl CargoPackage {
    fn new() -> CargoPackage {
        CargoPackage {
            version: env!("CARGO_PKG_VERSION"),
            name: env!("CARGO_PKG_NAME"),
            authors: env!("CARGO_PKG_AUTHORS").split(':').collect(),
        }
    }
}

impl Config {
    fn new() -> Config {
        let package = CargoPackage::new();
        let matches = App::new(package.name)
            .version(package.version)
            .author(package.authors[0])
            .about(
                "
Generate tags based on a config file
                ",
            )
            .arg(
                Arg::with_name("config-file")
                    .short("c")
                    .long("config-file")
                    .takes_value(true)
                    .default_value(".tagsinate-config.json")
                    .help("")
                    .value_name("FILENAME"),
            )
            .get_matches();

        let config_file = Path::new(matches.value_of("config-file").unwrap()).to_owned();

        Config { config_file }
    }
}
