use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{Context, Result};
use midly::num::u7;
use regex::Regex;

pub fn load_file_to_map(filepath: &PathBuf) -> Result<HashMap<String, u7>> {
    let data: Vec<String> = fs::read_to_string(filepath)?
        .lines()
        .map(|x| x.to_string())
        .collect();
    let mut result: HashMap<String, u7> = HashMap::new();
    let re = Regex::new(r"(?<val>\d+)\s(?<name>.*)").unwrap();

    for line in data.iter() {
        let Some(capture) = re.captures(line) else {
            continue;
        };

        let note_name = capture["name"].trim();

        if note_name == "-" {
            continue;
        }

        let val = capture["val"].parse::<u8>()?;
        let val = u7::try_from(val).context("failed converting value to u7")?;
        result.insert(note_name.to_string(), val);
    }

    Ok(result)
}

pub fn make_conversion_map(input: &PathBuf, output: &PathBuf) -> Result<HashMap<u7, u7>> {
    let input_map = load_file_to_map(input)?;
    let output_map = load_file_to_map(output)?;

    let mut result: HashMap<u7, u7> = HashMap::new();

    for (k, v) in input_map {
        if let Some(value) = output_map.get(&k) {
            result.insert(v, *value);
        } else {
            result.insert(v, v);
        }
    }

    Ok(result)
}
