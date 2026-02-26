//! Parser for include patterns

use std::{path::PathBuf, str::FromStr, sync::LazyLock};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

static WITH_DEPTH_REGEX: LazyLock<Regex> = LazyLock::new(||
    // (.*?)            Matches the path (group 0)
    // (?::[..])?       Optionally match the depth specifier without capturing
    // (?<depth>[0-9]+) Capture the number for maximum depth in the named group depth
    Regex::new("^(.*?)(?::(?<depth>[0-9]+))?$").unwrap());

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub path: PathBuf,
    pub max_depth: usize,
}

impl FromStr for Pattern {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let matched = WITH_DEPTH_REGEX.captures(str);

        if let Some(x) = matched {
            Ok(Pattern {
                path: PathBuf::from(&x[1]),
                max_depth: x.name("depth").map_or(1, |m| m.as_str().parse().unwrap()),
            })
        } else {
            Err(anyhow::Error::msg("Invalid pattern syntax: \"{x}\""))
        }
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Pattern>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .iter()
        .map(|x| x.parse().map_err(serde::de::Error::custom))
        .collect()
}

pub fn serialize<S>(patterns: &[Pattern], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    patterns
        .iter()
        .map(|x| format!("{}:{}", x.path.to_string_lossy(), x.max_depth))
        .collect::<Vec<_>>()
        .serialize(serializer)
}
