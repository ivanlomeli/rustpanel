# RustPanel

**RustPanel** is an open-source, high-performance hosting control panel built in Rust. It aims to be a secure, memory-efficient, and modern alternative to cPanel.

## Vision

- **Performance:** Native speed and low footprint using Rust.
- **Security:** Memory safety guarantees.
- **Open Source:** MIT Licensed. Community driven.
- **Modern:** API-first architecture.

## Stack (Planned)

- **Core:** Rust
- **Web Framework:** Axum
- **Frontend:** React/TypeScript
- **Database:** SQLite (Implemented via SQLx)
- **Security:** Bcrypt hashing + JWT Auth
- **System:** Systemd integration & sysinfo

## Getting Started

1. The system creates a default admin user on first run:
   - User: `admin`
   - Pass: `password`

2. Run the backend:
```bash
cargo run
```
