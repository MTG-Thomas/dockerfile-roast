# Dockerfile Roast agent guide

This Rust project provides the `droast` CLI and library, with WASM, editor integrations, and a GitHub Action. Read [README.md](README.md), [Cargo.toml](Cargo.toml), and the relevant implementation in `src/`; use [DOCS.md](DOCS.md) for configuration behavior.

## Verification

The release workflow uses `cargo test --locked` and `cargo publish --dry-run --locked`. Run the focused parser/rule regression tests for code changes and the locked suite before publication. The package dry run is a packaging check, not authorization to publish.

For relevant integrations, follow their workflow commands: `scripts/test-neovim-extension.sh` for Neovim and `./scripts/update-public-metadata.sh --check` for generated release examples. Browser WASM, WASI, Podman, BuildKit differential, and editor packaging have separate workflows and dependencies; select those matching the changed surface rather than treating a native build as proof of every target.

## Compatibility and release boundaries

Preserve stable rule IDs, severity and suppression semantics, machine-readable JSON/SARIF/GitHub output, source spans, CLI-over-file precedence, and Docker versus Podman ignore-file behavior. Add representative regression fixtures when changing parser acceptance or context discovery.

Native-only dependencies are separated from browser WASM in Cargo; maintain that split. Keep release tag, crate version, generated README examples, action metadata, and packaged assets consistent through the existing scripts.

This is an MTG-used fork. Upstream registry, marketplace, website, and Homebrew destinations remain in its files: inspect the exact destination before publishing or changing those workflows. Do not assume fork tags authorize publishing upstream artifacts. Preserve license attribution and document any intentional divergence.
