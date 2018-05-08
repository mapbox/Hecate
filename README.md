<h1 align='center'>Hecate</h1>

<p align=center>OpenStreetMap Inspired Data Storage Backend Focused on Performance and GeoJSON Interchange</p>

<p align="center">
  <a href="https://circleci.com/gh/ingalls/Hecate/tree/master"><img src="https://circleci.com/gh/ingalls/Hecate/tree/master.svg?style=shield"/></a>
  <a href="https://crates.io/crates/hecate"><img src="https://img.shields.io/crates/v/hecate.svg"/></a>
</p>

## Table Of Contents

1. [Brief](#brief)
2. [Table of Contents](#table-of-contents)
3. [Build Environment](#build-environment)
3. [Docker File](#docker-file-coverage-tests)
4. [Feature Format](#feature-format)
5. [Server](#server)
    - [Database Connection](#database)
    - [JSON Validation](#json-validation)
6. [API](#api)
    - [User Options](#user-options)
    - [Admin Interface](#admin-interface)
    - [Schema](#schema)
    - [Styles](#styles)
    - [Vector Tiles](#vector-tiles)
    - [Downloading via Boundaries](#downloading-via-boundaries)
    - [Downloading Individual Features](#downloading-individual-features)
    - [Downloading Multiple Features via BBOX](#downloading-multiple-features-via-bbox)
    - [Feature Creation](#feature-creation)
    - [Deltas](#deltas)
    - [OpenStreetMap API](#openstreetmap-api)

## Build Environment

- Start by installing Rust from rust-lang.org, this will install the current stagle version

```bash
curl https://sh.rustup.rs -sSf | sh
```

- Source your bashrc/bash_profile to update your `PATH` variable

```bash
source ~/.bashrc        # Most Linux Distros, some OSX
source ~/.bash_profile  # Most OSX, some Linux Distros
```

- Install the `nightly-2018-01-13` build of rust, `Rocket`, the web-framework relies on some advanced compiler options not yet included in the default build.

```bash
rustup install nightly-2018-01-13
rustup default nightly-2018-01-13
```

- Download and compile the project and all of it's libraries

```bash
cargo build
```

- Create the `hecate` database using the provided schema file.

```bash
echo "CREATE DATABASE hecate;" | psql -U postgres

psql -U postgres -f src/schema.sql hecate
``` 

- Start the server

```bash
cargo run
```

- Test it is working - should respond with `HTTP200`

```bash
curl 'localhost:8000
```

You will now have an empty database which can be populated with your own data/user accounts.

If you want to populate the database with sample data for testing, [ingalls/hecate-example](https://github.com/ingalls/hecate-example)
has a selection of scripts to populate the database with test data.

## Docker File (Coverage Tests)

The Docker file is designed to give the user a testing environment to get rust tests to generate code coverage information

Install docker and then run

```
docker build .

docker run  --security-opt seccomp=unconfined {{HASH FROM ABOVE}}
```

The --security-opt flag is required to be able to run and view `kcov` output.

## Feature Format

Hecate is designed as a GeoJSON first interchange and uses [standard GeoJSON](http://geojson.org/) with a couple additions
and exceptions as outlined below.

*Supported Geometry Types*
- `Point`
- `MultiPoint`
- `LineString`
- `MultiLineString`
- `Polygon`
- `MultiPolygon`

*Unsupported Geometry Types*
- `GeometryCollection`

### Additional Members

The following table outlines top-level members used by hecate to handle feature creation/modification/deletion.

Key/Value pairs in the `.properties` of a given feature are _never_ directly used by the server and are simply
passed through to the storage backend. This prevents potential conflicts between user properties and required
server members.

| Member    | Notes |
| :-------: | ----- |
| `id`      | The unique integer `id` of a given feature. Note that all features get a unique id accross GeoJSON Geometry Type |
| `version` | The version of a given feature, starts at `1` for a newly created feature |
| `action`  | Only used for uploads, the desired action to be performed. One of `create`, `modify` or `delete` |


### Examples

#### Downloaded Features

```JSON
{
    "id": 123,
    "version": 2,
    "type": "Feature",
    "properties": {
        "shop": true,
        "name": "If Pigs Could Fly"
    },
    "geometry": {
        "type": "Point",
        "coordinates": [0,0]
    }
}
```

Downloaded Features will return the integer `id` of the feature, the current `version` and the user supplied `properties` and `geojson`.
`action` is not applicable for downloaded features, it is only used on upload.

#### Create Features

```JSON
{
    "action": "create",
    "type": "Feature",
    "properties": {
        "shop": true,
        "name": "If Pigs Could Fly"
    },
    "geometry": {
        "type": "Point",
        "coordinates": [0,0]
    }
}
```

A features being uploaded for creation must have the `action: create` property. Since an `id` and `version` have not yet been
assigned they should be omitted. Should an `id` or `version` be included the server will ignore them, assigning a new
`id` and `version` per the servers internal id provisioner.

#### Modify Features

```JSON
{
    "id": 123,
    "version": 1,
    "action": "modify",
    "type": "Feature",
    "properties": {
        "shop": true,
        "name": "If Pigs Could Fly"
    },
    "geometry": {
        "type": "Point",
        "coordinates": [0,0]
    }
}
```

A feature being uploaded for modification must have the `action: modify` as well as the `id` and `version` property. The `id` is the integer id of the feature to modify and the `version` property is the 
current version of the feature as stored by the server. If the version uploaded does not match the version that the server has stored, the modify will fail. This prevents consecutive edits from conflicting.

Note that the modify operation is _not a delta operation_ and the full feature with the complete Geometry & All Properties must be included with each modify.

Also note that since the `id` pool is shared accross geometry types, an id is allowed to change it's geometry type. eg. If `id: 1` is a `Point` and then a subsequent `action: modify` with a `Polygon` geometry is performed, `id: 1` is allowed to switch to the new `Polygon` type.

#### Delete Features

```JSON
{
    "id": 123,
    "version": 1,
    "action": "delete",
    "type": "Feature",
    "properties": null,
    "geometry": null
}
```

A feature being uploaded for deletion must have the `action: delete` as well as the `id` and `version` property. See _Modify Features_ above for an explanation of those properties.

Note the `properties` and `geometry` attributes must still be included. They can be set to `null` or be their previous value. They will be ignored.

### Samples

## Server

This section of the guide goes over various options on has when launching the server

Hecate can be launched with default options with

```
cargo run
```

### Database

By default hecate will attempt to connect to `postgres@localhost:5432/hecate`.

Note that only postgres/postgis backed databases are currently supported.

This database should be created prior to launching hecate. For instructions on setting up the database
see the [Build Environment](#build-environment) section of this doc.

A custom database name, postgres user or port can be specified using the database flag.

```
cargo run -- --database "<USER>:<PASSWORD>@<HOST>/<DATABASE>"

cargo run -- --database "<USER>@<HOST>/<DATABASE>"
```

### JSON Validation

By default hecate will allow any properties on a given GeoJSON feature, including nestled arrays, maps, etc.

A custom property validation file can be specified using the schema flag.

```
cargo run -- --schema <PATH-TO-SCHEMA>.json
```

Note hecate currently supports the JSON Schema draft-04. Once draft-06/07 support lands in
[valico](https://github.com/rustless/valico) we can support newer versions of the spec.

## API

### Index

#### `GET` `/`

Healthcheck URL, currently returns `Hello World!`

*Example*

```bash
curl -X GET 'http://localhost:8000/
```

---

<h3 align='center'>Admin Interface</h3>

View the Admin Interface in your browser by pointing to `127.0.0.1:8000/admin/index.html`

---

<h3 align='center'>Styles</h3>

#### `GET` `/api/styles`

Return an array containing a reference to every public style

*Example*

```bash
curl -X GET 'http://localhost:8000/api/styles'
```

---

#### `GET` `/api/styles/<user id>`

Return an array containing styles owned by a particular user.

By default any request will only return the public styles for a given user.

If an authenticated user requests their own styles, it will return their public and private styles.

*Options*

| Option | Notes |
| :----: | ----- |
| `<user id>` | `REQUIRED` Numeric ID of the user to get styles from |

*Example*

Return only public styles of user 1

```bash
curl -X GET 'http://localhost:8000/api/styles/1'
```

User requesting their own styles will get public & private styles

```bash
curl -X GET 'http://username:password@localhost:8000/api/styles/1'
```

---

#### `POST` `/api/style`

Create a new private style attached to the authenticated user

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"name": "Name of this particular style", "style": "Mapbox Style Object Here"}' \
    'http://username:password@localhost:8000/api/style'
```

---

#### `DELETE` `/api/style/<id>`

Delete a particular style by id. Users must be authorized and 
can only delete styles created by them.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to delete |

*Example*

```bash
curl -X DELETE 'http://localhost:8000/api/style/1'
```

---

#### `GET` `/api/style/<id>`

Get a particular style by id, public styles can be requested unauthenticated,
private styles can only be obtained by the corresponding user making the request.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/style/1'
```

---

#### `PATCH` `/api/style/<id>`

Update a style - auth required - users can only update their own styles

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"name": "New Name", "style": "New Mapbox Style Object Here"}' \
    'http://username:password@localhost:8000/api/style/1'
```

---

#### `POST` `/api/style/<id>/private`

Update a public style and mark it as private.

Note: Once a style is public other users may have cloned it. This will not
affect cloned styles that were made when it was public.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl -X POST 'http://username:password@localhost:8000/api/style/1/private'
```

---

#### `POST` `/api/style/<id>/public`

Update a style to make it public.

It will then appear to all users in the global styles list
and other users will be able to download, clone, and use it

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given style to download |

*Example*

```bash
curl -X POST 'http://username:password@localhost:8000/api/style/1/public'
```

---

<h3 align='center'>Schema</h3>

#### `GET` `/api/schema`

Return a JSON object containing the schema used by the server or return a 404 if no schema file is in use.


*Example*

```bash
curl -X GET 'http://localhost:8000/api/schema
```

---

<h3 align='center'>Vector Tiles</h3>

#### `GET` `/api/tiles/<z>/<x>/<y>`

Request a vector tile for a given set of coordinates. A [Mapbox Vector Tile](https://www.mapbox.com/vector-tiles/) is returned.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<z>` | `REQUIRED` Desired zoom level for tile
| `<x>` | `REQUIRED` Desired x coordinate for tile
| `<y>` | `REQUIRED` Desired y coordinate for tle

*Example*

```bash
curl -X GET 'http://localhost:8000/api/tiles/1/1/1
```

---

#### `GET` `/api/tiles/<z>/<x>/<y>/meta`

Return any stored metadata about a given tile.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<z>` | `REQUIRED` Desired zoom level for tile
| `<x>` | `REQUIRED` Desired x coordinate for tile
| `<y>` | `REQUIRED` Desired y coordinate for tle

*Example*

```bash
curl -X GET 'http://localhost:8000/api/tiles/1/1/1/meta
```

---

#### `GET` `/api/tiles/<z>/<x>/<y>/regen`

Allows an authenticated user to request a new tile for the given tile coordinates,
ensuring the tile isn't returned from the tile cache.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<z>` | `REQUIRED` Desired zoom level for tile
| `<x>` | `REQUIRED` Desired x coordinate for tile
| `<y>` | `REQUIRED` Desired y coordinate for tle

*Example*

```bash
curl -X GET 'http://username:password@localhost:8000/api/tiles/1/1/1/regen
```

---

<h3 align='center'>User Options</h3>

#### `GET` `/api/user/create`

Create a new user, provied the username & email are not already taken

*Options*

| Option     | Notes |
| :--------: | ----- |
| `username` | `REQUIRED` Desired username, must be unique |
| `password` | `REQUIRED` Desired password |
| `email`    | `REQUIRED` Desired email, must be unique |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/user/create?ingalls&password=yeaheh&email=ingalls@protonmail.com
```

---

#### `GET` `/api/user/session`

Return a new session cookie and the `uid` given an Basic Authenticated request.

*Example*

```bash
curl -X GET 'http://username:password@localhost:8000/api/user/session
```

---

<h3 align='center'>Downloading via Boundaries</h3>

#### `GET` `/api/data/bounds/`

Return an array of possible boundary files with which data can be extracted from the server with

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/bounds
```

---

#### `GET` `/api/data/bounds/<bounds>`

Return line delimited GeoJSON `Feature` of all the geometries within the specified boundary file.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<bounds>` | `REQUIRED` One of the boundary files as specified via the `/ap/data/bounds` |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/bounds/us_dc
```

---

<h3 align='center'>Downloading Individual Features</h3>

#### `GET` `/api/data/feature/<id>`

Return a single GeoJSON `Feature` given its' ID.

*Options*

| Option | Notes |
| :----: | ----- |
| `<id>` | `REQUIRED` Numeric ID of a given feature to download |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/data/features/1542
```

---

<h3 align='center'>Downloading Multiple Features via BBOX</h3>

#### `GET` `/api/data/features`

Return a `FeatureCollection` of all features within a given bbox

*Options*

| Option | Notes |
| :----: | ----- |
| `bbox` | `REQUIRED` Bounding Box in format `left,bottom,right,top` |

---

<h3 align='center'>Feature Creation</h3>

#### `POST` `/api/data/feature` *Auth Required*

Create, Modify, or Delete an individual GeoJSON `Feature`

The Feature must follow format defined in [Feature Format](#feature-format).

The feature also must contain a top-level String `message` attribute describing the changes being made (The delta message)

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"action": "create", "message": "Random Changes", "type":"Feature","properties":{"shop": true},"geometry":{"type":"Point","coordinates":[0,0]}}' \
    'http://username:password@localhost:8000/api/data/feature'
```

---

#### `POST` `/api/data/features` *Auth Required*

Create, Modify, and/or Delete many features via a GeoJSON `FeatureCollection`

The Features in the FeatureCollection must follow format defined in [Feature Format](#feature-format).

The FeatureCollection also must contain a top-level String `message` attribute describing the changes being made (The delta message)

Note that a mix of `create`, `modify`, and `delete` operatioons are allowed
within each `FeatureCollection`

*Example*

```bash
curl \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{"type":"FeatureCollection","message":"A bunch of changes","features": [{"action": "create", "type":"Feature","properties":{"shop": true},"geometry":{"type":"Point","coordinates":[0,0]}}]}' \
    'http://username:password@localhost:8000/api/data/features'
```

---

<h3 align='center'>Deltas</h3>

#### `GET` `/api/deltas`

Returns an array of the last 20 deltas with their corresponding metadata. Does not include geometric
data on the delta. Request a specific delta to get geometric data.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `offset` | `OPTIONAL` Offset the returned 20 values by a given integer |

*Example*

```bash
curl -X GET 'http://localhost:8000/api/deltas
```

```bash
curl -X GET 'http://localhost:8000/api/deltas?offset=3
```

---

#### `GET` `/api/deltas/<id>`

Returns all data for a given delta as a JSON Object, including geometric data.

*Options*

| Option     | Notes |
| :--------: | ----- |
| `<id>` | `REQUIRED` Get all data on a given delta

*Example*

```bash
curl -X GET 'http://localhost:8000/api/delta/4
```

---

<h3 align='center'>OpenStreetMap API</h3>

The primary goal of the hecate project is a very fast GeoJSON based Interchange. That said, the tooling the OSM community has built around editing is unparalled. As such, 
Hecate provides a Work-In-Progress OpenStreetMap Shim to support a subset of API operations as defined by the [OSM API v0.6](httpl://wiki.openstreetmap.org/wiki/API_v0.6) document.

*Important Notes*
- All GeoJSON types can be downloaded via the API and viewed in JOSM
- MultiPoints
    - Are represented using an OSM  `Relation`
    - The type will be `multipoint`
    - The member type will be `point`
- MultiLineStrings
    - Are represented using an OSM `Relation`
    - The type will be `multilinestring`
    - The member will be `line`
- Uploading `Way` & `Relation` types are not currently supported, attempting to upload them may produce undesirable results.

The following incomplete list of endpoints are implemented with some degree of coverage with the OSM API Spec but are likely incomplete/or written with the minimum flexibility required to
support editing from JOSM. See the code for a full list.

#### `GET` `/api/capabilities`
#### `GET` `/api/0.6/capabilities`

Return a static XML document describing the capabilities of the API.

*Example*

```bash
curl -X GET 'http://localhost:8000/api/capabilities
```

---

#### `GET` `/api/0.6/user/details` *Auth Required*

Returns a static XML document describing the number of unread messages that a user has. Every n minutes JOSM checks
this and displays in the interface if there is a new message, to cut down on errors it simply returns a 0 message response.

*Example*

```bash
curl -X GET 'http://localhost:8000/api/0.6/user/details
```

---

#### `PUT` `/api/0.6/changeset/create` *Auth Required*

Create a new changset and set the meta information, returning the opened id.

*Example*

```bash
curl \
    -X PUT \
    -d '<osm><changeset><tag k="comment" v="Just adding some streetnames"/></changeset></osm>' \
    'http://localhost:8000/api/0.6/changeset/create
```

---

#### `GET` `/api/0.6/changeset/<changeset_id>/upload` *Auth Required*

Upload osm xml data to a given changeset

*Example*

```bash
curl \
    -X POST \
    -d '<diffResult version="0.6">NODE/WAY/RELATIONS here</diffResult>' \
    'http://localhost:8000/api/0.6/changeset/1/upload'
```

---

#### `PUT` `/api/0.6/changeset/<changeset_id>/close` *Auth Required*

Close a given changeset, preventing further modification to it

*Example*

```bash
curl -X PUT 'http://localhost:8000/api/0.6/changeset/1/close'
```

---
