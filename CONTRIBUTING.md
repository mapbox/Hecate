# Contributing

## 1. Merging Pull Requests

:warning: All PRs to master **_must_** have a corresponding versioned release

- Add a `CHANGELOG.md` comment following current formatting, commit and push it.

### Manual way

- Bump the version in Cargo.toml
- Bump the version in `src/lib.rs` (Top of file)
- Bump the version in `src/cli.yml`
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

## 2. Finalize release

CI will create a new prerelease and upload a binary for you. You can finalize this release by:

- Finding your release at https://github.com/mapbox/Hecate/releases
- Add as title the new released version. _i.e. `v0.71.1`_
- Copy your comment in the `CHANGELOG.md` and paste it in the description field.
- Uncheck `This is a prerelease` if necessary.
- Click on the `update release button`
