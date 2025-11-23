# RustSpy

Phase 1 Scaffolding.

## Building

To build the project, you need a Rust toolchain (2021 edition).

```bash
cargo build
```

## Running

```bash
cargo run
```

## Linting

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Packaging

To generate an AppImage, ensure you have `wget` and `file` installed. The build script will handle downloading `linuxdeploy`.

Run the following command:

```bash
./build_deploy.sh
```

The resulting AppImage will be located in the `dist/` directory.

### Dependencies

- Rust (latest stable)
- wget
- file
- fuse (optional, only needed if running the AppImage, building uses fallback)
