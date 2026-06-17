mod config;
mod state;
mod compress;
mod rotator;

use std::path::PathBuf;
use std::process;
use clap::Parser;
use colored::*;

#[derive(Parser, Debug)]
#[command(
    name = "logrustate",
    version,
    about = "A modern, drop-in replacement for logrotate — written in Rust",
    long_about = "logrustate is a modern replacement for the 30-year-old logrotate tool. It supports both legacy logrotate.conf syntax and modern TOML configuration files."
)]
struct Args {
    #[arg(help = "Path to the configuration file(s)", required = true)]
    configs: Vec<String>,

    #[arg(short = 'd', long = "debug", help = "Debug mode: implies --verbose, does not modify any files (dry-run)")]
    debug: bool,

    #[arg(short = 'v', long = "verbose", help = "Verbose mode: print details during processing")]
    verbose: bool,

    #[arg(short = 'f', long = "force", help = "Force rotation of all logs, even if not yet scheduled")]
    force: bool,

    #[arg(short = 's', long = "state", help = "Path to the state file", default_value = "/var/lib/logrustate/status")]
    state_file: String,
}

fn main() {
    let args = Args::parse();

    let dry_run = args.debug;
    let verbose = args.verbose || args.debug;

    if verbose {
        println!("{}", "Starting logrustate...".green().bold());
        println!("State file: {}", args.state_file);
        println!("Force rotation: {}", args.force);
        println!("Dry-run mode: {}", dry_run);
    }

    let mut state_db = match state::StateDB::load(&args.state_file) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("{}: {:?}", "Error loading state file".red().bold(), e);
            process::exit(1);
        }
    };

    let mut rotator = rotator::Rotator::new(&mut state_db, dry_run, verbose, args.force);

    for config_path_str in &args.configs {
        let path = std::path::Path::new(config_path_str);
        if verbose {
            println!("Parsing configuration file: {:?}", path);
        }

        let entries = match config::parse_config_file(path) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: {:?} -> {:?}", "Error parsing configuration".red().bold(), path, e);
                process::exit(1);
            }
        };

        if verbose {
            println!("Loaded {} log rotation blocks.", entries.len());
        }

        for entry in &entries {
            if let Err(e) = rotator.process_entry(entry) {
                eprintln!("{}: {:?}", "Error rotating logs".red().bold(), e);
            }
        }
    }

    if !dry_run {
        if let Err(e) = state_db.save() {
            eprintln!("{}: {:?}", "Error saving state file".red().bold(), e);
            process::exit(1);
        }
    } else if verbose {
        println!("{}", "Dry-run complete. State file was not saved.".yellow());
    }

    if verbose {
        println!("{}", "logrustate finished successfully.".green().bold());
    }
}
