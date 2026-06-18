use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result, anyhow};
use chrono::{Local, Datelike};
use colored::*;

use crate::config::{LogEntry, LogConfig};
use crate::state::StateDB;
use crate::compress::compress_file;

pub struct Rotator<'a> {
    pub state_db: &'a mut StateDB,
    pub dry_run: bool,
    pub verbose: bool,
    pub force: bool,
}

impl<'a> Rotator<'a> {
    pub fn new(state_db: &'a mut StateDB, dry_run: bool, verbose: bool, force: bool) -> Self {
        Rotator {
            state_db,
            dry_run,
            verbose,
            force,
        }
    }

    pub fn process_entry(&mut self, entry: &LogEntry) -> Result<()> {
        let mut matched_files = Vec::new();
        for path_pattern in &entry.paths {
            if path_pattern.contains('*') || path_pattern.contains('?') {
                if let Ok(paths) = glob::glob(path_pattern) {
                    for p_entry in paths {
                        if let Ok(p) = p_entry {
                            if p.is_file() {
                                matched_files.push(p);
                            }
                        }
                    }
                }
            } else {
                let p = PathBuf::from(path_pattern);
                if p.exists() && p.is_file() {
                    matched_files.push(p);
                }
            }
        }

        if matched_files.is_empty() {
            if self.verbose {
                println!("No matching files found for pattern {:?}", entry.paths);
            }
            return Ok(());
        }

        let mut files_to_rotate = Vec::new();
        for file in &matched_files {
            if self.should_rotate_file(file, &entry.config)? {
                files_to_rotate.push(file.clone());
            }
        }

        if files_to_rotate.is_empty() {
            if self.verbose {
                println!("No files require rotation in this block.");
            }
            return Ok(());
        }

        if let Some(ref script) = entry.config.prerotate {
            if entry.config.sharedscripts == Some(true) {
                self.run_script(script, &entry.paths.join(" "))?;
            }
        }

        for file in &files_to_rotate {
            if let Some(ref script) = entry.config.prerotate {
                if entry.config.sharedscripts != Some(true) {
                    self.run_script(script, &file.to_string_lossy())?;
                }
            }

            self.rotate_single_file(file, &entry.config)?;

            if let Some(ref script) = entry.config.postrotate {
                if entry.config.sharedscripts != Some(true) {
                    self.run_script(script, &file.to_string_lossy())?;
                }
            }
        }

        if let Some(ref script) = entry.config.postrotate {
            if entry.config.sharedscripts == Some(true) {
                self.run_script(script, &entry.paths.join(" "))?;
            }
        }

        Ok(())
    }

    fn should_rotate_file(&self, path: &Path, config: &LogConfig) -> Result<bool> {
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                if config.missingok == Some(true) {
                    return Ok(false);
                }
                return Err(anyhow::Error::new(e).context(format!("Failed to get metadata for {:?}", path)));
            }
        };

        let size = metadata.len();
        if size == 0 && config.notifempty == Some(true) {
            if self.verbose {
                println!("{:?} is empty and notifempty is set, skipping.", path);
            }
            return Ok(false);
        }

        if self.force {
            if self.verbose {
                println!("Rotation forced for {:?}", path);
            }
            return Ok(true);
        }

        let path_str = path.to_string_lossy().to_string();

        let time_rotate = if let Some(ref freq) = config.frequency {
            self.state_db.should_rotate(&path_str, freq)
        } else {
            false
        };

        if let Some(s) = config.size {
            if size > s {
                if self.verbose {
                    println!("{:?} size {} exceeds size threshold {}, rotating.", path, size, s);
                }
                return Ok(true);
            }
        }

        if let Some(mins) = config.minsize {
            if size > mins && time_rotate {
                if self.verbose {
                    println!("{:?} size {} exceeds minsize {} and time threshold met, rotating.", path, size, mins);
                }
                return Ok(true);
            }
        }

        if let Some(maxs) = config.maxsize {
            if size > maxs || time_rotate {
                if self.verbose {
                    println!("{:?} size {} exceeds maxsize {} or time threshold met, rotating.", path, size, maxs);
                }
                return Ok(true);
            }
        }

        if time_rotate {
            if self.verbose {
                println!("{:?} time threshold met, rotating.", path);
            }
            return Ok(true);
        }

        Ok(false)
    }

    fn rotate_single_file(&mut self, path: &Path, config: &LogConfig) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        if self.dry_run {
            println!("{} {:?}", "Would rotate".yellow().bold(), path);
            self.state_db.update(&path_str);
            return Ok(());
        }

        let dest_base = if let Some(ref olddir) = config.olddir {
            let p = Path::new(olddir);
            let absolute = if p.is_absolute() {
                p.to_path_buf()
            } else {
                path.parent().unwrap_or_else(|| Path::new("")).join(p)
            };
            fs::create_dir_all(&absolute).ok();
            absolute
        } else {
            path.parent().unwrap_or_else(|| Path::new("")).to_path_buf()
        };

        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        if config.dateext == Some(true) {
            let fmt = config.dateformat.as_deref().unwrap_or("-%Y%m%d");
            let date_str = Local::now().format(fmt).to_string();
            let dest_name = format!("{}{}", file_name, date_str);
            let mut dest_path = dest_base.join(&dest_name);

            if config.compress == Some(true) && config.delaycompress != Some(true) {
                dest_path.set_extension(format!("{}{}", dest_path.extension().unwrap_or_default().to_string_lossy(), ".gz"));
            }

            if config.copytruncate == Some(true) {
                fs::copy(path, &dest_path)?;
                let file = fs::OpenOptions::new().write(true).truncate(true).open(path)?;
                file.set_len(0)?;
            } else {
                fs::rename(path, &dest_path)?;
                self.create_new_log(path, config)?;
            }

            if config.compress == Some(true) && config.delaycompress != Some(true) {
                let uncompressed = dest_base.join(&dest_name);
                compress_file(&uncompressed, &dest_path)?;
                fs::remove_file(&uncompressed)?;
            }
        } else {
            let max_rotate = config.rotate.unwrap_or(0);
            if max_rotate > 0 {
                let oldest_path = self.get_rotated_path(&dest_base, &file_name, max_rotate, config);
                if oldest_path.exists() {
                    fs::remove_file(&oldest_path)?;
                }

                for i in (1..max_rotate).rev() {
                    let src = self.get_rotated_path(&dest_base, &file_name, i, config);
                    let dest = self.get_rotated_path(&dest_base, &file_name, i + 1, config);
                    if src.exists() {
                        fs::rename(&src, &dest)?;
                    }
                }

                let dest_path_1 = self.get_rotated_path(&dest_base, &file_name, 1, config);

                if config.copytruncate == Some(true) {
                    let copy_dest = if config.compress == Some(true) && config.delaycompress != Some(true) {
                        dest_base.join(format!("{}.1", file_name))
                    } else {
                        dest_path_1.clone()
                    };
                    fs::copy(path, &copy_dest)?;
                    let file = fs::OpenOptions::new().write(true).truncate(true).open(path)?;
                    file.set_len(0)?;

                    if config.compress == Some(true) && config.delaycompress != Some(true) {
                        compress_file(&copy_dest, &dest_path_1)?;
                        fs::remove_file(&copy_dest)?;
                    }
                } else {
                    let rename_dest = if config.compress == Some(true) && config.delaycompress != Some(true) {
                        dest_base.join(format!("{}.1", file_name))
                    } else {
                        dest_path_1.clone()
                    };

                    fs::rename(path, &rename_dest)?;
                    self.create_new_log(path, config)?;

                    if config.compress == Some(true) && config.delaycompress != Some(true) {
                        compress_file(&rename_dest, &dest_path_1)?;
                        fs::remove_file(&rename_dest)?;
                    }
                }

                if config.compress == Some(true) && config.delaycompress == Some(true) {
                    let second_path = self.get_rotated_path(&dest_base, &file_name, 2, config);
                    let second_path_uncompressed = dest_base.join(format!("{}.2", file_name));

                    if second_path_uncompressed.exists() {
                        compress_file(&second_path_uncompressed, &second_path)?;
                        fs::remove_file(&second_path_uncompressed)?;
                    }
                }
            }
        }

        self.state_db.update(&path_str);
        Ok(())
    }

    fn get_rotated_path(&self, dest_base: &Path, file_name: &str, index: u32, config: &LogConfig) -> PathBuf {
        let is_compressed = config.compress == Some(true);
        let is_delayed = config.delaycompress == Some(true);

        if is_compressed {
            if is_delayed && index == 1 {
                dest_base.join(format!("{}.{}", file_name, index))
            } else {
                dest_base.join(format!("{}.{}.gz", file_name, index))
            }
        } else {
            dest_base.join(format!("{}.{}", file_name, index))
        }
    }

    fn create_new_log(&self, path: &Path, config: &LogConfig) -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)?;
        file.set_len(0)?;

        if let Some(ref create_val) = config.create {
            let parts: Vec<&str> = create_val.split_whitespace().collect();
            if !parts.is_empty() {
            }
        }
        Ok(())
    }

    fn run_script(&self, script: &str, args: &str) -> Result<()> {
        if self.dry_run {
            println!("{} script: {}", "Would run".yellow().bold(), script);
            return Ok(());
        }

        if self.verbose {
            println!("Running script: {}", script);
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("{} {}", script, args))
            .output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!(
                        "Script failed with exit code: {:?}",
                        out.status.code()
                    );
                    if !out.stderr.is_empty() {
                        eprintln!("Error output: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::Error::new(e).context("Failed to execute script"));
            }
        }

        Ok(())
    }
}
