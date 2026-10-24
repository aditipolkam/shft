# Contributing to shft

Thanks for your interest in improving `shft`. Bug reports, feature requests and pull requests are all welcome.

## Reporting bugs and requesting features

Open an issue using the bug report or feature request template. For bugs, include the exact command you ran, the output, your OS, and `shft --version`. If the problem depends on a specific image, attach it or describe its format, dimensions and whether it has transparency.

Please report security vulnerabilities privately as described in [SECURITY.md](SECURITY.md), not in a public issue.

## Development setup

You need Rust 1.88 or newer (the MSRV declared in `Cargo.toml`).

```bash
git clone https://github.com/aditipolkam/shft
cd shft
cargo build
cargo test
```

## Before opening a pull request

CI runs these checks on every pull request to `main`, so run them locally first:

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

Tests run on Linux, macOS and Windows, and the crate is also checked against the MSRV.

## Guidelines

- Keep changes focused; one logical change per pull request.
- Add or update tests for behaviour changes.
- Update `README.md` when you change CLI flags or user-visible behaviour, and add an entry under `Unreleased` in `CHANGELOG.md`.
- `shft` is offline and safe by default: it must never make network calls, shell out to external media tools, or overwrite an existing file without `--overwrite`.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `ci:`, ...).

By contributing, you agree that your contributions are licensed under the [MIT license](LICENSE-MIT).
