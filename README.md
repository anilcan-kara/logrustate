# logrustate

A modern, drop-in replacement for `logrotate` — written in Rust.

`logrustate` is designed to be fully compatible with the traditional logrotate configuration format, while bringing modern features like TOML configuration support, better performance, safety, and clear output formatting.

## Why logrustate?

- **Fast & Safe** — Built in Rust, offering memory safety and high performance.
- **Drop-in Compatible** — Supports legacy `logrotate.conf` configuration syntax.
- **TOML Support** — Allows writing structured configurations in modern TOML.
- **Robust Suffix Management** — Supports both numeric and custom date-based suffix names (`dateext`).
- **Gzip Compression** — Built-in fast gzip compression (`flate2`).
- **Dry-run Mode** — Colorized preview output of what actions would be performed.

## Installation

### 1. From Source (Cargo)
```bash
cargo install --git https://github.com/anilcan-kara/logrustate.git
```

### 2. Direct Binary Download
You can download the precompiled static binary for your platform directly from the GitHub Release assets:
- 💻 **Windows (x64)**: [logrustate-win32-x64.exe](https://github.com/anilcan-kara/logrustate/releases/download/v0.1.2/logrustate-win32-x64.exe)
- 🐧 **Linux (x64)**: [logrustate-linux-x64](https://github.com/anilcan-kara/logrustate/releases/download/v0.1.2/logrustate-linux-x64)
- 🐧 **Linux (ARM64)**: [logrustate-linux-arm64](https://github.com/anilcan-kara/logrustate/releases/download/v0.1.2/logrustate-linux-arm64)
- 🍎 **macOS (x64)**: [logrustate-darwin-x64](https://github.com/anilcan-kara/logrustate/releases/download/v0.1.2/logrustate-darwin-x64)
- 🍎 **macOS (ARM64)**: [logrustate-darwin-arm64](https://github.com/anilcan-kara/logrustate/releases/download/v0.1.2/logrustate-darwin-arm64)

## CLI Usage

```bash
logrustate [FLAGS] [OPTIONS] <CONFIG_FILE>...
```

### Flags & Options

- `-d, --debug` — Dry-run mode. Implies `--verbose` and does not modify any files.
- `-v, --verbose` — Print details during processing.
- `-f, --force` — Force rotation of all logs, even if not yet scheduled.
- `-s, --state <PATH>` — Path to the state file (defaults to `/var/lib/logrustate/status`).

## Configuration Formats

### 1. Legacy Syntax (`logrotate.conf` compatible)

```text
/var/log/nginx/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    sharedscripts
    postrotate
        systemctl reload nginx
    endscript
}
```

### 2. Modern TOML Syntax

Create a configuration file with the `.toml` extension:

```toml
[global]
rotate = 4
compress = true

[[entries]]
paths = ["/var/log/app/*.log"]
frequency = "daily"
copytruncate = true
```

## Systemd Integration

You can schedule `logrustate` using a Systemd timer.

### Service File (`/etc/systemd/system/logrustate.service`)

```ini
[Unit]
Description=Rotate log files
Documentation=https://github.com/anilcan-kara/logrustate

[Service]
Type=oneshot
ExecStart=/usr/local/bin/logrustate /etc/logrustate.conf
```

### Timer File (`/etc/systemd/system/logrustate.timer`)

```ini
[Unit]
Description=Daily rotation of log files

[Timer]
OnCalendar=daily
AccuracySec=1h
Persistent=true

[Install]
WantedBy=timers.target
```

## License

This project is licensed under the MIT License.
