use std::path::PathBuf;

pub struct Project {
    pub ai_md: Option<String>,
    pub rules: Vec<Rule>,
    pub agents: Vec<Agent>,
    pub skills: Vec<Skill>,
    pub commands: Vec<Command>,
}

pub struct Rule {
    pub name: String,
    pub description: Option<String>,
    pub globs: Vec<String>,
    pub always_apply: bool,
    pub body: String,
}

pub struct Agent {
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub mode: Option<String>,
    pub tools: Vec<String>,
    pub body: String,
}

pub struct Skill {
    pub name: String,
    pub description: String,
    pub allowed_tools: Vec<String>,
    pub paths: Vec<String>,
    pub body: String,
    pub src_dir: Option<PathBuf>,
}

pub struct Command {
    pub name: String,
    pub description: String,
    pub argument_hint: Option<String>,
    pub agent: Option<String>,
    pub body: String,
}
