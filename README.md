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

---

## Installation

### ⚡ Quick Install (Linux / macOS)
```bash
curl -fsSL https://raw.githubusercontent.com/anilcan-kara/logrustate/master/install.sh | sh
```

### Cargo
```bash
cargo install logrustate
```

---

## Quick Start

### Create a TOML config
```toml
# /etc/logrustate.toml

[[logs]]
path = "/var/log/nginx/access.log"
rotate = 7          # keep 7 rotated files
compress = true     # gzip after rotation
daily = true        # rotate daily
missingok = true    # don't error if file is missing
notifempty = true   # skip rotation if log is empty
```

### Preview without changes (dry-run)
```bash
logrustate /etc/logrustate.toml --debug
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). All PRs welcome.

---

## License

MIT © [Anilcan Kara](https://github.com/anilcan-kara)
