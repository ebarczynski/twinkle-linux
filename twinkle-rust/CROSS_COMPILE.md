# Cross-Compiling Twinkle Linux for Linux from macOS

This guide explains how to cross-compile the Rust version of Twinkle Linux for Linux targets while developing on macOS.

## Overview

Cross-compilation allows you to build Linux binaries on macOS using Docker and the [`cross`](https://github.com/cross-rs/cross) tool. This is particularly useful for:

- Building release binaries for Linux distribution
- Testing on multiple Linux architectures (x86_64, ARM64, ARMv7)
- CI/CD pipelines
- Developing on macOS while targeting Linux

## Prerequisites

### 1. Install Docker Desktop

Download and install [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop/).

Start Docker Desktop and ensure it's running:

```bash
docker info
```

### 2. Install Rust Toolchain

If you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 3. Install Cross

Install the `cross` tool:

```bash
cargo install cross
```

### 4. Install Linux Target

Add the Linux target you want to build for:

```bash
# For standard Linux (x86_64, glibc)
rustup target add x86_64-unknown-linux-gnu

# For musl-based Linux (static linking)
rustup target add x86_64-unknown-linux-musl

# For ARM64 Linux (Raspberry Pi 4, ARM servers)
rustup target add aarch64-unknown-linux-gnu

# For ARMv7 Linux (Raspberry Pi 3/Zero)
rustup target add armv7-unknown-linux-gnueabihf
```

## Quick Start

### Using the Build Script

The easiest way to cross-compile is using the provided [`cross-build.sh`](cross-build.sh) script:

```bash
# Build for standard Linux (x86_64)
./cross-build.sh x86_64-unknown-linux-gnu release

# Build for ARM64 (Raspberry Pi 4)
./cross-build.sh aarch64-unknown-linux-gnu release

# Build and package
./cross-build.sh x86_64-unknown-linux-gnu package
```

### Using Cross Directly

You can also use `cross` directly:

```bash
# Build release binary
cross build --release --target x86_64-unknown-linux-gnu

# Run tests
cross test --target x86_64-unknown-linux-gnu
```

## Configuration

### `.cross.toml`

The [`.cross.toml`](.cross.toml) file configures cross-compilation targets:

```toml
[target.x86_64-unknown-linux-gnu]
dockerfile = "Dockerfile.cross"

[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"
```

### `Dockerfile.cross`

The [`Dockerfile.cross`](Dockerfile.cross) defines the Docker image used for cross-compilation. It includes:

- GTK4 development libraries
- Related dependencies (GLib, Pango, Cairo, etc.)
- pkg-config for library discovery

## Supported Targets

| Target | Description | Use Case |
|--------|-------------|-----------|
| `x86_64-unknown-linux-gnu` | Standard Linux (glibc) | Most Linux distributions |
| `x86_64-unknown-linux-musl` | Linux with musl (static) | Alpine Linux, containers |
| `aarch64-unknown-linux-gnu` | ARM64 Linux | Raspberry Pi 4, ARM servers |
| `armv7-unknown-linux-gnueabihf` | ARMv7 Linux | Raspberry Pi 3/Zero |

## Building for Specific Targets

### Standard Linux (x86_64)

```bash
# Build release binary
cross build --release --target x86_64-unknown-linux-gnu

# Binary location
target/x86_64-unknown-linux-gnu/release/twinkle-linux
```

### ARM64 Linux (Raspberry Pi 4)

```bash
# Build release binary
cross build --release --target aarch64-unknown-linux-gnu

# Binary location
target/aarch64-unknown-linux-gnu/release/twinkle-linux
```

### ARMv7 Linux (Raspberry Pi 3/Zero)

```bash
# Build release binary
cross build --release --target armv7-unknown-linux-gnueabihf

# Binary location
target/armv7-unknown-linux-gnueabihf/release/twinkle-linux
```

## Packaging

The build script can create tarball packages:

```bash
./cross-build.sh x86_64-unknown-linux-gnu package
```

This creates: `dist/twinkle-linux-x86_64-unknown-linux-gnu.tar.gz`

## Troubleshooting

### Docker Not Running

```
Error: Docker is not running
```

**Solution**: Start Docker Desktop and wait for it to fully initialize.

### Cross Not Installed

```
Error: 'cross' is not installed
```

**Solution**: Install cross with `cargo install cross`

### GTK4 Build Errors

If you encounter GTK4-related build errors:

1. Ensure the Dockerfile includes all GTK4 dependencies
2. Rebuild the Docker image:
   ```bash
   docker build -f Dockerfile.cross -t twinkle-cross:x86_64 .
   ```
3. Clear cross cache:
   ```bash
   rm -rf ~/.cargo/registry/cache
   ```

### Permission Denied

```
bash: ./cross-build.sh: Permission denied
```

**Solution**: Make the script executable:
```bash
chmod +x cross-build.sh
```

## Limitations

1. **GTK4 Dependencies**: Cross-compiling GTK4 applications is complex due to native library dependencies. The Dockerfile must include all required development libraries.

2. **Testing**: Cross-compiled binaries cannot be run on macOS. You must transfer them to a Linux system for testing.

3. **Debugging**: Debugging cross-compiled binaries requires a Linux environment.

4. **Build Time**: First-time builds can be slow due to Docker image downloads and dependency compilation.

## Alternative Approaches

If cross-compilation proves difficult, consider these alternatives:

### 1. GitHub Actions

Use GitHub Actions to build Linux binaries automatically:

```yaml
name: Build Linux

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
```

### 2. Linux VM

Use a Linux virtual machine (VirtualBox, VMware, UTM) for native builds.

### 3. Cloud Build

Use cloud build services like:
- [GitHub Actions](https://github.com/features/actions)
- [GitLab CI](https://docs.gitlab.com/ee/ci/)
- [CircleCI](https://circleci.com/)

### 4. Remote Development

Use remote development tools:
- [SSH](https://www.openssh.com/) to a Linux machine
- [Visual Studio Code Remote - SSH](https://code.visualstudio.com/docs/remote/ssh)
- [GitHub Codespaces](https://github.com/features/codespaces)

## Resources

- [cross-rs GitHub](https://github.com/cross-rs/cross)
- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [GTK4 Documentation](https://docs.gtk.org/gtk4/)
- [Docker Desktop for Mac](https://www.docker.com/products/docker-desktop)

## Contributing

When contributing to Twinkle Linux, please:

1. Test cross-compilation before submitting PRs
2. Update this documentation if you add new targets
3. Ensure the Dockerfile includes all necessary dependencies
4. Test binaries on actual Linux hardware when possible
