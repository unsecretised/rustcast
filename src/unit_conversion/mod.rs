//! Unit conversion parsing and calculation.

use crate::unit_conversion::defs::{UNITS, UnitDef};

mod defs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitCategory {
    Length,
    Mass,
    Volume,
    Temperature,
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub source_value: f64,
    pub source_unit: &'static UnitDef,
    pub target_value: f64,
    pub target_unit: &'static UnitDef,
}

#[derive(Debug, Clone)]
struct ParsedQuery {
    value: f64,
    source_unit: &'static UnitDef,
    target_unit: Option<&'static UnitDef>,
}

pub fn convert_query(query: &str) -> Option<Vec<ConversionResult>> {
    let parsed = parse_query(query)?;
    let base_value = to_base(parsed.value, parsed.source_unit);

    let mut results = Vec::new();
    let targets: Vec<&UnitDef> = match parsed.target_unit {
        Some(target) => vec![target],
        None => UNITS
            .iter()
            .filter(|unit| unit.category == parsed.source_unit.category)
            .collect(),
    };

    for target_unit in targets {
        if target_unit.name == parsed.source_unit.name {
            continue;
        }
        let target_value = from_base(base_value, target_unit);
        results.push(ConversionResult {
            source_value: parsed.value,
            source_unit: parsed.source_unit,
            target_value,
            target_unit,
        });
    }

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

pub fn format_number(value: f64) -> String {
    let value = if value.abs() < 1e-9 { 0.0 } else { value };
    let mut s = format!("{value:.6}");
    if let Some(dot_pos) = s.find('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') && dot_pos == s.len() - 1 {
            s.pop();
        }
    }
    s
}

fn parse_query(query: &str) -> Option<ParsedQuery> {
    let (value_str, rest) = parse_number_prefix(query)?;
    let value: f64 = value_str.parse().ok()?;
    let rest = rest.trim_start();
    if rest.is_empty() {
        return None;
    }

    let rest_lc = rest.to_lowercase();
    let tokens: Vec<&str> = rest_lc.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let source_unit = find_unit(tokens[0])?;
    match tokens.len() {
        1 => Some(ParsedQuery {
            value,
            source_unit,
            target_unit: None,
        }),
        2 => {
            let target_unit = find_unit(tokens[1])?;
            if target_unit.category != source_unit.category {
                return None;
            }
            Some(ParsedQuery {
                value,
                source_unit,
                target_unit: Some(target_unit),
            })
        }
        3 if tokens[1] == "to" || tokens[1] == "in" => {
            let target_unit = find_unit(tokens[2])?;
            if target_unit.category != source_unit.category {
                return None;
            }
            Some(ParsedQuery {
                value,
                source_unit,
                target_unit: Some(target_unit),
            })
        }
        _ => None,
    }
}

fn parse_number_prefix(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }

    let mut chars = s.char_indices().peekable();
    if let Some((_, c)) = chars.peek()
        && (*c == '+' || *c == '-')
    {
        chars.next();
    }

    let mut end = 0;
    let mut has_digit = false;
    while let Some((idx, c)) = chars.peek().cloned() {
        if c.is_ascii_digit() {
            has_digit = true;
            end = idx + c.len_utf8();
            chars.next();
        } else if c == '.' {
            end = idx + c.len_utf8();
            chars.next();
        } else {
            break;
        }
    }

    if !has_digit || end == 0 {
        return None;
    }

    let (num, rest) = s.split_at(end);
    Some((num, rest))
}

fn find_unit(token: &str) -> Option<&'static UnitDef> {
    let token = token.trim();
    if token.is_empty() {
        return None;
    }

    UNITS
        .iter()
        .find(|unit| unit.name == token || unit.aliases.contains(&token))
}

fn to_base(value: f64, unit: &UnitDef) -> f64 {
    (value + unit.offset) * unit.scale
}

fn from_base(value: f64, unit: &UnitDef) -> f64 {
    value / unit.scale - unit.offset
}
