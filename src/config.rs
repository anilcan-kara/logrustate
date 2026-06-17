use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogConfig {
    pub rotate: Option<u32>,
    pub frequency: Option<String>,
    pub size: Option<u64>,
    pub minsize: Option<u64>,
    pub maxsize: Option<u64>,
    pub maxage: Option<u32>,
    pub compress: Option<bool>,
    pub delaycompress: Option<bool>,
    pub copytruncate: Option<bool>,
    pub create: Option<String>,
    pub missingok: Option<bool>,
    pub notifempty: Option<bool>,
    pub dateext: Option<bool>,
    pub dateformat: Option<String>,
    pub olddir: Option<String>,
    pub postrotate: Option<String>,
    pub prerotate: Option<String>,
    pub sharedscripts: Option<bool>,
    pub mail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub paths: Vec<String>,
    #[serde(flatten)]
    pub config: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TomlConfig {
    pub global: Option<LogConfig>,
    pub entries: Option<Vec<LogEntry>>,
}

pub fn parse_config_file(path: &Path) -> Result<Vec<LogEntry>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {:?}", path))?;

    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
        return parse_toml(&content);
    }

    parse_legacy(&content, path.parent().unwrap_or_else(|| Path::new("")))
}

fn parse_toml(content: &str) -> Result<Vec<LogEntry>> {
    let toml_data: TomlConfig = toml::from_str(content)?;
    let global = toml_data.global.unwrap_or_default();
    let mut entries = toml_data.entries.unwrap_or_default();

    for entry in &mut entries {
        entry.config.rotate = entry.config.rotate.or(global.rotate);
        entry.config.frequency = entry.config.frequency.clone().or(global.frequency.clone());
        entry.config.size = entry.config.size.or(global.size);
        entry.config.minsize = entry.config.minsize.or(global.minsize);
        entry.config.maxsize = entry.config.maxsize.or(global.maxsize);
        entry.config.maxage = entry.config.maxage.or(global.maxage);
        entry.config.compress = entry.config.compress.or(global.compress);
        entry.config.delaycompress = entry.config.delaycompress.or(global.delaycompress);
        entry.config.copytruncate = entry.config.copytruncate.or(global.copytruncate);
        entry.config.create = entry.config.create.clone().or(global.create.clone());
        entry.config.missingok = entry.config.missingok.or(global.missingok);
        entry.config.notifempty = entry.config.notifempty.or(global.notifempty);
        entry.config.dateext = entry.config.dateext.or(global.dateext);
        entry.config.dateformat = entry.config.dateformat.clone().or(global.dateformat.clone());
        entry.config.olddir = entry.config.olddir.clone().or(global.olddir.clone());
        entry.config.postrotate = entry.config.postrotate.clone().or(global.postrotate.clone());
        entry.config.prerotate = entry.config.prerotate.clone().or(global.prerotate.clone());
        entry.config.sharedscripts = entry.config.sharedscripts.or(global.sharedscripts);
        entry.config.mail = entry.config.mail.clone().or(global.mail.clone());
    }

    Ok(entries)
}

fn parse_legacy(content: &str, base_dir: &Path) -> Result<Vec<LogEntry>> {
    let mut global = LogConfig::default();
    let mut entries: Vec<LogEntry> = Vec::new();

    let mut current_paths: Option<Vec<String>> = None;
    let mut current_config = LogConfig::default();
    let mut current_script_type: Option<String> = None;
    let mut current_script_body = String::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(ref script_type) = current_script_type {
            if line == "endscript" {
                match script_type.as_str() {
                    "postrotate" => {
                        if current_paths.is_some() {
                            current_config.postrotate = Some(current_script_body.trim().to_string());
                        } else {
                            global.postrotate = Some(current_script_body.trim().to_string());
                        }
                    }
                    "prerotate" => {
                        if current_paths.is_some() {
                            current_config.prerotate = Some(current_script_body.trim().to_string());
                        } else {
                            global.prerotate = Some(current_script_body.trim().to_string());
                        }
                    }
                    _ => {}
                }
                current_script_type = None;
                current_script_body.clear();
            } else {
                current_script_body.push_str(raw_line);
                current_script_body.push('\n');
            }
            continue;
        }

        if line.starts_with("include ") {
            let include_path_str = line["include ".len()..].trim().trim_matches('"');
            let include_path = PathBuf::from(include_path_str);
            let absolute_path = if include_path.is_absolute() {
                include_path
            } else {
                base_dir.join(include_path)
            };

            let pattern = absolute_path.to_string_lossy().to_string();
            if let Ok(paths) = glob::glob(&pattern) {
                for path_entry in paths {
                    if let Ok(p) = path_entry {
                        if p.is_file() {
                            if let Ok(mut sub_entries) = parse_config_file(&p) {
                                entries.append(&mut sub_entries);
                            }
                        }
                    }
                }
            }
            continue;
        }

        if line.ends_with('{') {
            let paths_part = line[..line.len() - 1].trim();
            let paths: Vec<String> = paths_part
                .split_whitespace()
                .map(|s| s.trim_matches('"').to_string())
                .collect();
            current_paths = Some(paths);
            current_config = LogConfig::default();
            continue;
        }

        if line == "}" {
            if let Some(paths) = current_paths.take() {
                entries.push(LogEntry {
                    paths,
                    config: current_config.clone(),
                });
            }
            continue;
        }

        if line == "postrotate" || line == "prerotate" {
            current_script_type = Some(line.to_string());
            current_script_body.clear();
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let directive = parts[0];
        let value = if parts.len() > 1 { Some(parts[1..].join(" ")) } else { None };

        let target = if current_paths.is_some() {
            &mut current_config
        } else {
            &mut global
        };

        match directive {
            "rotate" => {
                if let Some(v) = value {
                    target.rotate = v.parse().ok();
                }
            }
            "daily" | "weekly" | "monthly" | "yearly" | "hourly" => {
                target.frequency = Some(directive.to_string());
            }
            "size" | "minsize" | "maxsize" => {
                if let Some(v) = value {
                    let bytes = parse_size_to_bytes(&v);
                    match directive {
                        "size" => target.size = Some(bytes),
                        "minsize" => target.minsize = Some(bytes),
                        "maxsize" => target.maxsize = Some(bytes),
                        _ => {}
                    }
                }
            }
            "maxage" => {
                if let Some(v) = value {
                    target.maxage = v.parse().ok();
                }
            }
            "compress" => {
                target.compress = Some(true);
            }
            "nocompress" => {
                target.compress = Some(false);
            }
            "delaycompress" => {
                target.delaycompress = Some(true);
            }
            "nodelaycompress" => {
                target.delaycompress = Some(false);
            }
            "copytruncate" => {
                target.copytruncate = Some(true);
            }
            "nocopytruncate" => {
                target.copytruncate = Some(false);
            }
            "create" => {
                target.create = value.or(Some("".to_string()));
            }
            "nocreate" => {
                target.create = None;
            }
            "missingok" => {
                target.missingok = Some(true);
            }
            "nomissingok" => {
                target.missingok = Some(false);
            }
            "notifempty" => {
                target.notifempty = Some(true);
            }
            "ifempty" => {
                target.notifempty = Some(false);
            }
            "dateext" => {
                target.dateext = Some(true);
            }
            "nodateext" => {
                target.dateext = Some(false);
            }
            "dateformat" => {
                target.dateformat = value;
            }
            "olddir" => {
                target.olddir = value;
            }
            "noolddir" => {
                target.olddir = None;
            }
            "sharedscripts" => {
                target.sharedscripts = Some(true);
            }
            "nosharedscripts" => {
                target.sharedscripts = Some(false);
            }
            "mail" => {
                target.mail = value;
            }
            _ => {}
        }
    }

    for entry in &mut entries {
        entry.config.rotate = entry.config.rotate.or(global.rotate);
        entry.config.frequency = entry.config.frequency.clone().or(global.frequency.clone());
        entry.config.size = entry.config.size.or(global.size);
        entry.config.minsize = entry.config.minsize.or(global.minsize);
        entry.config.maxsize = entry.config.maxsize.or(global.maxsize);
        entry.config.maxage = entry.config.maxage.or(global.maxage);
        entry.config.compress = entry.config.compress.or(global.compress);
        entry.config.delaycompress = entry.config.delaycompress.or(global.delaycompress);
        entry.config.copytruncate = entry.config.copytruncate.or(global.copytruncate);
        entry.config.create = entry.config.create.clone().or(global.create.clone());
        entry.config.missingok = entry.config.missingok.or(global.missingok);
        entry.config.notifempty = entry.config.notifempty.or(global.notifempty);
        entry.config.dateext = entry.config.dateext.or(global.dateext);
        entry.config.dateformat = entry.config.dateformat.clone().or(global.dateformat.clone());
        entry.config.olddir = entry.config.olddir.clone().or(global.olddir.clone());
        entry.config.postrotate = entry.config.postrotate.clone().or(global.postrotate.clone());
        entry.config.prerotate = entry.config.prerotate.clone().or(global.prerotate.clone());
        entry.config.sharedscripts = entry.config.sharedscripts.or(global.sharedscripts);
        entry.config.mail = entry.config.mail.clone().or(global.mail.clone());
    }

    Ok(entries)
}

fn parse_size_to_bytes(val: &str) -> u64 {
    let lower = val.to_lowercase();
    let num_part: String = lower.chars().take_while(|c| c.is_digit(10)).collect();
    let num: u64 = num_part.parse().unwrap_or(0);
    if lower.ends_with('k') {
        num * 1024
    } else if lower.ends_with('m') {
        num * 1024 * 1024
    } else if lower.ends_with('g') {
        num * 1024 * 1024 * 1024
    } else {
        num
    }
}
