# AGENTS.md

## Project

- `extern-trait` is a Rust workspace containing the public crate and its proc-macro implementation.

## Commands

| Task          | Command                                                    |
| ------------- | ---------------------------------------------------------- |
| Build         | `cargo build`                                              |
| Test all      | `cargo test --all-features`                                |
| Test UI suite | `cargo test --test ui`                                     |
| Format check  | `cargo +nightly fmt --all --check`                         |
| Lint          | `cargo clippy --all-targets --all-features -- -D warnings` |
| Docs          | `cargo doc --all-features --no-deps`                       |

## Cross-target Testing

- CI uses `cross` for non-host targets; use it when a change may be target-sensitive.
- Example: `cross test --all-features --target <target> --verbose`.
- Do not run the cross matrix after every edit; prefer focused local checks first.

## Structure

- `src/`: public crate surface.
- `impl/`: proc-macro implementation crate.
- `tests/`: integration and trybuild UI tests.

## Rules

- Preserve the workspace split: public API changes usually need matching macro implementation and integration tests.
- Use the nightly toolchain for rustfmt; `rustfmt.toml` enables unstable formatting options.
- Use Conventional Commits: `<type>(<scope>): <summary>`. Omit `(scope)` when none.
- Accepted commit types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- Keep commit summaries lowercase, imperative, and under 72 characters; omit the trailing period.
- For breaking changes, use `!` before the colon and add a `BREAKING CHANGE:` footer.
