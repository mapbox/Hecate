# Contributing

## 1. Merging Pull Requests

:warning: All PRs to master **_must_** have a corresponding versioned release

- Add a `CHANGELOG.md` comment following current formatting, commit and push it.

### Manual way

- Bump the version in Cargo.toml
- Bump the version in `src/lib.rs` (Top of file)
- Bump the version in `src/cli.yml`
- Build a Linux release from a Linux machine (`cargo build --release`)
- `git commit -am "v#.#.#`
- `git tag v#.#.#`
- `git push`
- `git push --tags`
- `cargo publish` (Optionally)

### Automatic way

- Run the `release script` without any argument to get the current version.

```
./release
```

- Run the release script with the first argument being the new version you intend to release

```
./release 0.71.1
```

and you should get a comment like:
```
ok - 0.71.0 => 0.71.1
ok - release pushed!
```

## 2. Create a new release

Via the GitHub interface create a new release and upload the Hecate binary

- Go to the [Hecate GitHub UI](https://github.com/mapbox/Hecate) and click in `releases`
- Click on the version you just released, it will look like `v0.71.1`
- In the publish release UI in Github
    - Add as title the new released version. _i.e. `v0.71.1`_
    - Copy your comment in the `CHANGELOG.md` and paste it in the description field.
    - Upload the binary from you local machine `~/Hecate/target/release/hecate`
    - Click on the `publish release button`
