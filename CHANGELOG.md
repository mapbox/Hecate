# CHANGELOG

## Emoji Cheatsheet
- :pencil2: doc updates
- :bug: when fixing a bug
- :rocket: when making general improvements
- :white_check_mark: when adding tests
- :arrow_up: when upgrading dependencies
- :tada: when adding new features

## Version History

## Future Releases

- Authentication via http basic & scoped tokens

- Ways
    - [x] Ways can be created
    - [x] Ways can be downloaded
    - [x] Ways tags can be modified
    - [ ] Ways can be moved
        - moves only have the individual nodes uploaded in the delta
        - resulting in the changes not actually affecting the way as the nodeids aren't tracked
- Relations
    - [ ] Everything...

## v0.1.3

- :rocket: Add CLI Options

## v0.1.2

- :bug: Track package.json & Cargo.toml

## v0.1.1

- :tada: `409 CONFLICT` errors are now thrown when a changeset is closed that the caller tries to access/append to

## v0.1.0

- :rocket: The first general release
  - Basic OSMXML support for points & sketchy line/simply poly suport
  - Full delta/geojson support
  - opening/committing/finalizing deltas
  - Pretty thorough suite of JS tests for JS integration
  - Basic Rust tests for node/way/rel objects

