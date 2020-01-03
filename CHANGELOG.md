# CHANGELOG

## Emoji Cheatsheet
- :pencil2: doc updates
- :bug: when fixing a bug
- :rocket: when making general improvements
- :white_check_mark: when adding tests
- :arrow_up: when upgrading dependencies
- :tada: when adding new features

# Version History

## v0.83.0

- :rocket: Refactor project into cargo workspace
- :rocket: Rewrite auth module to use Derivable traits for far better code security & readability

## v0.82.2

- :rocket: Huge number of rust clippy improvements

## v0.82.1

- :bug: `user_modify_info` function requires write DB access

## v0.82.0

- :tada: [UI] Add ability for admin to make changes to user accounts
- :tada: Add `user_modify_info` endpoint
- :tada: Add new `disabled` user access type
- :rocket: Switch Auth access value to Enum
- :rocket: [UI] Add `insufficient permission` warnings to parts of the UI

## v0.81.0

- :tada: Add `disabled` type for custom auth
- :rocket: Rewrite Auth struct to remove boilerplate `allows_*`  methods and allow direct auth checks
- :rocket: Simplify webhook permissions to `get/set`
- :rocket: Simplify meta permission to `get/set`
- :rocket: Simplify stats permission to `get`
- :rocket: Create JSON -> Struct trait for each section for manual JSON parsing and better error messages
- :rocket Ensure `disabled` type is respected

## v0.80.0

- :rocket: Create new `geo_history` table to retrieve feature history
- :rocket: Remove `features` column from the `deltas` table; features are now stored in `geo_history`
- :bug: Make responses from the deltas and feature history API consistent
- :rocket: Add `features/history` streaming endpoint
- :rocket: Add `migrations` directory to track database migrations

## v0.79.0

- :rocket: Even more uses of `web::block` to make more calls async in nature

## v0.78.0

- :rocket: Use `web::block` to make a number of calls async in nature

## v0.77.3

- :bug: Revert v0.77.2 due to timeout issues
- :white_check_mark: Add tests for session invalidation

## v0.77.2

- :bug: Don't return duplicates on get_bounds when a point exists on a subdivision line

## v0.77.1

- :rocket: Move from nightly to stable rust

## v0.77.0

- :arrow_up: Update all general deps to latest versions
- :rocket: Add API endpoint and UI functionality to reset a user password

## v0.76.0

- :bug: Ensure MVT::get fn is able to write cached tile to DB

## v0.75.0

- :rocket: Don't include `web/dist` folder in git to avoid constant conflicts
- :pencil2: Update docs around building frontend UI

## v0.74.4

- :rocket: Add automatic session cookie clearing
- :rocket: Set the expiration time on JOSM urls tokens to 2 weeks and on user session tokens to 24 hours

## v0.74.3

- :rocket: Add webhook secrets so receivers can authenticate webhook requests from Hecate
- :rocket: [UI] Show webhook secret when creating a webhook

## v0.74.2

- :rocket: [UI] Add protocol in JOSM token generation

## v0.74.1

- :bug: Use read/write database connection for webhook create, update, and delete requests

## v0.74.0

- :rocket: Add create and delete token endpoints, and allow for path-based tokens to fix JOSM
    integration when basic authentication is not sent
- :rocket: Add read/write scoping to every endpoint

## v0.73.0

- :rocket: Add identity middleware to require every endpoint for authentication, returning 401
    or redirecting to login page if the request is not authenticated

## v0.72.0

- :rocket: Switch backend from `rocket` to `actix`

## v0.71.1

- :rocket: Add validation for bbox.

## v0.71.0

- :rocket: [UI] Completely rework UI based error handling into a single error handling component
- :rocket: [UI] Move more UI API calls into the standard hecate library
- :rocket: [UI] Add support for loading the server schema at load time
- :tada: [UI] Add support for displaying schema descriptions in the features panel
- :tada: [UI] Add support for optionally loading the feature history
- :tada: [UI] Rough support for feature history UI

## v0.70.5

- :rocket: [UI] Show back button when individual feature is selected of multiple potential

## v0.70.4

- :rocket: [UI] Add support for multiple feature selection via click UI

## v0.70.3

- :rocket: Add feature limits to tiles

## v0.70.2

- :bug: Limit tile geometry pool with `ST_Intersects`

## v0.70.1

- :bug: [UI] Update maxzoom to 17 in main map

## v0.70.0

- :rocket: Switch to `ST_AsMVT` instead of server side generation
- :rocket: Add support for z17-z15 (previously ended at z14) tile generation
- :rocket: Add ability to query multiple features at a given point

## v0.69.0

- :rocket: Add preliminary support for webhooks

## v0.68.0

- :rocket: Use `ST_DWithin` for feature at lng/lat endpoint
- [UI] Swith to using lng/lat feature API instead of ID lookup

## v0.67.0

- :rocket: Framework for supporting delta based operations
- :rocket: Update formatting for readability in `lib.rs`, moving all fn parameters to their own line
- :rocket: Update all possible `r2d2` data types to the generic `postgres::GenericConnection`
- [UI] Update to mapbox-gl-js 0.53

## v0.66.0

- :rocket: Update to 2018 Rust Edition

## v0.65.0

- :tada: Introduce bounds meta API for accessing underlying bounds data
- [UI] Display bounds in UI

## v0.64.0

- :tada: Add daemon for performing background tasks based on api events
- :tada: Regenerate tiles proactively based on tilecover of uploaded geometries

## v0.63.0

- [UI] Add bounds filtering and limit number of bounds shown in API

## v0.62.2

- :rocket: Return `application/json` when retrieving individual features

## v0.62.1

- :arrow_up: Update all deps to latest versions

## v0.62.0

- :bug: Improved session handling to ensure server restarts without `HECATE_SECRET` don't lock session cookie

## v0.61.2

- :bug: Database Sandbox arg fix

## v0.61.1

- :bug: Dynamic PostGIS/PostgreSQL version error message
- :rocket: only test version of write DB connection

## v0.61.0

- :tada: Optimize Boundary Stats by subdividing large/irregularly shaped geometries for faster indexed retrieval.
- :rocket: Redefine database connections into `database`, `database_replica`, `database_sandbox`

## v0.60.0

- :tada: Add postgres/posgis version checking to database checks

## v0.59.1

- :bug: Optimize Bounds#Get with `ST_Subdivide`

## v0.59.0

- :tada: Feature query by lat/lng.

## v0.58.1

- :rocket: Reduce `clone()` calls where possible to decrease allocation calls
- :rocket: Rename `xml` => `osm` mod to better reflect contents

## v0.58.0

- :arrow_up: Update to `geojson@13` which adds support for better representation of `Feature::Id` values
- :tada: Validate coordinates on upload (180,-180,90,-90)
- [UI] Fix styles overflow in admin panel

## v0.57.0

- :tada: Add `HECATE_SECRET` option for providing a 256 bit base64 key for cookie signing (`openssl rand --base64 32`)

## v0.56.0

- :bug: Explicitly set the cookie path

## v0.55.1

- :arrow_up: `Rocket@0.4.0`

## v0.55.0

- :tada: Add standard error class to store and parse errors from across the server
- :rocket: Change the majority of custom error objects to use the new error class
- :rocket: Remove unneeded `extern crate` calls

## v0.54.0

- :arrow_up: [BREAKING] Update to `nightly-2018-12-01` per README instructions to compile
- :arrow_up: Update Rocket to RC2
- :arrow_up: General dep updates

## v0.53.0

- :tada: Add new `/api/data/stats/regen` endpoint
- :rocket: Make `hecate` ROLE database owner for regen endpoint

## v0.52.0


- :rocket: Add filter consistency to users and bounds APIs
- :tada: `users` has `limit` & `filter` param
- :tada: `bounds` has `limit` & `filter` param
- :rocket: `users` & `bounds` no longer require 2 function with rocket@0.4

## v0.51.0

- :tada: Add database check to ensure connections work before starting server

## v0.50.1

- :bug: Fix regression where Data => String is not supported in release builds

## v0.50.0

- :arrow_up: [BREAKING] Update to `nightly-2018-11-19` per README instructions to compile
- :arrow_up: Update to Rocket@0.4

## v0.49.3

- [UI] Much better session handling, refreshing the page no longer marks you as logged out, when you actually still have a valid session cookie!
- [UI] Add User Listing to admin panel
- [UI] Add Self Modal to view your own account information when signed in
- [UI] Add clickable usernames to open a user's public profile
- [UI] Load auth settings at startup to allow components to conditionally request content - avoiding 401 errors where possible by telling the user they aren't logged in/aren't admin, etc.
- [UI] Fix turf bbox calculation bug in Delta Panel
- [UI] Fix Foot component error in Features Panel

## v0.49.2

- [UI] More whitespace fixes in deltas panel

## v0.49.1

- [UI] Fix spacing on main toolbar
- [UI] Fix whitespace & overflow on delta message

## v0.49.0

- :rocket: Make fallback for `database_read` flag
- :tada: Schema now creates a `hecate` & `hecate_read` user instead of using the `postgres` user by default

## v0.48.3

- :arrow_up: Update to latest deps

## v0.48.2

- :rocket: [UI] Show JOSM changeset messages in delta panel

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
