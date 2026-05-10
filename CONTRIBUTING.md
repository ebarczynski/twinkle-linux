# Contributing to Twinkle Linux

Thank you for contributing to Twinkle Linux! This guide covers everything you need to get started.

## How to Contribute

1. **Fork** the repository at [github.com/ebarczynski/twinkle-linux](https://github.com/ebarczynski/twinkle-linux)
2. **Clone** your fork and add the upstream remote:
   ```bash
   git clone https://github.com/YOUR_USERNAME/twinkle-linux.git
   cd twinkle-linux
   git remote add upstream https://github.com/ebarczynski/twinkle-linux.git
   ```
3. **Create a branch** for your change:
   ```bash
   git checkout -b feat/your-feature-name
   ```
4. **Make your changes**, commit, and push to your fork
5. **Open a Pull Request** against the `main` branch

## Development Setup

### Prerequisites

- **Rust toolchain** (install via [rustup](https://rustup.rs)):
  ```bash
  rustup toolchain install stable
  ```
- **GTK4 development libraries** (required for the GUI):
  ```bash
  # Ubuntu/Debian
  sudo apt install libgtk-4-dev libadwaita-1-dev

  # Fedora
  sudo dnf install gtk4-devel libadwaita-devel

  # Arch Linux
  sudo pacman -S gtk4 libadwaita
  ```
- **ddcutil** (for DDC/CI monitor communication):
  ```bash
  # Ubuntu/Debian
  sudo apt install ddcutil i2c-tools

  # Fedora
  sudo dnf install ddcutil i2c-tools

  # Arch Linux
  sudo pacman -S ddcutil i2c-tools
  ```

### Project Structure

```
twinkle-linux/
├── twinkle-rust/          # Primary Rust implementation
│   ├── src/               # Application source
│   ├── Cargo.toml         # Rust package manifest
│   └── ...
├── twinkle-cpp/           # Future C++23/26 implementation
│   └── ...
├── CONTRIBUTING.md
├── LICENSE                # MIT
└── README.md
```

## Building & Testing

All commands run from `twinkle-rust/`:

```bash
cd twinkle-rust

# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Code Style

We rely on the standard Rust tooling — no custom config needed:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint
cargo clippy

# Treat clippy warnings as errors (CI does this)
cargo clippy -- -D warnings
```

### Guidelines

- Follow idiomatic Rust conventions (see [The Book](https://doc.rust-lang.org/book/))
- Use `cargo clippy` and fix all warnings before submitting
- Keep public API documented with `///` doc comments
- Prefer `Result` over `unwrap`/`expect` in library code

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`

**Scopes:** `ui`, `ddc`, `config`, `cpp`, or omit for general changes.

Examples:

```
feat(ui): add brightness slider widget
fix(ddc): handle monitor timeout errors
docs(readme): update build instructions
```

## Pull Request Process

Before submitting a PR:

1. **Rebase** on upstream `main`:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```
2. **Run checks** and fix any issues:
   ```bash
   cd twinkle-rust
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
3. **Update documentation** if your change affects user-facing behavior

### PR Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` reports no warnings
- [ ] `cargo test` passes
- [ ] New features include tests
- [ ] Commit messages follow conventional commits
- [ ] No merge conflicts with `main`

## Adding Features to twinkle-cpp

The `twinkle-cpp/` directory hosts a future C++23/26 implementation that mirrors the architecture of `twinkle-rust/`. When adding a feature:

1. **Implement in Rust first** (`twinkle-rust/`) — this is the primary implementation
2. **Add a corresponding C++ implementation** in `twinkle-cpp/` following the same module structure
3. Keep the C++ version API-compatible with the Rust version where practical
4. C++ code should target C++23/26 and follow modern idioms (smart pointers, `std::expected`, concepts, modules where applicable)

The C++ implementation is a work in progress. Contributions there are welcome but should track the Rust architecture.

## Reporting Issues

When filing a bug report, include:

- OS and version
- Rust version (`rustc --version`)
- ddcutil version (`ddcutil --version`)
- Steps to reproduce
- Expected vs. actual behavior
- Relevant log output

File issues at [GitHub Issues](https://github.com/ebarczynski/twinkle-linux/issues).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
