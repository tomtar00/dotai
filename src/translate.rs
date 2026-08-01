pub enum VarStyle {
    Dollar,
    OpenCode,
    Keep,
}

pub fn tool_to_claude(tool: &str) -> Option<String> {
    match tool {
        "read" => Some("Read".to_string()),
        "write" => Some("Write".to_string()),
        "edit" => Some("Edit".to_string()),
        "bash" => Some("Bash".to_string()),
        "grep" => Some("Grep".to_string()),
        "glob" => Some("Glob".to_string()),
        "list" => Some("Glob".to_string()),
        "webfetch" => Some("WebFetch".to_string()),
        "websearch" => Some("WebSearch".to_string()),
        "task" => Some("Agent".to_string()),
        _ if tool.starts_with("mcp__") => Some(tool.to_string()),
        _ => None,
    }
}

pub fn is_readonly_tool(tool: &str) -> bool {
    matches!(
        tool,
        "read" | "grep" | "glob" | "list" | "webfetch" | "websearch"
    )
}

pub fn translate_command_vars(body: &str, style: VarStyle) -> String {
    let chars: Vec<char> = body.chars().collect();
    let mut out = String::with_capacity(body.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            let mut j = i + 2;
            let mut name = String::new();
            let mut found = false;
            while j + 1 < chars.len() {
                if chars[j] == '}' && chars[j + 1] == '}' {
                    found = true;
                    break;
                }
                name.push(chars[j]);
                j += 1;
            }
            if found {
                match style {
                    VarStyle::Keep => {
                        out.push_str(&format!("{{{{{}}}}}", name));
                    }
                    VarStyle::Dollar => {
                        if name == "input" {
                            out.push_str("$ARGUMENTS");
                        } else if name.chars().all(|c| c.is_ascii_digit()) {
                            let n: usize = name.parse().unwrap_or(0);
                            out.push_str(&format!("${}", n.saturating_sub(1)));
                        } else {
                            out.push_str(&format!("${}", name));
                        }
                    }
                    VarStyle::OpenCode => {
                        if name == "input" {
                            out.push_str("$ARGUMENTS");
                        } else if name.chars().all(|c| c.is_ascii_digit()) {
                            out.push_str(&format!("${}", name));
                        } else {
                            out.push_str(&format!("{{{{{}}}}}", name));
                        }
                    }
                }
                i = j + 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn command_named_args(body: &str) -> Vec<String> {
    let chars: Vec<char> = body.chars().collect();
    let mut args = Vec::new();
    let mut i = 0;
    while i + 1 < chars.len() {
        if chars[i] == '{' && chars[i + 1] == '{' {
            let mut j = i + 2;
            let mut name = String::new();
            let mut found = false;
            while j + 1 < chars.len() {
                if chars[j] == '}' && chars[j + 1] == '}' {
                    found = true;
                    break;
                }
                name.push(chars[j]);
                j += 1;
            }
            if found {
                if name != "input"
                    && !name.chars().all(|c| c.is_ascii_digit())
                    && !args.contains(&name)
                {
                    args.push(name);
                }
                i = j + 2;
                continue;
            }
        }
        i += 1;
    }
    args
}

pub fn strip_provider_prefix(model: &str) -> String {
    model
        .strip_prefix("anthropic/")
        .or_else(|| model.strip_prefix("claude/"))
        .unwrap_or(model)
        .to_string()
}
