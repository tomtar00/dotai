use anyhow::{Context, Result};
use serde_yaml::{Mapping, Value};

pub struct Parsed {
    pub meta: Mapping,
    pub body: String,
}

pub fn parse(content: &str) -> Result<Parsed> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first() != Some(&"---") {
        return Ok(Parsed {
            meta: Mapping::new(),
            body: content.trim().to_string(),
        });
    }

    let mut end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" {
            end = Some(i);
            break;
        }
    }
    let end = end
        .ok_or_else(|| anyhow::anyhow!("unterminated YAML frontmatter (missing closing `---`)"))?;

    let yaml = lines[1..end].join("\n");
    let value: Value = serde_yaml::from_str(&yaml).context("failed to parse YAML frontmatter")?;
    let meta = value.as_mapping().cloned().unwrap_or_default();
    let body = lines[end + 1..].join("\n").trim().to_string();

    Ok(Parsed { meta, body })
}

pub fn get_str(meta: &Mapping, key: &str) -> Option<String> {
    meta.get(Value::String(key.to_string()))
        .and_then(|v| v.as_str())
        .map(String::from)
        .filter(|s| !s.is_empty())
}

pub fn get_list(meta: &Mapping, key: &str) -> Vec<String> {
    match meta.get(Value::String(key.to_string())) {
        Some(Value::String(s)) => split_csv(s),
        Some(Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect(),
        _ => vec![],
    }
}

fn split_csv(s: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '{' => {
                depth += 1;
                cur.push(c);
            }
            '}' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                let t = cur.trim();
                if !t.is_empty() {
                    items.push(t.to_string());
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    let t = cur.trim();
    if !t.is_empty() {
        items.push(t.to_string());
    }
    items
}

pub fn get_bool(meta: &Mapping, key: &str) -> bool {
    meta.get(Value::String(key.to_string()))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}
