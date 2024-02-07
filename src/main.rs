use colored::*;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fmt, fs};

extern crate clap;
extern crate colored;

mod args;
mod config;

fn find_file_in_ancestors(basename: &Path) -> io::Result<(PathBuf, usize)> {
    let cwd = env::current_dir()?;
    for (ancestor_count, directory) in cwd.ancestors().enumerate() {
        let candidate = directory.join(basename);
        if candidate.is_file() {
            return Ok((candidate, ancestor_count));
        }
    }
    Err(io::Error::from(io::ErrorKind::NotFound))
}

fn find_config_file(config_filename: &Path) -> io::Result<(PathBuf, usize)> {
    if config_filename.is_file() {
        Ok((config_filename.to_owned(), 0))
    } else if config_filename.is_absolute() {
        Err(io::Error::from(io::ErrorKind::NotFound))
    } else {
        find_file_in_ancestors(config_filename)
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
    if let Some(language_maps) = &job.language_maps {
        args.push(OsString::from(format!("--langmap={}", language_maps)));
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
    for (symbol, definition) in &job.defines {
        if let Some(definition) = definition {
            args.push(OsString::from(format!("-D{}='{}'", symbol, definition)));
        } else {
            args.push(OsString::from(format!("-D{}", symbol)));
        }
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
    let ok = "[ok]".green();
    let fail = "[fail]".red();

    let args = args::parse();
    print!("Finding {} ... ", args.config_file.display());
    let (config_file, ancestor_count) = match find_config_file(&args.config_file) {
        Ok((config_file, ancestor_count)) => {
            println!("{}", &ok);
            (config_file, ancestor_count)
        }
        Err(e) => {
            println!("{}", &fail);
            return Err(Box::new(e));
        }
    };
    print!("Parsing {} ... ", {
        if args.config_file.is_absolute() {
            config_file.clone()
        } else {
            let mut p = PathBuf::new();
            for _ in 0..ancestor_count {
                p.push("..");
            }
            p.push(&args.config_file);
            p.clone()
        }
        .display()
    });
    let config = match config::parse(&config_file) {
        Ok(config) => {
            println!("{}", &ok);
            config
        }
        Err(e) => {
            println!("{}", &fail);
            return Err(e);
        }
    };

    let config_file_parent = config_file
        .parent()
        .expect("Failed to get parent directory of config file");

    let config_file_dir: PathBuf = if !config_file_parent.as_os_str().is_empty() {
        config_file_parent.to_path_buf()
    } else {
        env::current_dir()?
    };

    print!(
        "Setting current working directory to {} ... ",
        config_file_dir.display()
    );
    match env::set_current_dir(&config_file_dir) {
        Ok(()) => {
            println!("{}", &ok);
        }
        Err(e) => {
            println!("{}", &fail);
            return Err(Box::new(e));
        }
    };

    let temp_output_file = {
        let mut output_file = config.output_file.clone();
        output_file.set_extension("tagsinate");
        output_file
    };
    if temp_output_file.exists() {
        print!(
            "Removing temporary tags file {} ...",
            temp_output_file.display()
        );
        match fs::remove_file(&temp_output_file) {
            Ok(()) => {
                println!("{}", &ok);
            }
            Err(e) => {
                println!("{}", &fail);
                return Err(Box::new(e));
            }
        };
    }

    println!("Generating tags into {}", temp_output_file.display());
    for (i, job) in config.jobs.iter().enumerate() {
        let append = i > 0;
        run_ctags(&config.binary, &temp_output_file, append, job);
    }

    if config.output_file.exists() {
        print!(
            "Removing output tags file {} ...",
            config.output_file.display()
        );
        match fs::remove_file(&config.output_file) {
            Ok(()) => {
                println!("{}", &ok);
            }
            Err(e) => {
                println!("{}", &fail);
                return Err(Box::new(e));
            }
        };
    }

    print!(
        "Renaming temporaty output tags file {} into {}...",
        temp_output_file.display(),
        config.output_file.display()
    );
    match fs::rename(&temp_output_file, &config.output_file) {
        Ok(()) => {
            println!("{}", &ok);
        }
        Err(e) => {
            println!("{}", &fail);
            return Err(Box::new(e));
        }
    };

    Ok(())
}
