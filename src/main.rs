use std::path::{Path, PathBuf};
use std::{env, fmt, io};

extern crate clap;

mod args;
mod config;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ErrorKind {
    ConfigFileNotFound,
    FailedToParseConfigFile,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }

    pub fn kind(self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn find_config_file(config_filename: &Path) -> io::Result<PathBuf> {
    if config_filename.is_file() {
        return Ok(config_filename.to_owned());
    }
    let mut directory = env::current_dir()?;
    loop {
        let candidate = directory.join(&config_filename);
        if candidate.is_file() {
            return Ok(candidate);
        }

        if !directory.pop() {
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }
    }
}

fn main() -> Result<(), Error> {
    let args = args::parse();
    let config_file: PathBuf = match find_config_file(&args.config_file) {
        Ok(config_file) => config_file,
        Err(_) => {
            eprintln!("Failed to find config file {}", args.config_file.display());
            return Err(Error::new(ErrorKind::ConfigFileNotFound));
        }
    };
    println!("Using {}", config_file.display());

    let config = match config::parse(&config_file) {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Failed to parse config file {}", config_file.display());
            return Err(Error::new(ErrorKind::FailedToParseConfigFile));
        }
    };

    println!("{:?}", config);

    return Ok(());
}
