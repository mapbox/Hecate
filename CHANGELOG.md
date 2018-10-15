# CHANGELOG

## Emoji Cheatsheet
- :pencil2: doc updates
- :bug: when fixing a bug
- :rocket: when making general improvements
- :white_check_mark: when adding tests
- :arrow_up: when upgrading dependencies
- :tada: when adding new features

# Version History

## v0.48.1

- :rocket: [UI] Add cursor change on hover events in mapbox-gl

# v0.48.0
- :rocket: Add endpoint for clearing the tile cache
- :rocket: Add UI Fxn to clear tile cache

## v0.47.0

- :bug: Meta Set was incorrectly set to check meta::list permission
- :tada: [UI] Add new server settings - allowing management of default styles
- :tada: [UI] Add default styles switching

## v0.46.0

- :tada: Add support for configurable web workers to allow custom levels of parallelism

## v0.45.0

- :rocket: Add EOT Character at end of streaming API calls

## v0.44.1

- :rocket: Add redirect from `/admin` => `/admin/index.html`

## v0.44.0

- :tada: Allow creating and deleting bounds programatically

## v0.43.0

- :tada: Add an endpoint for allowing an admin to add admins who can add more admins
- :tada: Add endpoint for admins to remove other admins
- :tada: Add endpoint for searching list of users

## v0.42.0

- :rocket: Add generic server meta store

## v0.41.0

- :rocket: Add Data Statistics API for Boundaries

## v0.40.0

- :rocket: Add Data Statistics API

## v0.39.0

- :bug: [UI] Ensure protocol (http(s)) is respected
- :rocket: [UI] Add paging fxn for deltas panel
- :bug: [UI] Ensure panels don't collide with mapbox logo

## v0.38.1

- :bug: Fix version mismatch

## v0.38.0

- :rocket: All query style endpoints now use read-only postgres account
- :tada: Allow multiple read-only postgres connections & load balancing between them

## v0.37.2

- :rocket: remove all `unwrap()` `feature` mod

## v0.37.1

- :white_check_mark: Add test for missing Action on features
- :bug: :white_check_mark: Ensure non-string `key` values error gracefully

## v0.37.0

- :rocket: Ensure app API requests include domain cookies

## v0.36.1

- :rocket: [UI] Add style deletion 
- :bug: [UI] Ensure feature requests didn't poll infinitely

## v0.36.0

- :rocket: Rewrite web UI to use Vue Components for better readability
- :tada: Add logout endpoint for deletion of session token

## v0.35.2

- :bug: Take 2 - Use `0.0.0.0` to make docker a happy camper

## v0.35.1

- :bug: Use `localhost` to make docker a happy camper

## v0.35.0

- :rocket: Remove `Rocket.toml`, replacing with CLI options

## v0.34.0

- :rocket: Add CLI option for manually specifying read only db connection

## v0.33.0

- :tada: Add arbitrary SQL query endpoint

## v0.32.1

- :white_check_mark: Add `FeatureCollection` force tests

## v0.32.0

- :tada: Add `force: true` option for force overwriting `create` features.

## v0.31.0

- :rocket: `Feature` and `FeatureCollection` uploads now will return the specific feature/id if there is an error

## v0.30.1

- :arrow_up: Update to latest deps

## v0.30.0

- :bug: Ensure `affected` array is the actual feature id on create and not the user specified id
- :rocket: Throw an error if a `version` is provided on create

## v0.29.0

- :rocket: Allow querying features by `key` value in addition to their assigned id.

## v0.28.0

- :tada: Add `start`, `end` and `limit` options to delta list API

## v0.27.2

- :arrow_up: Remove unused `geo` dependency

## v0.27.1

- :bug: A `key: null` JSON property on an uploaded feature should not return a `Duplicate Key Value` error when attempting an upload. Duplicate `null` is allowed

## v0.27.0

- :tada: Add optional `key` value to allow the user to specify a duplication avoidance policy

## v0.26.0

- :tada: Add new `action: restore` on Features to be able to restore previously deleted features to the given id

## v0.25.0

- :white_check_mark: Add a bunch of tests around the `deltas` endpoints
- :bug: change behavior of `?offset=` param to return the anticipated results

## v0.24.1

- :bug: Fix session token bug preventing style creation
- :bug: (UI) Fix stringification of function of new style creation

## v0.24.0

- :tada: Add `GET /api/auth` endpoint for retrieving overview of auth settings

## v0.23.0

- :rocket: Consistent Streaming Line-Delimited GeoJSON Output

## v0.22.0

- :tada: Add data clone API

## v0.21.1

- :rocket: Refector `BoundsStream` into a new generic `PGStream`

## v0.21.0

- :tada: Add custom authentication config

## v0.20.3

- :rocket: Migrate all BoundsStream implementation details into the `bounds` mod for cleaner `lib.rs` file

## v0.20.2

- :rocket: Add ability to toggle style access `public/private` via web UI

## v0.20.1

- :bug: Fix negative feature ids entering deltas from xml_shim

## v0.20.0

- :tada: Add feature history endpoint

## v0.19.0

- :tada: Add API Meta endpoint

## v0.18.0

- :arrow_up: `rust@nightly-2018-05-05`
- :arrow_up: Update all deps to latest versions

## v0.17.0

- :tada: Add `GET /api/tiles/<z>/<x>/<y>/meta` endpoint & assoc. tests

## v0.16.0

- :tada: Add Styles UI
- :rocket: Return Style ID on create
- :rocket: Return `uid` on session token

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

