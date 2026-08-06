<div align="center">

<h1>logrustate</h1>

<p><strong>A modern, drop-in replacement for logrotate — written in Rust.</strong></p>

[![Crates.io](https://img.shields.io/crates/v/logrustate?style=flat-square&color=fc8d62)](https://crates.io/crates/logrustate)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Build](https://img.shields.io/github/actions/workflow/status/anilcan-kara/logrustate/release.yml?style=flat-square)](https://github.com/anilcan-kara/logrustate/actions)
[![GitHub Release](https://img.shields.io/github/v/release/anilcan-kara/logrustate?style=flat-square&color=8be04e)](https://github.com/anilcan-kara/logrustate/releases)

```bash
logrustate /etc/logrustate.toml --verbose
```

</div>

---

## Why logrustate?

`logrotate` has been the de-facto log rotation tool on Linux since **1996**. It works — but it was written in C, relies on `crond`, has a confusing config format, no dry-run mode, and cryptic errors.

**logrustate** is a modern replacement with a clean TOML config, verbose output, proper dry-run, and full compatibility with legacy `logrotate.conf` syntax.

| | logrustate 🦀 | logrotate |
|---|---|---|
| Language | Rust | C |
| Config format | TOML (+ legacy logrotate.conf) | logrotate.conf |
| Dry-run mode | `--debug` | `--debug` (partial) |
| State tracking | Per-file JSON state | `/var/lib/logrotate/status` |
| Verbose output | Full colored output | Minimal |
| Memory safety | ✓ (Rust) | C (manual) |
| Install | Single binary | Package manager |

---

## Features

- 🔄 **Drop-in compatible** — reads existing `logrotate.conf` and `/etc/logrotate.d/` configs
- 📄 **Modern TOML config** — cleaner, more readable alternative config format
- 🐛 **Debug mode** — full dry-run with verbose output, no files touched
- 🗜️ **Compression** — gzip, bzip2, xz support
- 📊 **State tracking** — tracks last rotation time per log file
- 🎨 **Colored output** — human-readable progress and error messages
- ⚡ **Fast** — native binary, no interpreter overhead
- 🔒 **Safe** — written in Rust, no unsafe memory operations

---

## Quick Start

### 1. Create a TOML config
```toml
# /etc/logrustate.toml

[[logs]]
path = "/var/log/nginx/access.log"
rotate = 7          # keep 7 rotated files
compress = true     # gzip after rotation
daily = true        # rotate daily
missingok = true    # don't error if file is missing
notifempty = true   # skip rotation if log is empty

[[logs]]
path = "/var/log/myapp/*.log"
rotate = 14
compress = true
weekly = true
postrotate = "systemctl reload myapp"
```

### 2. Run logrustate
```bash
logrustate /etc/logrustate.toml
```

### 3. Preview without changes (dry-run)
```bash
logrustate /etc/logrustate.toml --debug
```

---

## CLI Reference

```
logrustate [OPTIONS] <CONFIG>...

Arguments:
  <CONFIG>...    Path to configuration file(s) — TOML or logrotate.conf format

Options:
  -d, --debug     Debug mode: verbose output + dry-run (no files modified)
  -v, --verbose   Verbose mode: print details during processing
  -f, --force     Force rotation of all logs, even if not yet scheduled
  -s, --state     State file path [default: /var/lib/logrustate/status]
  -h, --help      Print help
  -V, --version   Print version
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All PRs welcome.

---

## License

MIT © [Anilcan Kara](https://github.com/anilcan-kara)
