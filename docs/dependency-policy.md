# Dependency Policy

Mio-GUI is an application-driven framework under active architectural development. Direct dependencies are pinned to exact versions in `Cargo.toml`, and `Cargo.lock` is committed.

Dependency upgrades are deliberate changes. Each upgrade requires:

- Reading upstream migration and release notes
- Compiling every target
- Running strict Clippy
- Running CPU golden tests
- Running GPU comparison tests on available backends
- Repeating relevant manual visual checks when rendering changes

Transitive versions are controlled by `Cargo.lock`. Security updates that require dependency movement follow the same verification gate rather than bypassing renderer tests.

The minimum supported Rust version is 1.87. CI and local verification use the toolchain declared in `rust-toolchain.toml`.
