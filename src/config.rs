use serde::{de, Deserialize, Deserializer};
use serde_json;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

#[derive(Debug)]
pub struct Job {
    pub path: PathBuf,
    pub recurse: bool,
    pub languages: Option<String>,
    pub extras: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub binary: PathBuf,
    pub output_file: PathBuf,
    pub jobs: Vec<Job>,
}

pub fn parse<P: AsRef<Path>>(path: P) -> Result<Config, ()> {
    // Open the file in read-only mode with buffer.
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    };
    let reader = io::BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let config_data: ConfigData = match serde_json::from_reader(reader) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", e);
            return Err(());
        }
    };

    let binary = config_data.binary;
    let recurse = config_data.recurse;
    let output_file = config_data.output_file.unwrap_or("tags".into());
    let extras = config_data.extras;
    let extras_str = match &extras {
        Some(s) => s.as_str(),
        None => "",
    };
    let languages = config_data.languages;
    let languages_str = match &languages {
        Some(s) => s.as_str(),
        None => "",
    };

    let mut jobs =
        Vec::<Job>::with_capacity(config_data.paths.len() + config_data.override_paths.len());
    for path in config_data.paths {
        jobs.push(Job {
            path,
            recurse,
            languages: languages.clone(),
            extras: extras.clone(),
        });
    }

    for override_path in config_data.override_paths {
        jobs.push(Job {
            path: override_path.path,
            recurse: override_path.recurse.unwrap_or(recurse),
            extras: match override_path.extras {
                Some(override_extras) => Some(override_extras.replace("${extras}", &extras_str)),
                None => extras.clone(),
            },
            languages: match override_path.languages {
                Some(override_languages) => {
                    Some(override_languages.replace("${languages}", &languages_str))
                }
                None => languages.clone(),
            },
        });
    }

    Ok(Config {
        binary,
        output_file,
        jobs,
    })
}

#[derive(Deserialize, Debug)]
struct OverridePathData {
    path: PathBuf,
    languages: Option<String>,
    extras: Option<String>,

    #[serde(deserialize_with = "option_bool_from_string", default)]
    recurse: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ConfigData {
    binary: PathBuf,
    output_file: Option<PathBuf>,
    languages: Option<String>,
    extras: Option<String>,

    #[serde(deserialize_with = "bool_from_string", default)]
    recurse: bool,

    paths: Vec<PathBuf>,
    override_paths: Vec<OverridePathData>,
}

fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match bool::from_str(&s) {
        Ok(value) => Ok(value),
        Err(e) => Err(de::Error::custom(e)),
    }
}

fn option_bool_from_string<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match bool::from_str(&s) {
        Ok(value) => Ok(Some(value)),
        Err(e) => Err(de::Error::custom(e)),
    }
}
