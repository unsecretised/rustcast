//! Parser for glob patterns

use std::str::FromStr;

use glob::Pattern;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Pattern>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .iter()
        .map(|x| Pattern::from_str(x).map_err(|e| serde::de::Error::custom(e.msg)))
        .collect()
}

pub fn serialize<S>(patterns: &[Pattern], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    patterns
        .iter()
        .map(|x| x.as_str())
        .collect::<Vec<_>>()
        .serialize(serializer)
}
