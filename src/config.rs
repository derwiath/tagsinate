use serde::{de, Deserialize, Deserializer};
use serde_json;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

#[derive(Debug)]
pub struct Job {
    pub path: PathBuf,
    pub recurse: bool,
    pub languages: Option<String>,
    pub extras: Option<String>,
    pub exclude: Option<String>,
    pub exclude_exception: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub binary: PathBuf,
    pub output_file: PathBuf,
    pub jobs: Vec<Job>,
}

pub fn parse<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let config_data: ConfigData = serde_json::from_reader(reader)?;

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
    let exclude = config_data.exclude;
    let exclude_str = match &exclude {
        Some(s) => s.as_str(),
        None => "",
    };
    let exclude_exception = config_data.exclude_exception;
    let exclude_exception_str = match &exclude_exception {
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
            exclude: exclude.clone(),
            exclude_exception: exclude_exception.clone(),
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
            exclude: match override_path.exclude {
                Some(override_exclude) => {
                    Some(override_exclude.replace("${exclude}", &exclude_str))
                }
                None => exclude.clone(),
            },
            exclude_exception: match override_path.exclude_exception {
                Some(override_exclude_exception) => Some(
                    override_exclude_exception
                        .replace("${excludeException}", &exclude_exception_str),
                ),
                None => exclude_exception.clone(),
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
#[serde(rename_all = "camelCase")]
struct OverridePathData {
    path: PathBuf,
    languages: Option<String>,
    extras: Option<String>,
    exclude: Option<String>,
    exclude_exception: Option<String>,

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
    exclude: Option<String>,
    exclude_exception: Option<String>,

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
