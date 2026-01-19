# Contributing to RustPanel

Thank you for your interest in contributing to RustPanel! We welcome contributions from everyone.

## Getting Started

1.  **Fork** the repository on GitHub.
2.  **Clone** your fork locally.
3.  Create a new **branch** for your feature or bugfix (`git checkout -b feature/amazing-feature`).

## Development Setup

### Backend (Rust)
```bash
cargo run
```

### Frontend (React)
```bash
cd ui
npm install
npm run dev
```

## Pull Request Process

1.  Ensure your code builds and tests pass (`cargo test`).
2.  Format your code using `cargo fmt`.
3.  Update the documentation if you change any public API.
4.  Submit a Pull Request to the `main` branch.

## Code Style

*   **Rust:** We follow standard Rust formatting (`cargo fmt`).
*   **Commits:** Please write clear, concise commit messages (e.g., `feat: add disk usage monitoring`).

## Reporting Bugs

Please use the GitHub Issues tab to report bugs. Include:
*   Your operating system.
*   Steps to reproduce the error.
*   Expected vs. actual behavior.
