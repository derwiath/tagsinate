use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fmt};

extern crate clap;

mod args;
mod config;

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

fn run_ctags<S: AsRef<OsStr> + fmt::Debug>(
    binary: &S,
    output_file: &S,
    append: bool,
    job: &config::Job,
) {
    let mut args: Vec<OsString> = Vec::new();
    args.push(OsString::from("-o"));
    args.push(OsString::from(output_file));
    if let Some(languages) = &job.languages {
        args.push(OsString::from(format!("--languages={}", languages)));
    }
    if let Some(extras) = &job.extras {
        args.push(OsString::from(format!("--extras={}", extras)));
    }
    if append {
        args.push(OsString::from("--append"));
    }
    if job.recurse {
        args.push(OsString::from("--recurse"));
    }
    if let Some(exclude) = &job.exclude {
        args.push(OsString::from(format!("--exclude={}", exclude)));
    }
    if let Some(exclude_exception) = &job.exclude_exception {
        args.push(OsString::from(format!(
            "--exclude_exception={}",
            exclude_exception
        )));
    }
    args.push(OsString::from(&job.path));

    println!("{:?} {:?}", binary, args);
    let output = Command::new(binary)
        .args(args)
        .output()
        .expect("ctags failed to start");
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse();
    print!("Finding {} ... ", args.config_file.display());
    let config_file: PathBuf = match find_config_file(&args.config_file) {
        Ok(config_file) => {
            println!("[ok]");
            config_file
        }
        Err(e) => {
            println!("[fail]");
            return Err(Box::new(e));
        }
    };
    print!("Parsing {} ... ", config_file.display());
    let config = match config::parse(&config_file) {
        Ok(config) => {
            println!("[ok]");
            config
        }
        Err(e) => {
            println!("[fail]");
            return Err(e);
        }
    };

    for (i, job) in config.jobs.iter().enumerate() {
        let append = if i > 0 { true } else { false };
        run_ctags(&config.binary, &config.output_file, append, job);
    }

    return Ok(());
}
