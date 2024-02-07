use serde::{de, Deserialize, Deserializer};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

#[derive(Debug)]
pub struct Job {
    pub path: PathBuf,
    pub recurse: bool,
    pub languages: Option<String>,
    pub language_maps: Option<String>,
    pub extras: Option<String>,
    pub exclude: Option<String>,
    pub exclude_exception: Option<String>,
    pub defines: Vec<(String, Option<String>)>,
}

#[derive(Debug)]
pub struct Config {
    pub binary: PathBuf,
    pub output_file: PathBuf,
    pub jobs: Vec<Job>,
}
fn get_language_map_string(language_maps: &[LanguageMapData]) -> String {
    language_maps
        .iter()
        .map(|lang_map| format!("{}:{}", lang_map.language.display(), lang_map.extensions))
        .fold("".to_string(), |acc, lang_map| {
            if acc.is_empty() {
                lang_map
            } else {
                format!("{acc},{lang_map}")
            }
        })
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
    let extras_str = extras.as_ref().map_or("", |s| s.as_str());
    let languages = config_data.languages;
    let languages_str = languages.as_ref().map_or("", |s| s.as_str());
    let language_maps = config_data
        .language_maps
        .map(|maps| get_language_map_string(&maps));
    let language_maps_str = language_maps.as_ref().map_or("", |s| s.as_str());
    let exclude = config_data.exclude;
    let exclude_str = exclude.as_ref().map_or("", |s| s.as_str());
    let exclude_exception = config_data.exclude_exception;
    let exclude_exception_str = exclude_exception.as_ref().map_or("", |s| s.as_str());

    let defines: Vec<(String, Option<String>)> = config_data
        .defines
        .iter()
        .map(|define| (define.symbol.clone(), define.definition.clone()))
        .collect();

    let mut jobs =
        Vec::<Job>::with_capacity(config_data.paths.len() + config_data.override_paths.len());
    for path in config_data.paths {
        jobs.push(Job {
            path,
            recurse,
            languages: languages.clone(),
            language_maps: language_maps.clone(),
            extras: extras.clone(),
            exclude: exclude.clone(),
            exclude_exception: exclude_exception.clone(),
            defines: defines.clone(),
        });
    }

    for override_path in config_data.override_paths {
        jobs.push(Job {
            path: override_path.path,
            recurse: override_path.recurse.unwrap_or(recurse),
            extras: match override_path.extras {
                Some(override_extras) => Some(override_extras.replace("${extras}", extras_str)),
                None => extras.clone(),
            },
            languages: match override_path.languages {
                Some(override_languages) => {
                    Some(override_languages.replace("${languages}", languages_str))
                }
                None => languages.clone(),
            },
            language_maps: match override_path.language_maps {
                Some(override_language_maps) => {
                    let override_language_maps = get_language_map_string(&override_language_maps);
                    Some(override_language_maps.replace("${language_maps}", language_maps_str))
                }
                None => language_maps.clone(),
            },
            exclude: match override_path.exclude {
                Some(override_exclude) => Some(override_exclude.replace("${exclude}", exclude_str)),
                None => exclude.clone(),
            },
            exclude_exception: match override_path.exclude_exception {
                Some(override_exclude_exception) => Some(
                    override_exclude_exception
                        .replace("${excludeException}", exclude_exception_str),
                ),
                None => exclude_exception.clone(),
            },
            defines: defines.clone(),
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
struct LanguageMapData {
    language: PathBuf,
    extensions: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct OverridePathData {
    path: PathBuf,
    languages: Option<String>,
    language_maps: Option<Vec<LanguageMapData>>,
    extras: Option<String>,
    exclude: Option<String>,
    exclude_exception: Option<String>,

    #[serde(deserialize_with = "option_bool_from_string", default)]
    recurse: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DefineData {
    symbol: String,
    definition: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ConfigData {
    binary: PathBuf,
    output_file: Option<PathBuf>,
    languages: Option<String>,
    language_maps: Option<Vec<LanguageMapData>>,
    extras: Option<String>,
    exclude: Option<String>,
    exclude_exception: Option<String>,

    #[serde(deserialize_with = "bool_from_string", default)]
    recurse: bool,

    defines: Vec<DefineData>,

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
