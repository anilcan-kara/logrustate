use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use chrono::{NaiveDateTime, Local, Datelike};

pub struct StateDB {
    path: String,
    pub entries: HashMap<String, NaiveDateTime>,
}

impl StateDB {
    pub fn load(path_str: &str) -> Result<Self> {
        let mut entries = HashMap::new();
        let path = Path::new(path_str);
        if !path.exists() {
            return Ok(StateDB {
                path: path_str.to_string(),
                entries,
            });
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read state file: {}", path_str))?;

        let mut lines = content.lines();
        if let Some(header) = lines.next() {
            if !header.starts_with("logrotate state") && !header.starts_with("logrustate state") {
            }
        }

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(first_quote) = line.find('"') {
                if let Some(second_quote) = line[first_quote + 1..].find('"') {
                    let path_val = &line[first_quote + 1..first_quote + 1 + second_quote];
                    let date_part = line[first_quote + 1 + second_quote + 1..].trim();

                    let parts: Vec<&str> = date_part.split(|c| c == '-' || c == ':').collect();
                    if parts.len() >= 6 {
                        let year = parts[0].parse::<i32>().unwrap_or(1970);
                        let month = parts[1].parse::<u32>().unwrap_or(1);
                        let day = parts[2].parse::<u32>().unwrap_or(1);
                        let hour = parts[3].parse::<u32>().unwrap_or(0);
                        let min = parts[4].parse::<u32>().unwrap_or(0);
                        let sec = parts[5].parse::<u32>().unwrap_or(0);

                        if let Some(dt) = chrono::NaiveDate::from_ymd_opt(year, month, day)
                            .and_then(|d| d.and_hms_opt(hour, min, sec)) {
                            entries.insert(path_val.to_string(), dt);
                        }
                    }
                }
            }
        }

        Ok(StateDB {
            path: path_str.to_string(),
            entries,
        })
    }

    pub fn save(&self) -> Result<()> {
        let mut content = String::new();
        content.push_str("logrotate state -- version 2\n");

        let mut sorted_keys: Vec<_> = self.entries.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            if let Some(dt) = self.entries.get(key) {
                let formatted = format!(
                    "\"{}\" {}-{}-{}-{}:{}:{}\n",
                    key,
                    dt.year(),
                    dt.month(),
                    dt.day(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                );
                content.push_str(&formatted);
            }
        }

        if let Some(parent) = Path::new(&self.path).parent() {
            fs::create_dir_all(parent).ok();
        }

        fs::write(&self.path, content)
            .with_context(|| format!("Failed to write state file: {}", self.path))?;

        Ok(())
    }

    pub fn update(&mut self, path: &str) {
        self.entries.insert(path.to_string(), Local::now().naive_local());
    }

    pub fn should_rotate(&self, path: &str, frequency: &str) -> bool {
        let last_rotated = match self.entries.get(path) {
            Some(dt) => *dt,
            None => return true,
        };

        let now = Local::now().naive_local();
        let diff = now.signed_duration_since(last_rotated);

        match frequency {
            "hourly" => diff.num_hours() >= 1,
            "daily" => diff.num_days() >= 1,
            "weekly" => diff.num_weeks() >= 1,
            "monthly" => {
                let last_month = last_rotated.month();
                let now_month = now.month();
                last_month != now_month || diff.num_days() >= 31
            }
            "yearly" => {
                let last_year = last_rotated.year();
                let now_year = now.year();
                last_year != now_year || diff.num_days() >= 365
            }
            _ => false,
        }
    }
}
