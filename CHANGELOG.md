# CHANGELOG

## Emoji Cheatsheet
- :pencil2: doc updates
- :bug: when fixing a bug
- :rocket: when making general improvements
- :white_check_mark: when adding tests
- :arrow_up: when upgrading dependencies
- :tada: when adding new features

## Version History

## v0.15.0

- :tada: Add endpoint for manually regenerating Mapbox Vector Tiles

## v0.14.0

- :tada: Add mapbox-gl-js style related endpoints

## v0.13.0

- :rocket: Rewrite `bounds` endpoint to use in-memory streams

## v0.12.2

- :bug: Fix Web UI timezone bug

## v0.12.1

- :bug: Unescape XML chars when uploading via OSMXML shim

## v0.12.0

- :rocket: Add support for JSON in OSM XML
- :tada: Add schema endpoint

## v0.11.0

- :rocket: Add time based TileCache for faster map rendering

## v0.10.1

- :rocket: Better visualization of Arrays & Object properties in web UI

## v0.10.0

- :rocket: `hecate::start` is now exposed via lib. Note thta it is blocking, per the rocket docs
- :white_check_mark: Each test now creates and manages its own server instance

## v0.9.0

- :rocket: Add JSON Schema Validation

## v0.8.0

- :rocket: Add login fxn to web UI
- :rocket: Add register fxn to web UI
- :tada: Add user info endpoint
- :tada: Add user session endpoint
- :rocket: Refactor Auth object to support Basic or Cookie auth

## v0.7.3

- :rocket: Allow downloading bounds files via web UI

## v0.7.2

- :rocket: Improvements to the web UI

## v0.7.1

- :rocket: View list of bounds via the web UI

## v0.7.0

- :white_check_mark: Rewrite all JS tests in pure rust w/ reqwest
- :rocket: Limit number of features in MVT at low zooms

## v0.6.1

- :bug: Use the `location.host` prop for all API calls in admin interface instead of hardcoded port

## v0.6.0

- :tada: Delta APIs
- :tada: Usable Web API

## v0.5.0

- :rocket: Add support for `multipoint`, `polygon`, `multilinestrint`, `multipolygon` via XML download shim

## v0.4.0

- :tada: Service Vector Tiles directly from data for web interface
- :pencil2: Doc new tile endpoint and admin interface

## v0.3.1

- :pencil2: Huge doc update on endpoints & setting up
- :rocket: Update a ton of install & Docker instructions

## v0.3.0 

- :tada: Add bounds API

## v0.2.0

- :tada: Authentication on all endpoints!

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

