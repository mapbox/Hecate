# Contributing

## Merging Pull Requests

All PRs to master _must_ have a corresponding versioned release

- add CHANGELOG.md comment following current formatting
- Bump the version in Cargo.toml
- Bump the version in `src/lib.rs` (Top of file)
- Build a linux release from a linux machine (`cargo build --release`)
- `git commit -am "v#.#.#`
- `git tag v#.#.#`
- `git push`
- `git push --tags`
- `cargo publish` (Optionally)
- Via the github interface create a new release and upload the hecate binary
